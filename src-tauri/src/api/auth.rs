use keyring_core::Entry;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

use super::client::GrindrClient;
use super::ws;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub email: String,
    pub expires_at: u64,
    pub profile_id: String,
    pub session_id: String,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub profile_id: String,
    pub session_id: String,
    pub auth_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub token: Option<String>,
    pub geohash: Option<String>,
}

trait AuthRequest: Serialize {
    fn email(&self) -> &str;
}

impl AuthRequest for LoginRequest {
    fn email(&self) -> &str {
        &self.email
    }
}

impl AuthRequest for RefreshRequest {
    fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub email: String,
    pub auth_token: String,
    pub token: Option<String>,
    pub geohash: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    pub profile_id: String,
}

/// Minimal account info for the account switcher UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountEntry {
    pub profile_id: String,
    pub email: String,
    /// Whether this is the currently active session.
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    exp: u64,
}

impl LoginRequest {
    pub fn new(email: String, password: String) -> Self {
        Self {
            email,
            password,
            token: None,
            geohash: None,
        }
    }
}

impl RefreshRequest {
    pub fn new(email: String, auth_token: String) -> Self {
        Self {
            email,
            auth_token,
            token: None,
            geohash: None,
        }
    }
}

fn decode_session_jwt(token: &str) -> Result<JwtClaims, AppError> {
    let data = jsonwebtoken::dangerous::insecure_decode::<JwtClaims>(token)
        .map_err(|e| AppError::Auth(format!("JWT decode failed: {e}")))?;

    Ok(data.claims)
}

// ── Keyring helpers ──────────────────────────────────────────────────────

const SESSION_KEYRING_SERVICE: &str = "open-grind";
const ACTIVE_KEY: &str = "active-profile-id";
const ACCOUNTS_INDEX_KEY: &str = "accounts-index";

fn keyring_entry(key: &str) -> Result<Entry, AppError> {
    Entry::new(SESSION_KEYRING_SERVICE, key).map_err(|e| AppError::Auth(e.to_string()))
}

fn session_key_for(profile_id: &str) -> String {
    format!("session-{profile_id}")
}

// ── AuthStorage — multi-account session persistence ─────────────────────

pub struct AuthStorage;

impl AuthStorage {
    // ── session CRUD ───────────────────────────────────────────────────

    pub fn get_session(profile_id: &str) -> Result<Option<Session>, AppError> {
        let entry = keyring_entry(&session_key_for(profile_id))?;
        let bytes = match entry.get_secret() {
            Ok(b) => b,
            Err(keyring_core::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(AppError::Auth(e.to_string())),
        };
        rmp_serde::decode::from_slice(&bytes)
            .map_err(|e| AppError::Auth(e.to_string()))
            .map(Some)
    }

    pub fn set_session(session: &Session) -> Result<(), AppError> {
        let bytes = rmp_serde::encode::to_vec(session)
            .map_err(|e| AppError::Auth(format!("session encode failed: {e}")))?;
        keyring_entry(&session_key_for(&session.profile_id))?
            .set_secret(&bytes)
            .map_err(|e| AppError::Auth(e.to_string()))?;
        Self::upsert_account_index(&session.profile_id)?;
        Ok(())
    }

    pub fn delete_session(profile_id: &str) {
        if let Ok(entry) = keyring_entry(&session_key_for(profile_id)) {
            let _ = entry.delete_credential();
        }
        Self::remove_account_index(profile_id);
    }

    // ── accounts index (profile_id list for enumeration) ───────────────

    fn accounts_index() -> Vec<String> {
        let Ok(entry) = keyring_entry(ACCOUNTS_INDEX_KEY) else {
            return Vec::new();
        };
        let Ok(bytes) = entry.get_secret() else {
            return Vec::new();
        };
        rmp_serde::decode::from_slice(&bytes).unwrap_or_default()
    }

    #[allow(clippy::collapsible_if)]
    fn save_accounts_index(ids: &[String]) {
        if let Ok(bytes) = rmp_serde::encode::to_vec(ids) {
            if let Ok(entry) = keyring_entry(ACCOUNTS_INDEX_KEY) {
                let _ = entry.set_secret(&bytes);
            }
        }
    }

    fn upsert_account_index(profile_id: &str) -> Result<(), AppError> {
        let mut ids = Self::accounts_index();
        if !ids.contains(&profile_id.to_string()) {
            ids.push(profile_id.to_string());
        }
        Self::save_accounts_index(&ids);
        Ok(())
    }

    #[allow(clippy::collapsible_if)]
    fn remove_account_index(profile_id: &str) {
        let mut ids = Self::accounts_index();
        ids.retain(|id| id != profile_id);
        Self::save_accounts_index(&ids);
        // Clear active if it was this one
        if let Ok(active) = Self::get_active_profile_id() {
            if active.as_deref() == Some(profile_id) {
                if let Ok(entry) = keyring_entry(ACTIVE_KEY) {
                    let _ = entry.delete_credential();
                }
            }
        }
    }

    // ── active session tracking ────────────────────────────────────────

    pub fn set_active_profile(profile_id: &str) -> Result<(), AppError> {
        let bytes = profile_id.as_bytes();
        keyring_entry(ACTIVE_KEY)?
            .set_secret(bytes)
            .map_err(|e| AppError::Auth(e.to_string()))
    }

    pub fn get_active_profile_id() -> Result<Option<String>, AppError> {
        let entry = keyring_entry(ACTIVE_KEY)?;
        match entry.get_secret() {
            Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).into_owned())),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Auth(e.to_string())),
        }
    }

    // ── unified load: restores active session from keyring ─────────────

    pub fn get_active_session() -> Result<Option<Session>, AppError> {
        let Some(pid) = Self::get_active_profile_id()? else {
            return Ok(None);
        };
        Self::get_session(&pid)
    }

    // ── list all accounts for the switcher UI ───────────────────────────

    pub fn list_accounts(active_profile_id: Option<&str>) -> Vec<AccountEntry> {
        let ids = Self::accounts_index();
        let mut accounts = Vec::with_capacity(ids.len());
        for pid in &ids {
            if let Ok(Some(session)) = Self::get_session(pid) {
                accounts.push(AccountEntry {
                    profile_id: pid.clone(),
                    email: session.email,
                    is_active: active_profile_id == Some(pid),
                });
            }
        }
        accounts
    }
}

