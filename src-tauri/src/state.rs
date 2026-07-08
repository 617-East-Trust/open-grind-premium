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
    /// P1 #6: Ring buffer for outbound WS messages queued while the WS is
    /// reconnecting. Flushed into `ws_tx` once `ws:connected` fires.
    pub ws_buffer: Mutex<Vec<WsCommand>>,
}

impl AppState {
    pub fn client(&self) -> Result<Arc<GrindrClient>, AppError> {
        self.client.get().cloned().ok_or(AppError::NotInitialized)
    }
}
