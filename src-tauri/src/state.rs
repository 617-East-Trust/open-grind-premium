use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use tokio::sync::{mpsc, Mutex, Notify};

use crate::api::client::GrindrClient;
use crate::api::ws::WsCommand;
use crate::error::AppError;

pub struct AppState {
    pub client: OnceLock<Arc<GrindrClient>>,
    pub ws_tx: mpsc::Sender<WsCommand>,
    pub ws_rx: Mutex<Option<mpsc::Receiver<WsCommand>>>,
    pub auth_notify: Arc<Notify>,
    /// Buffered outbound WS commands held during disconnect. Flushed on reconnect.
    pub ws_buffer: Mutex<Vec<WsCommand>>,
    /// true when the WebView is visible/active; false when app is backgrounded.
    /// Used by the WS loop to decide whether to post system notifications.
    pub is_foreground: AtomicBool,
}

impl AppState {
    /// Get the API client, lazily initializing it if setup failed to create one.
    /// This ensures that even if BoringSSL/TLS init failed during app startup
    /// (e.g., due to Android system library timing), the client can be created
    /// later when the user actually tries to log in.
    pub fn client(&self) -> Result<Arc<GrindrClient>, AppError> {
        // Fast path: client already initialized
        if let Some(client) = self.client.get() {
            return Ok(client.clone());
        }

        // Slow path: try to initialize now (lazy recovery from startup failure)
        tracing::info!("attempting lazy GrindrClient initialization");
        match GrindrClient::new() {
            Ok(client) => {
                let arc = Arc::new(client);
                // OnceLock::set may fail if another thread raced us — that's fine,
                // just return whichever instance won.
                let _ = self.client.set(arc.clone());
                tracing::info!("lazy GrindrClient initialization succeeded");
                Ok(self.client.get().cloned().unwrap_or(arc))
            }
            Err(e) => {
                tracing::error!(error = %e, "lazy GrindrClient initialization failed");
                Err(AppError::NotInitialized)
            }
        }
    }
}