// ── GrindrClient extensions for multi-account ────────────────────────────

impl GrindrClient {
    pub async fn switch_to_session(&self, session: Session) -> Result<(), AppError> {
        AuthStorage::set_session(&session)?;
        AuthStorage::set_active_profile(&session.profile_id)?;
        *self.session.write().await = Some(session);
        Ok(())
    }

    pub async fn active_profile_id(&self) -> Option<String> {
        self.session
            .read()
            .await
            .as_ref()
            .map(|s| s.profile_id.clone())
    }

    async fn create_session(&self, body: &impl AuthRequest) -> Result<Session, AppError> {
        let session_resp: SessionResponse = self
            .request_json(wreq::Method::POST, "/v8/sessions", Some(body))
            .await?;
        let claims = decode_session_jwt(&session_resp.session_id)?;

        Ok(Session {
            email: body.email().to_owned(),
            profile_id: session_resp.profile_id.clone(),
            session_id: session_resp.session_id,
            auth_token: session_resp.auth_token,
            expires_at: claims.exp,
        })
    }

    /// Login and immediately activate this session.
    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResult, AppError> {
        let body = LoginRequest::new(email.to_owned(), password.to_owned());
        let session = self.create_session(&body).await?;
        let profile_id = session.profile_id.clone();
        self.switch_to_session(session).await?;
        Ok(LoginResult { profile_id })
    }

    /// Add a second account without switching to it.
    pub async fn add_account(&self, email: &str, password: &str) -> Result<LoginResult, AppError> {
        let body = LoginRequest::new(email.to_owned(), password.to_owned());
        let session = self.create_session(&body).await?;
        AuthStorage::set_session(&session)?;
        Ok(LoginResult {
            profile_id: session.profile_id,
        })
    }

    pub async fn refresh_token(&self) -> Result<LoginResult, AppError> {
        let _guard = self.refresh_lock.lock().await;

        // Re-check — another thread may have already refreshed while we waited
        // for the lock.
        {
            let current = self.session.read().await;
            if let Some(ref s) = *current {
                let still_expired = s.expires_at
                    < (chrono::Utc::now().timestamp() as u64 + 60);
                if !still_expired {
                    return Ok(LoginResult {
                        profile_id: s.profile_id.clone(),
                    });
                }
            }
        }

        let current = self.session.read().await;
        let session = current
            .as_ref()
            .ok_or_else(|| AppError::Auth("Not logged in".to_owned()))?;

        let body = RefreshRequest::new(session.email.clone(), session.auth_token.clone());

        drop(current);

        let session = self.create_session(&body).await?;
        let profile_id = session.profile_id.clone();
        *self.session.write().await = Some(session);

        Ok(LoginResult { profile_id })
    }

