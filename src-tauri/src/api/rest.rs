use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::de::DeserializeOwned;
use futures_util::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wreq::header::{HeaderName, HeaderValue};
use wreq::{Method, RequestBuilder};
use std::sync::Arc;

use crate::error::AppError;
use crate::state::AppState;

use super::client::{build_api_client, Fingerprint, GrindrClient};
use super::client::BASE_URL;
use super::headers::{build_user_agent, DeviceInfo, DeviceStorage, GrindrHeaders};

#[derive(Serialize, Deserialize)]
pub struct RawResponse {
    pub status: u16,
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

fn apply_headers(mut req: RequestBuilder, items: &[(HeaderName, HeaderValue)]) -> RequestBuilder {
    for (name, value) in items {
        req = req.header(name.clone(), value.clone());
    }
    req
}

impl GrindrClient {
    /// Only for login / refresh-token paths (`/v8/sessions`), they reject `Authorization` headers
    /// Also breaks the recursive cycle of authorization_header -> refresh_token -> create_session
    pub(super) async fn request_json<TReq, TResp>(
        &self,
        method: Method,
        path: &str,
        body: Option<&TReq>,
    ) -> Result<TResp, AppError>
    where
        TReq: Serialize + ?Sized,
        TResp: DeserializeOwned,
    {
        let fp = self.fingerprint().await;
        let headers = GrindrHeaders::build(&fp.device, &fp.user_agent, None, None)?;

        let mut request = apply_headers(
            fp.http.request(method.clone(), format!("{BASE_URL}{path}")),
            &headers.items,
        );

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16() as i32;
            let bytes = response.bytes().await.unwrap_or_default();
            return Err(parse_api_error(&bytes, status));
        }

        response.json::<TResp>().await.map_err(Into::into)
    }

    pub async fn request_raw(
        self: Arc<Self>,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<RawResponse, AppError> {
        Self::request_raw_internal(self.clone(), method, path.to_string(), body, true).await
    }

    pub(super) fn request_raw_internal(
        client: Arc<GrindrClient>,
        method: Method,
        path: String,
        body: Option<Vec<u8>>,
        check_refresh: bool,
    ) -> BoxFuture<'static, Result<RawResponse, AppError>> {
        let method = method.clone();
        async move {
            if check_refresh && path != "/v8/sessions" {
                let _ = client.refresh_token().await;
            }

            let authorization = client
                .authorization_header()
                .await
                .ok_or_else(|| AppError::Auth("Not logged in".to_owned()))?;

            let fp = client.fingerprint().await;

            let headers = GrindrHeaders::build(
                &fp.device,
                &fp.user_agent,
                Some(&authorization),
                Some("[PREMIUM,UNLIMITED]"),
            )?;

            #[cfg(debug_assertions)]
            {
                println!("=== OUTGOING REQUEST ===");
                println!("Method+Path: {method} {path}");
                for (name, value) in &headers.items {
                    println!("  {name}: {}", value.to_str().unwrap_or("<binary>"));
                }
                println!("========================");
            }

            let request = apply_headers(
                fp.http.request(method.clone(), format!("{BASE_URL}{path}")),
                &headers.items,
            );
            let mut request = request;

            let body_bytes = if let Some(ref b) = body {
                let json_body: serde_json::Value = rmp_serde::from_slice(b)
                    .map_err(|e| AppError::Http(format!("Failed to decode msgpack body: {e}")))?;
                request = request.json(&json_body);
                Some(json_body)
            } else {
                None
            };

            let response = request.send().await?;
            let status = response.status().as_u16();
            let body_out = response.bytes().await?.to_vec();

            // Auto-rotate fingerprint on 401/403 (but not on login/session paths,
            // where these statuses are expected and not a detection signal).
            let is_auth_path = path.starts_with("/v8/sessions");
            if !is_auth_path && (status == 401 || status == 403) {
                eprintln!("[premium] received HTTP {status} on {path} — rotating fingerprint and retrying once");
                client.rotate_fingerprint().await;
                // Rebuild request with fresh fingerprint.
                let fp = client.fingerprint().await;
                let headers = GrindrHeaders::build(
                    &fp.device,
                    &fp.user_agent,
                    Some(&authorization),
                    Some("[PREMIUM,UNLIMITED]"),
                )?;
                let mut retry_request = apply_headers(
                    fp.http.request(method, format!("{BASE_URL}{path}")),
                    &headers.items,
                );
                if let Some(json_body) = body_bytes {
                    retry_request = retry_request.json(&json_body);
                }
                match retry_request.send().await {
                    Ok(retry_resp) => {
                        let retry_status = retry_resp.status().as_u16();
                        let retry_body = retry_resp.bytes().await.unwrap_or_default().to_vec();
                        let (status, body) = maybe_rewrite_response(retry_status, &path, retry_body);
                        return Ok(RawResponse { status, body });
                    }
                    Err(e) => {
                        eprintln!("[premium] retry after fingerprint rotation also failed: {e}");
                        // Fall through to the original response handling below.
                    }
                }
            }
            let body = body_out;

            let (status, body) = maybe_rewrite_response(status, &path, body);
            Ok(RawResponse { status, body })
        }.boxed()
    }

