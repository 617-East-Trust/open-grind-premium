use reqwest::header::HeaderMap;
use reqwest::Client;
use serde::Serialize;
use tokio::sync::{Mutex, RwLock};

use crate::error::AppError;
use crate::state::AppState;

use super::auth::Session;
use super::headers::{build_default_headers, build_user_agent, DeviceInfo};

pub const BASE_URL: &str = "https://grindr.mobi";

pub struct GrindrClient {
    pub(super) http: RwLock<Client>,
    pub(super) default_headers: RwLock<HeaderMap>,
    pub(super) session: RwLock<Option<Session>>,
    pub(super) refresh_lock: Mutex<()>,
    pub user_agent: RwLock<String>,
}

#[derive(Debug, Serialize)]
pub struct RotateResult {
    #[serde(rename = "user-agent")]
    pub user_agent: String,
    #[serde(rename = "l-device-info")]
    pub l_device_info: String,
}

impl GrindrClient {
    pub fn new() -> Result<Self, AppError> {
        let device = DeviceInfo::default();
        let user_agent = build_user_agent(&device, "Free");
        let headers = build_default_headers(&device, &user_agent);

        let http = Client::builder().default_headers(headers.clone()).build()?;

        #[cfg(all(target_os = "macos", not(feature = "keychain")))]
        let session = None;
        #[cfg(not(all(target_os = "macos", not(feature = "keychain"))))]
        let session = match super::auth::AuthStorage::get_session() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[client] could not load session: {e}");
                None
            }
        };

        Ok(Self {
            http: RwLock::new(http),
            default_headers: RwLock::new(headers),
            session: RwLock::new(session),
            refresh_lock: Mutex::new(()),
            user_agent: RwLock::new(user_agent),
        })
    }

    #[allow(dead_code)]
    pub async fn reload_session(&self) {
        match super::auth::AuthStorage::get_session() {
            Ok(s) => *self.session.write().await = s,
            Err(e) => eprintln!("[client] reload_session: {e}"),
        }
    }
}

#[tauri::command]
pub async fn rotate_api_params(
    state: tauri::State<'_, AppState>,
) -> Result<RotateResult, AppError> {
    let client = state.client()?;
    let old_ua = client.user_agent.read().await.clone();
    let old_device_info = client
        .default_headers
        .read()
        .await
        .get("L-Device-Info")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();

    let device = DeviceInfo::default();
    let user_agent = build_user_agent(&device, "Free");
    let headers = build_default_headers(&device, &user_agent);
    let http = Client::builder().default_headers(headers.clone()).build()?;

    *client.http.write().await = http;
    *client.default_headers.write().await = headers;
    *client.user_agent.write().await = user_agent;

    Ok(RotateResult {
        user_agent: old_ua,
        l_device_info: old_device_info,
    })
}
