use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wreq::header::{HeaderName, HeaderValue};
use wreq::{Method, RequestBuilder};

use crate::error::AppError;
use crate::state::AppState;

use super::client::GrindrClient;
use super::client::BASE_URL;
use super::headers::GrindrHeaders;

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
        let device = self.device.read().await.clone();
        let user_agent = self.user_agent.read().await.clone();
        let headers = GrindrHeaders::build(&device, &user_agent, None, None)?;

        let http = self.http.read().await.clone();
        let mut request = apply_headers(
            http.request(method, format!("{BASE_URL}{path}")),
            &headers.items,
        );

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let json: serde_json::Value = response.json().await.unwrap_or_default();
            return Err(AppError::Api {
                code: json.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32,
                message: json
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_owned(),
            });
        }

        response.json::<TResp>().await.map_err(Into::into)
    }

    async fn request_raw(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<RawResponse, AppError> {
        let authorization = self
            .authorization_header()
            .await
            .ok_or_else(|| AppError::Auth("Not logged in".to_owned()))?;

        let device = self.device.read().await.clone();
        let user_agent = self.user_agent.read().await.clone();

        let headers =
            GrindrHeaders::build(&device, &user_agent, Some(&authorization), Some("[FREE]"))?;

        #[cfg(debug_assertions)]
        {
            println!("=== OUTGOING REQUEST ===");
            println!("Method+Path: {method} {path}");
            for (name, value) in &headers.items {
                println!("  {name}: {}", value.to_str().unwrap_or("<binary>"));
            }
            println!("========================");
        }

        let http = self.http.read().await.clone();
        let mut request = apply_headers(
            http.request(method, format!("{BASE_URL}{path}")),
            &headers.items,
        );

        if let Some(body) = body {
            let json_body: serde_json::Value = rmp_serde::from_slice(&body)
                .map_err(|e| AppError::Http(format!("Failed to decode msgpack body: {e}")))?;
            request = request.json(&json_body);
        }

        let response = request.send().await?;
        let status = response.status().as_u16();
        let body = response.bytes().await?.to_vec();

        Ok(RawResponse { status, body })
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

    let raw = state
        .client()?
        .request_raw(method, &payload.path, payload.body)
        .await?;

    let response_bytes =
        rmp_serde::encode::to_vec_named(&raw).map_err(|e| AppError::Http(e.to_string()))?;

    Ok(STANDARD.encode(&response_bytes))
}