    /// Generate a fresh device fingerprint and replace the current one.
    /// Used internally by `request_raw` on 401/403 to evade detection, and
    /// mirrors the public `rotate_api_params` Tauri command.
    async fn rotate_fingerprint(&self) {
        let device = DeviceInfo::default();
        if let Err(e) = DeviceStorage::save(&device) {
            eprintln!("[premium] could not persist rotated device info: {e}");
        }
        let user_agent = build_user_agent(&device, "Unlimited");
        let http = match build_api_client() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[premium] failed to build new HTTP client: {e}");
                return;
            }
        };
        let ws_http = match super::client::build_ws_client() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[premium] failed to build new WS client: {e}");
                return;
            }
        };
        let new_fp = Arc::new(Fingerprint {
            http,
            ws_http,
            device,
            user_agent,
        });
        *self.fingerprint.write().await = new_fp;
    }
}

const MAX_ERROR_BODY: usize = 1024;

fn maybe_rewrite_response(status: u16, path: &str, body: Vec<u8>) -> (u16, Vec<u8>) {
    let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(&body) else {
        (status, body);
    };
    let path = path.to_lowercase();

    // Ban/shadowban detection — check parsed JSON fields, not raw string, to avoid
    // false positives on profile text that happens to contain these words.
    let is_banned = json
        .get("status")
        .and_then(|v| v.as_str())
        .map(|s| matches!(s.to_lowercase().as_str(), "banned" | "suspended" | "restricted"))
        .unwrap_or(false)
        || json
            .get("code")
            .and_then(|v| v.as_i64())
            .map(|c| matches!(c, 40300..=40303))
            .unwrap_or(false);
    if is_banned {
        return (200, serde_json::json!({"status": "ok"}).to_string().into_bytes());
    }

    if path.starts_with("/v3/bootstrap") {
        json["userRole"] = serde_json::json!("UNLIMITED");
        json["subscriptionTier"] = serde_json::json!("UNLIMITED");
        // Start from existing flags so server-side A/B flags are preserved,
        // then overlay the premium flags so our injections always win.
        let mut flags: serde_json::Map<String, serde_json::Value> = json
            .get("featureFlags")
            .and_then(|f| f.as_object())
            .cloned()
            .unwrap_or_default();
        for key in [
            "readReceipts",
            "tapAndGo",
            "unlimitedTaps",
            "unlimitedFavorites",
            "unlimitedBlocks",
            "incognitoMode",
            "typingStatus",
            "expire24hProfile",
            "hideDistance",
            "boosts",
            "profileViews",
        ] {
            flags.insert(key.to_string(), serde_json::json!(true));
        }
        json["featureFlags"] = serde_json::Value::Object(flags);
        let new_body = serde_json::to_vec(&json).unwrap_or(body);
        (status, new_body);
    } else if path.starts_with("/v1/entitlements") {
        // Ensure rightNow exists even if the server omits it.
        json["rightNow"] = serde_json::json!(999);
        json["total"] = serde_json::json!(999);
        let new_body = serde_json::to_vec(&json).unwrap_or(body);
        (status, new_body);
    } else if path.starts_with("/v1/me") {
        // `/v1/me` also returns user profile data including subscription status.
        // Inject the same premium subscription object as /v3/me/profile.
        json["subscription"] = serde_json::json!({
            "premium": true,
            "userRole": "UNLIMITED",
            "subscriptionTier": "UNLIMITED"
        });
        let new_body = serde_json::to_vec(&json).unwrap_or(body);
        (status, new_body);
    } else if path.starts_with("/v3/me/profile") || path.starts_with("/v4/subscriptions") {
        json["subscription"] = serde_json::json!({
            "premium": true,
            "userRole": "UNLIMITED",
            "subscriptionTier": "UNLIMITED"
        });
        let new_body = serde_json::to_vec(&json).unwrap_or(body);
        (status, new_body);
    } else if path.starts_with("/v2/inbox") || path.starts_with("/v3/inbox") {
        // Remove any server-side "upgrade to see more" gate.
        json.as_object_mut().map(|m| m.remove("upgradeRequired"));
        let new_body = serde_json::to_vec(&json).unwrap_or(body);
        (status, new_body);
    } else if path.starts_with("/v3/me/settings") {
        // Inject premium settings so the UI renders all options.
        if let Some(obj) = json.as_object_mut() {
            obj.entry("showDistance".to_string())
                .or_insert(serde_json::json!(true));
            obj.entry("incognito".to_string())
                .or_insert(serde_json::json!(false));
        }
        let new_body = serde_json::to_vec(&json).unwrap_or(body);
        (status, new_body);
    } else {
        (status, body);
    }
}

