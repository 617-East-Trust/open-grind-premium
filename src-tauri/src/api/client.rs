use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{Mutex, RwLock};
use wreq::{
    Client,
};

use crate::error::AppError;
use crate::state::AppState;

use super::auth::Session;
use super::headers::{build_user_agent, DeviceInfo, DeviceStorage};

pub const BASE_URL: &str = "https://grindr.mobi";

/// Use wreq-util's OkHttp4_12 emulation profile which provides a
/// real Android OkHttp TLS/JA3/JA4 fingerprint that matches what
/// Cloudflare expects from the official Grindr Android app.
fn grindr_emulation() -> wreq_util::Emulation {
    wreq_util::Emulation::OkHttp4_12
}

pub struct Fingerprint {
    /// ALPN: `["h2", "http/1.1"]`
    pub http: Client,
    /// ALPN: `["http/1.1"]`
    pub ws_http: Client,
    pub device: DeviceInfo,
    pub user_agent: String,
}

pub struct GrindrClient {
    pub(super) fingerprint: RwLock<Arc<Fingerprint>>,
    pub(super) session: RwLock<Option<Session>>,
    pub(super) refresh_lock: Mutex<()>,
}

#[derive(Debug, Serialize)]
pub struct RotateResult {
    #[serde(rename = "user-agent")]
    pub user_agent: String,
    #[serde(rename = "l-device-info")]
    pub l_device_info: String,
}

fn build_api_client() -> Result<Client, AppError> {
    Client::builder()
        .emulation(grindr_emulation())
        .gzip(true)
        .no_deflate()
        .no_brotli()
        .no_zstd()
        .build()
        .map_err(Into::into)
}

pub(super) fn build_ws_client() -> Result<Client, AppError> {
    Client::builder()
        .emulation(grindr_emulation())
        .gzip(true)
        .no_deflate()
        .no_brotli()
        .no_zstd()
        .http1_only()
        .build()
        .map_err(Into::into)
}

impl GrindrClient {
    pub fn new() -> Result<Self, AppError> {
        let device = match DeviceStorage::load() {
            Ok(Some(d)) => d,
            Ok(None) => {
                let d = DeviceInfo::default();
                if let Err(e) = DeviceStorage::save(&d) {
                    eprintln!("[client] could not persist device info: {e}");
                }
                d
            }
            Err(e) => {
                eprintln!("[client] could not load device info, regenerating: {e}");
                DeviceInfo::default()
            }
        };
        let user_agent = build_user_agent(&device, "Unlimited");

        let http = build_api_client()?;
        let ws_http = build_ws_client()?;

        #[cfg(all(target_os = "macos", not(feature = "keychain")))]
        let session = None;
        #[cfg(not(all(target_os = "macos", not(feature = "keychain"))))]
        let session = match super::auth::AuthStorage::get_active_session() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[client] could not load active session: {e}");
                None
            }
        };

        Ok(Self {
            fingerprint: RwLock::new(Arc::new(Fingerprint {
                http,
                ws_http,
                device,
                user_agent,
            })),
            session: RwLock::new(session),
            refresh_lock: Mutex::new(()),
        })
    }

    pub async fn fingerprint(&self) -> Arc<Fingerprint> {
        Arc::clone(&*self.fingerprint.read().await)
    }

    #[allow(dead_code)]
    pub async fn reload_session(&self) {
        match super::auth::AuthStorage::get_active_session() {
            Ok(s) => *self.session.write().await = s,
            Err(e) => eprintln!("[client] reload_session: {e}"),
        }
    }

    pub async fn set_active_session(&self, session: Session) {
        *self.session.write().await = Some(session);
    }

    pub async fn clear_session(&self) {
        *self.session.write().await = None;
    }
}

#[tauri::command]
pub async fn rotate_api_params(
    state: tauri::State<'_, AppState>,
) -> Result<RotateResult, AppError> {
    let client = state.client()?;

    let device = DeviceInfo::default();
    if let Err(e) = DeviceStorage::save(&device) {
        eprintln!("[client] could not persist rotated device info: {e}");
    }
    let user_agent = build_user_agent(&device, "Unlimited");
    let http = build_api_client()?;
    let ws_http = build_ws_client()?;

    let new_fp = Arc::new(Fingerprint {
        http,
        ws_http,
        device,
        user_agent,
    });

    // Capture new values before moving into the lock.
    let new_ua = new_fp.user_agent.clone();
    let new_device_info = super::headers::build_device_info_header(&new_fp.device);

    {
        let mut guard = client.fingerprint.write().await;
        *guard = new_fp;
    }

    // Return the *new* fingerprint values so the caller can verify the rotation.
    Ok(RotateResult {
        user_agent: new_ua,
        l_device_info: new_device_info,
    })
}

/// Used by `ci/fingerprint_check.rs`
pub fn probe_emulation() -> wreq_util::Emulation {
    grindr_emulation()
}
