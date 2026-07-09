use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

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
    /// Rotation circuit-breaker state.
    pub(super) last_rotation: AtomicI64,
    pub(super) consecutive_rotations: AtomicU32,
}

#[derive(Debug, Serialize)]
pub struct RotateResult {
    #[serde(rename = "user-agent")]
    pub user_agent: String,
    #[serde(rename = "l-device-info")]
    pub l_device_info: String,
}

pub(super) fn build_api_client() -> Result<Client, AppError> {
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
            last_rotation: AtomicI64::new(0),
            consecutive_rotations: AtomicU32::new(0),
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

    // ── Rotation circuit breaker ──────────────────────────────────────────

    /// Max consecutive fingerprint rotations in the circuit breaker window.
    const MAX_ROTATIONS_PER_WINDOW: u32 = 5;
    /// Circuit breaker cooldown window in seconds.
    const ROTATION_WINDOW_SECS: i64 = 600;

    /// Increment the rotation counter, resetting it if the last rotation was
    /// outside the circuit-breaker window (i.e. a fresh burst).
    pub fn increment_rotation_counter(&self) {
        let now = chrono::Utc::now().timestamp();
        let last = self.last_rotation.swap(now, Ordering::Relaxed);
        if now.saturating_sub(last) < Self::ROTATION_WINDOW_SECS {
            self.consecutive_rotations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.consecutive_rotations.store(1, Ordering::Relaxed);
        }
    }

    pub fn reset_rotation_counter(&self) {
        self.consecutive_rotations.store(0, Ordering::Relaxed);
    }

    /// Returns true when the circuit breaker trips: more than
    /// `MAX_ROTATIONS_PER_WINDOW` rotations have occurred within the last
    /// `ROTATION_WINDOW_SECS`. Callers should refuse further rotations until
    /// the window elapses (checked on next call).
    pub fn rotation_circuit_breaker_tripped(&self) -> bool {
        let count = self.consecutive_rotations.load(Ordering::Relaxed);
        if count < Self::MAX_ROTATIONS_PER_WINDOW {
            return false;
        }
        let now = chrono::Utc::now().timestamp();
        let last = self.last_rotation.load(Ordering::Relaxed);
        if now.saturating_sub(last) >= Self::ROTATION_WINDOW_SECS {
            self.consecutive_rotations.store(0, Ordering::Relaxed);
            return false;
        }
        true
    }
}

/// Debug helper: short fingerprint hash of the current device identity.
#[tauri::command]
pub async fn device_fingerprint_hash(
    state: tauri::State<'_, AppState>,
) -> Result<String, AppError> {
    let fp = state.client()?.fingerprint().await;
    Ok(fp.device.fingerprint_hash())
}

#[tauri::command]
pub async fn rotate_api_params(
    state: tauri::State<'_, AppState>,
) -> Result<RotateResult, AppError> {
    let client = state.client()?;

    // Circuit breaker: refuse rapid manual rotations.
    if client.rotation_circuit_breaker_tripped() {
        return Err(AppError::Http(
            "Fingerprint rotation rate-limited — wait before rotating again".to_owned(),
        ));
    }

    let device = DeviceInfo::default();
    if let Err(e) = DeviceStorage::save(&device) {
        eprintln!("[client] could not persist rotated device info: {e}");
    }
    let user_agent = build_user_agent(&device, "Unlimited");

    // Reuse existing HTTP/WS clients to avoid expensive TLS handshake teardown.
    // Only the device identity (and headers built from it) needs to change.
    let guard = client.fingerprint.read().await;
    let new_fp = Arc::new(Fingerprint {
        http: guard.http.clone(),
        ws_http: guard.ws_http.clone(),
        device,
        user_agent,
    });
    drop(guard);

    // Capture new values before acquiring the write lock.
    let new_ua = new_fp.user_agent.clone();
    let new_device_info = super::headers::build_device_info_header(&new_fp.device);

    client.increment_rotation_counter();
    {
        let mut guard = client.fingerprint.write().await;
        *guard = new_fp;
    }

    Ok(RotateResult {
        user_agent: new_ua,
        l_device_info: new_device_info,
    })
}

/// Used by `ci/fingerprint_check.rs`
pub fn probe_emulation() -> wreq_util::Emulation {
    grindr_emulation()
}