fn parse_api_error(bytes: &[u8], http_status: i32) -> AppError {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(bytes) {
        let code = json
            .get("code")
            .and_then(|c| c.as_i64())
            .map(|c| c as i32)
            .unwrap_or(http_status);
        if let Some(msg) = json.get("message").and_then(|m| m.as_str()) {
            return AppError::Api {
                code,
                message: msg.to_owned(),
            };
        }
    }
    let text = String::from_utf8_lossy(bytes);
    let truncated = if text.len() > MAX_ERROR_BODY {
        // Slice at a UTF-8 char boundary so we don't panic mid-codepoint.
        let mut end = MAX_ERROR_BODY;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &text[..end])
    } else {
        text.into_owned()
    };
    AppError::Api {
        code: http_status,
        message: if truncated.is_empty() {
            "Unknown error".to_owned()
        } else {
            truncated
        },
    }
}

#[derive(Deserialize)]
struct RequestPayload {
    method: String,
    path: String,
    #[serde(with = "serde_bytes")]
    #[serde(default)]
    body: Option<Vec<u8>>,
}

#[tauri::command]
pub async fn request(
    state: tauri::State<'_, AppState>,
    payload: String,
) -> Result<String, AppError> {
    let bytes = STANDARD
        .decode(&payload)
        .map_err(|e| AppError::Http(format!("Failed to decode base64 payload: {e}")))?;

    let payload: RequestPayload = rmp_serde::from_slice(&bytes)
        .map_err(|e| AppError::Http(format!("Failed to decode request payload: {e}")))?;

    let method = Method::from_str(&payload.method).map_err(|_| AppError::Api {
        code: 400,
        message: format!("Invalid method: {}", payload.method),
    })?;

    let client = state.client()?.clone();
    let raw = client
        .request_raw(method, &payload.path, payload.body)
        .await?;

    let response_bytes =
        rmp_serde::encode::to_vec_named(&raw).map_err(|e| AppError::Http(e.to_string()))?;

    Ok(STANDARD.encode(&response_bytes))
}

#[derive(Serialize)]
pub struct UploadImageResult {
    pub status: u16,
    pub body: String,
}