    pub async fn logout_current(&self) -> Result<Option<String>, AppError> {
        let removed = self.session.write().await.take();
        if let Some(ref s) = removed {
            AuthStorage::delete_session(&s.profile_id);
        }
        // Try to activate the next available account
        let accounts = AuthStorage::list_accounts(None);
        #[allow(clippy::collapsible_if)]
        if let Some(next) = accounts.first() {
            if let Ok(Some(session)) = AuthStorage::get_session(&next.profile_id) {
                AuthStorage::set_active_profile(&next.profile_id)?;
                *self.session.write().await = Some(session);
                return Ok(Some(next.profile_id.clone()));
            }
        }
        Ok(None)
    }

    pub async fn authorization_header(&self) -> Option<String> {
        // Do NOT hold refresh_lock across a call to refresh_token() —
        // refresh_token already acquires that mutex, and tokio::Mutex is
        // not re-entrant (would deadlock). Concurrent refresh races are
        // handled inside refresh_token via double-checked locking.
        let expires_at = self
            .session
            .read()
            .await
            .as_ref()
            .map(|s| s.expires_at)
            .unwrap_or(0);

        if expires_at < (chrono::Utc::now().timestamp() as u64 + 60) {
            let _ = self.refresh_token().await;
        }

        self.session
            .read()
            .await
            .as_ref()
            .map(|s| format!("Grindr3 {}", s.session_id))
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn login(
    state: tauri::State<'_, AppState>,
    email: String,
    password: String,
) -> Result<LoginResult, AppError> {
    let result = state.client()?.login(&email, &password).await?;
    state.auth_notify.notify_one();
    ws::request_ws_reconnect();
    Ok(result)
}

#[tauri::command]
pub async fn add_account(
    state: tauri::State<'_, AppState>,
    email: String,
    password: String,
) -> Result<LoginResult, AppError> {
    // Add account without switching active session
    state.client()?.add_account(&email, &password).await
}

#[tauri::command]
pub async fn switch_account(
    state: tauri::State<'_, AppState>,
    profile_id: String,
) -> Result<LoginResult, AppError> {
    let client = state.client()?;

    // If already active, no-op
    if client.active_profile_id().await.as_deref() == Some(&profile_id) {
        return Ok(LoginResult { profile_id });
    }

    let session = AuthStorage::get_session(&profile_id)?
        .ok_or_else(|| AppError::Auth(format!("No session for profile {profile_id}")))?;

    client.switch_to_session(session).await?;
    state.auth_notify.notify_one();
    ws::request_ws_reconnect();
    Ok(LoginResult { profile_id })
}

#[tauri::command]
pub async fn remove_account(
    state: tauri::State<'_, AppState>,
    profile_id: String,
) -> Result<serde_json::Value, AppError> {
    let client = state.client()?;

    // If removing the active account, logout first (which auto-switches)
    let was_active = client.active_profile_id().await.as_deref() == Some(&profile_id);

    if was_active {
        client.logout_current().await?;
        state.auth_notify.notify_one();
        ws::request_ws_reconnect();
    } else {
        AuthStorage::delete_session(&profile_id);
    }

    let accounts = AuthStorage::list_accounts(
        client.active_profile_id().await.as_deref(),
    );

    Ok(serde_json::json!({
        "removed": true,
        "was_active": was_active,
        "accounts": accounts,
    }))
}

#[tauri::command]
pub async fn list_accounts(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AccountEntry>, AppError> {
    Ok(AuthStorage::list_accounts(
        state.client()?.active_profile_id().await.as_deref(),
    ))
}

#[tauri::command]
pub async fn refresh_token(state: tauri::State<'_, AppState>) -> Result<LoginResult, AppError> {
    let result = state.client()?.refresh_token().await?;
    state.auth_notify.notify_one();
    Ok(result)
}

#[tauri::command]
pub async fn logout(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    state.client()?.logout_current().await?;
    state.auth_notify.notify_one();
    ws::request_ws_reconnect();
    Ok(())
}

#[tauri::command]
pub async fn auth_state(state: tauri::State<'_, AppState>) -> Result<Option<String>, AppError> {
    let Ok(client) = state.client() else {
        return Ok(None);
    };
    let session = client.session.read().await;
    Ok(session.as_ref().map(|s| s.profile_id.clone()))
}
