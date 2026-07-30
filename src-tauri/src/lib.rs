// `api` is `pub` so that `ci/fingerprint_check.rs` can reuse same header / client builders
#![allow(clippy::needless_return)]
pub mod api;
mod error;
mod log_init;
mod state;
mod storage;

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use tauri::Manager;
use tokio::sync::{mpsc, Mutex, Notify};

use crate::state::AppState;
use api::client::GrindrClient;

/// Called by the frontend when the app enters foreground or background.
/// Gates push notifications and drives Doze-aware WS reconnect on resume.
#[tauri::command]
fn set_foreground(state: tauri::State<'_, AppState>, foreground: bool) {
    let was_foreground = state
        .is_foreground
        .swap(foreground, std::sync::atomic::Ordering::Relaxed);
    tracing::debug!(foreground, was_foreground, "app foreground state changed");

    // Leaving Doze / returning to the app: force a WS reconnect so half-open
    // sockets from deep sleep are not left hanging until the next heartbeat miss.
    if foreground && !was_foreground {
        tracing::info!("app resumed — requesting WebSocket reconnect");
        api::ws::request_ws_reconnect();
        state.auth_notify.notify_waiters();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_init::init();

    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init();

    let (ws_tx, ws_rx) = mpsc::channel(64);
    let auth_notify = Arc::new(Notify::new());

    #[allow(unused_mut)] // mut only needed when debug_assertions injects devtools
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_geolocation::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            client: OnceLock::new(),
            ws_tx,
            ws_rx: Mutex::new(Some(ws_rx)),
            auth_notify,
            ws_buffer: Mutex::new(Vec::new()),
            is_foreground: AtomicBool::new(true),
        })
        .invoke_handler(tauri::generate_handler![
            api::auth::login,
            api::auth::add_account,
            api::auth::switch_account,
            api::auth::remove_account,
            api::auth::list_accounts,
            api::auth::refresh_token,
            api::auth::logout,
            api::auth::auth_state,
            api::rest::request,
            api::ws::ws_connect,
            api::ws::ws_send,
            api::ws::ws_status,
            api::client::rotate_api_params,
            api::client::device_fingerprint_hash,
            api::rest::upload_image,
            api::rest::fetch_authed_bytes,
            api::version::grindr_app_version,
            set_foreground,
        ])
        .setup(|app| {
            // ── Phase 1: Safe initialization (never panics) ──────────────
            // These MUST run before anything else and cannot be skipped.
            let app_data = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    tracing::error!(error = %e, "failed to get app data directory — using fallback");
                    std::path::PathBuf::from("/data/data/com.opengrind.premium/files")
                }
            };

            // Initialize credential store (file-backed fallback, never panics).
            storage::init_keyring(app_data);

            // Seed Grindr API version from keyring cache (never panics).
            api::version::load_cached();

            // ── Phase 2: Client + background tasks (can panic) ───────────
            // GrindrClient::new() can panic inside BoringSSL/wreq on some
            // Android devices. Wrap in catch_unwind so the app still opens.
            // The lazy init in state.rs will retry on first API command.
            let setup_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                setup_client_and_tasks(app)
            }));

            match setup_result {
                Ok(()) => {
                    tracing::info!("app setup completed successfully");
                }
                Err(panic_info) => {
                    let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };
                    tracing::error!(
                        panic = %msg,
                        "client/task setup panicked — recovered; client will lazy-init on first use"
                    );
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                // Prevent exit on back press when running as a mobile app; let the
                // OS manage the lifecycle. The app will be backgrounded instead.
                api.prevent_exit();
            }
        });
}

/// Phase 2 of setup: create the API client and spawn background tasks.
/// This function may panic (BoringSSL); caller wraps in catch_unwind.
/// If it panics, keyring is already initialized so lazy client init works.
fn setup_client_and_tasks(app: &mut tauri::App) {
    // Create the API client.
    match GrindrClient::new() {
        Ok(client) => {
            let _ = app.state::<AppState>().client.set(Arc::new(client));
            tracing::info!("GrindrClient initialized successfully");
        }
        Err(e) => {
            tracing::error!(error = %e, "GrindrClient creation failed — will retry on first use");
            return; // Skip background tasks that depend on the client
        }
    }

    // Background: refresh Grindr app version and rebuild UA if it changed.
    {
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let info = api::version::refresh_if_stale().await;
            if let Ok(client) = handle.state::<AppState>().client() {
                client.apply_app_version(&info).await;
            }
        });
    }

    // Periodic WS health while backgrounded (Doze-friendly soft keep-alive).
    api::ws::spawn_background_health_task(app.handle().clone());

    // Reload session after keyring is ready (covers file-store path too).
    {
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let state = handle.state::<AppState>();
            if let Ok(client) = state.client() {
                client.clone().reload_session().await;
                if client.clone().authorization_header().await.is_some() {
                    state.auth_notify.notify_one();
                }
            }
        });
    }

    api::ws::spawn_ws_task(app.handle().clone());
}