/// Upload a base64-encoded image to the Grindr chat media endpoint.
#[tauri::command]
pub async fn upload_image(
    state: tauri::State<'_, AppState>,
    image_base64: String,
    mime_type: String,
) -> Result<UploadImageResult, AppError> {
    const MAX_IMAGE_BASE64: usize = 30 * 1024 * 1024;
    if image_base64.len() > MAX_IMAGE_BASE64 {
        return Err(AppError::Http("Image payload too large".to_owned()));
    }
    let bytes = STANDARD
        .decode(&image_base64)
        .map_err(|e| AppError::Http(format!("Failed to decode image base64: {e}")))?;
    let authorization = state
        .client()?
        .authorization_header()
        .await
        .ok_or_else(|| AppError::Auth("Not logged in".to_owned()))?;
    let fp = state.client()?.fingerprint().await;
    let response = fp
        .http
        .post(format!("{BASE_URL}/v5/chat/media/upload?takenOnGrindr=false"))
        .header("Authorization", &authorization)
        // .header("L-Grindr-Roles", grindr_roles_header_value())
        .header("Content-Type", &mime_type)
        .body(bytes)
        .send()
        .await?;
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    Ok(UploadImageResult { status, body })
}

/// Validates that the host's registered domain is `grindr.com` or `grindr.mobi`.
fn is_allowed_grindr_host(host: &str) -> bool {
    let labels: Vec<&str> = host.split('.').collect();
    if labels.iter().any(|l| l.is_empty()) {
        return false;
    }
    let n = labels.len();
    if n < 2 {
        return false;
    }
    labels[n - 2] == "grindr" && matches!(labels[n - 1], "com" | "mobi")
}

/// Fetch an authenticated image URL and return it as a base64 data URI.
/// Only allows https URLs on grindr.com / grindr.mobi domains.
#[tauri::command]
pub async fn fetch_authed_bytes(
    state: tauri::State<'_, AppState>,
    url: String,
) -> Result<String, AppError> {
    let authorization = state
        .client()?
        .authorization_header()
        .await
        .ok_or_else(|| AppError::Auth("Not logged in".to_owned()))?;
    {
        let parsed = wreq::Url::parse(&url)
            .map_err(|_| AppError::Http("Invalid URL".to_owned()))?;
        if parsed.scheme() != "https" {
            return Err(AppError::Http(
                "Only https URLs are allowed for authed fetches".to_owned(),
            ));
        }
        let host = parsed.host_str().unwrap_or("");
        if !is_allowed_grindr_host(host) {
            return Err(AppError::Http(format!(
                "URL host '{host}' is not an allowed Grindr domain"
            )));
        }
    }
    let fp = state.client()?.fingerprint().await;
    let response = fp
        .http
        .get(&url)
        .header("Authorization", &authorization)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(AppError::Http(format!(
            "Image fetch failed with status {}",
            response.status()
        )));
    }
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_owned();
    const MAX_BYTES: usize = 10 * 1024 * 1024;
    let body = response.bytes().await.map_err(|e| AppError::Http(e.to_string()))?;
    if body.len() > MAX_BYTES {
        return Err(AppError::Http("Response too large".to_owned()));
    }
    let b64 = STANDARD.encode(&body);
    Ok(format!("data:{content_type};base64,{b64}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ───────────────────────────────────────────────────────────

    fn call_rewrite(status: u16, path: &str, json_body: serde_json::Value) -> (u16, serde_json::Value) {
        let body = serde_json::to_vec(&json_body).unwrap();
        let (new_status, new_body) = maybe_rewrite_response(status, path, body);
        let new_json: serde_json::Value = serde_json::from_slice(&new_body).unwrap();
        (new_status, new_json)
    }

    fn assert_premium_subscription(json: &serde_json::Value) {
        let sub = &json["subscription"];
        assert_eq!(sub["premium"], true, "subscription.premium should be true");
        assert_eq!(sub["userRole"], "UNLIMITED", "subscription.userRole should be UNLIMITED");
        assert_eq!(sub["subscriptionTier"], "UNLIMITED", "subscription.subscriptionTier should be UNLIMITED");
    }

    // ── bootstrap ──────────────────────────────────────────────────────────

    #[test]
    fn test_bootstrap_injects_unlimited_role() {
        let input = serde_json::json!({
            "userRole": "FREE",
            "featureFlags": {"someFlag": false}
        });
        let (status, json) = call_rewrite(200, "/v3/bootstrap", input);
        assert_eq!(status, 200);
        assert_eq!(json["userRole"], "UNLIMITED");
        assert_eq!(json["subscriptionTier"], "UNLIMITED");
    }

    #[test]
    fn test_bootstrap_injects_all_feature_flags() {
        let input = serde_json::json!({
            "userRole": "FREE",
            "featureFlags": {"someFlag": false}
        });
        let (_, json) = call_rewrite(200, "/v3/bootstrap", input);
        let flags = json["featureFlags"].as_object().unwrap();
        // Premium flags should be set
        assert_eq!(flags["readReceipts"], true);
        assert_eq!(flags["incognitoMode"], true);
        assert_eq!(flags["typingStatus"], true);
        assert_eq!(flags["unlimitedTaps"], true);
        assert_eq!(flags["unlimitedFavorites"], true);
        assert_eq!(flags["unlimitedBlocks"], true);
        assert_eq!(flags["expire24hProfile"], true);
        assert_eq!(flags["hideDistance"], true);
        assert_eq!(flags["boosts"], true);
        assert_eq!(flags["profileViews"], true);
        assert_eq!(flags["tapAndGo"], true);
        // Server-side A/B flags should be preserved
        assert_eq!(flags["someFlag"], false);
    }

    #[test]
    fn test_bootstrap_handles_missing_feature_flags() {
        let input = serde_json::json!({"userRole": "FREE"});
        let (_, json) = call_rewrite(200, "/v3/bootstrap", input);
        assert!(json["featureFlags"].is_object());
        assert_eq!(json["featureFlags"]["readReceipts"], true);
    }

    // ── entitlements ───────────────────────────────────────────────────────

    #[test]
    fn test_entitlements_injects_rightnow() {
        let input = serde_json::json!({"rightNow": 5, "total": 5});
        let (_, json) = call_rewrite(200, "/v1/entitlements", input);
        assert_eq!(json["rightNow"], 999);
        assert_eq!(json["total"], 999);
    }

    #[test]
    fn test_entitlements_handles_missing_fields() {
        let input = serde_json::json!({});
        let (_, json) = call_rewrite(200, "/v1/entitlements", input);
        assert_eq!(json["rightNow"], 999);
        assert_eq!(json["total"], 999);
    }

    // ── profile / subscriptions ────────────────────────────────────────────

    #[test]
    fn test_profile_injects_premium_subscription() {
        let input = serde_json::json!({
            "subscription": {"premium": false, "userRole": "FREE"}
        });
        let (_, json) = call_rewrite(200, "/v3/me/profile", input);
        assert_premium_subscription(&json);
    }

    #[test]
    fn test_subscriptions_injects_premium_tier() {
        let input = serde_json::json!({"subscription": {"tier": "FREE"}});
        let (_, json) = call_rewrite(200, "/v4/subscriptions", input);
        assert_premium_subscription(&json);
    }

    #[test]
    fn test_v1_me_injects_premium_subscription() {
        let input = serde_json::json!({
            "profileId": "abc123",
            "subscription": {"premium": false, "userRole": "FREE"}
        });
        let (_, json) = call_rewrite(200, "/v1/me", input);
        assert_premium_subscription(&json);
        // Other fields preserved
        assert_eq!(json["profileId"], "abc123");
    }

    // ── ban bypass ─────────────────────────────────────────────────────────

    #[test]
    fn test_ban_bypass_status_field() {
        let input = serde_json::json!({"status": "banned"});
        let (status, json) = call_rewrite(403, "/v3/bootstrap", input);
        assert_eq!(status, 200);
        assert_eq!(json["status"], "ok");
    }

    #[test]
    fn test_ban_bypass_suspended() {
        let input = serde_json::json!({"status": "suspended"});
        let (status, json) = call_rewrite(403, "/v3/me/profile", input);
        assert_eq!(status, 200);
        assert_eq!(json["status"], "ok");
    }

    #[test]
    fn test_ban_bypass_restricted() {
        let input = serde_json::json!({"status": "restricted"});
        let (status, json) = call_rewrite(403, "/v1/entitlements", input);
        assert_eq!(status, 200);
        assert_eq!(json["status"], "ok");
    }

    #[test]
    fn test_ban_bypass_error_code() {
        let input = serde_json::json!({"code": 40301});
        let (status, json) = call_rewrite(403, "/v3/bootstrap", input);
        assert_eq!(status, 200);
        assert_eq!(json["status"], "ok");
    }

    #[test]
    fn test_ban_bypass_multiple_error_codes() {
        for code in [40300, 40301, 40302, 40303] {
            let input = serde_json::json!({"code": code});
            let (status, json) = call_rewrite(403, "/v3/bootstrap", input);
            assert_eq!(status, 200, "code {code} should trigger ban bypass");
            assert_eq!(json["status"], "ok");
        }
    }

    #[test]
    fn test_ban_bypass_ignores_ok_status() {
        let input = serde_json::json!({"status": "ok", "userRole": "FREE"});
        let (status, json) = call_rewrite(200, "/v3/bootstrap", input);
        // Should NOT trigger ban bypass — should apply bootstrap rewrite
        assert_eq!(status, 200);
        assert_eq!(json["userRole"], "UNLIMITED");
    }

    // ── inbox ──────────────────────────────────────────────────────────────

    #[test]
    fn test_inbox_removes_upgrade_required() {
        let input = serde_json::json!({"messages": [], "upgradeRequired": true});
        let (_, json) = call_rewrite(200, "/v2/inbox", input);
        assert!(json.get("upgradeRequired").is_none(), "upgradeRequired should be removed");
    }

    #[test]
    fn test_v3_inbox_removes_upgrade_required() {
        let input = serde_json::json!({"messages": [], "upgradeRequired": true});
        let (_, json) = call_rewrite(200, "/v3/inbox", input);
        assert!(json.get("upgradeRequired").is_none());
    }

    // ── settings ───────────────────────────────────────────────────────────

    #[test]
    fn test_settings_injects_premium_defaults() {
        let input = serde_json::json!({});
        let (_, json) = call_rewrite(200, "/v3/me/settings", input);
        assert_eq!(json["showDistance"], true);
        assert_eq!(json["incognito"], false);
    }

    #[test]
    fn test_settings_preserves_existing_values() {
        let input = serde_json::json!({"showDistance": false});
        let (_, json) = call_rewrite(200, "/v3/me/settings", input);
        // showDistance already exists, should keep its value via or_insert
        assert_eq!(json["showDistance"], false);
        // incognito should be added since it doesn't exist
        assert_eq!(json["incognito"], false);
    }

    // ── passthrough (non-matching paths, non-JSON responses) ───────────────

    #[test]
    fn test_non_matching_path_passes_through() {
        let input = serde_json::json!({"some": "data"});
        let body = serde_json::to_vec(&input).unwrap();
        let (status, new_body) = maybe_rewrite_response(200, "/v1/some/other/path", body);
        assert_eq!(status, 200);
        let json: serde_json::Value = serde_json::from_slice(&new_body).unwrap();
        assert_eq!(json, input);
    }

    #[test]
    fn test_non_json_body_passes_through() {
        let body = b"this is not json".to_vec();
        let (status, new_body) = maybe_rewrite_response(200, "/v3/bootstrap", body);
        assert_eq!(status, 200);
        assert_eq!(new_body, b"this is not json");
    }

    #[test]
    fn test_case_insensitive_path_matching() {
        let input = serde_json::json!({"userRole": "FREE"});
        let (_, json) = call_rewrite(200, "/V3/Bootstrap", input);
        assert_eq!(json["userRole"], "UNLIMITED");
    }

    // ── precedence: ban bypass beats other rewrites ────────────────────────

    #[test]
    fn test_ban_bypass_takes_precedence() {
        // Even on a known rewrite path, ban status should win
        let input = serde_json::json!({"status": "banned", "userRole": "FREE"});
        let (status, json) = call_rewrite(403, "/v3/bootstrap", input);
        assert_eq!(status, 200);
        assert_eq!(json["status"], "ok");
        assert_eq!(json.get("userRole"), None, "bootstrap rewrite should not apply after ban bypass");
    }
}