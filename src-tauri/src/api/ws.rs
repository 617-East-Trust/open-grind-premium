use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Notify;
use tokio::time::{sleep, timeout, interval};
use wreq::websocket::{Message, WebSocket};

use crate::error::AppError;
use crate::state::AppState;

use super::headers::GrindrHeaders;

const WS_URL: &str = "wss://grindr.mobi/v1/ws";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(45);
/// Cap the WS handshake. Without this, a half-open connection on Android Doze
/// wedges the reconnect loop forever.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Max consecutive reconnect attempts before giving up.
const MAX_RECONNECT_ATTEMPTS: u32 = 50;

/// Outcome of `run_message_loop`.
/// Distinguishes a clean shutdown (command channel closed) from a
/// transient disconnect so the outer loop knows whether to reconnect or exit.
#[derive(Debug)]
enum WsOutcome {
    /// The command-sender side was dropped — no point reconnecting.
    ChannelClosed,
    /// Server sent a close frame or the connection dropped — should reconnect.
    Disconnected(AppError),
    /// Reconnect was explicitly requested (account switch / login).
    Reconnect,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WsCommand {
    pub r#type: String,
    pub ref_id: String,
    pub payload: Value,
}

impl WsCommand {
    /// Max buffer size before oldest messages are dropped.
    pub const BUFFER_CAPACITY: usize = 128;
}

/// Dedicated notify for triggering a WS reconnect on account switch.
/// Separated from `AppState.auth_notify` to avoid a race between the
/// outer listen loop and the per-connection reconnect watcher — both
/// waiting on the same `Notify` means `notify_one` only wakes one of them,
/// and the wrong one can win.
static WS_RECONNECT: std::sync::OnceLock<Arc<Notify>> = std::sync::OnceLock::new();

fn ws_reconnect_notify() -> &'static Arc<Notify> {
    WS_RECONNECT.get_or_init(|| Arc::new(Notify::new()))
}

/// Called by account-switch / remove / logout commands to force the WS
/// to reconnect with the new session.
pub fn request_ws_reconnect() {
    // notify_waiters() ensures both the outer loop and any per-connection
    // watcher receive the signal regardless of which is currently waiting.
    ws_reconnect_notify().notify_waiters();
}

pub fn spawn_ws_task(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        run_ws_loop(app).await;
    });
}

async fn run_ws_loop(app: AppHandle) {
    let state = app.state::<AppState>();
    let mut backoff = Duration::from_secs(1);
    let mut reconnect_count: u32 = 0;

    // Dedicated reconnect signal for the outer loop.
    let outer_reconnect = Arc::new(Notify::new());
    outer_reconnect.notify_one();

    loop {
        // Circuit-breaker: cap consecutive reconnects.
        if reconnect_count >= MAX_RECONNECT_ATTEMPTS {
            eprintln!(
                "[ws] circuit-breaker tripped after {MAX_RECONNECT_ATTEMPTS} consecutive reconnect attempts — giving up"
            );
            break;
        }

        outer_reconnect.notified().await;

        match connect_and_run(&app, &outer_reconnect).await {
            WsOutcome::ChannelClosed => {
                break;
            }
            WsOutcome::Reconnect => {
                backoff = Duration::from_secs(1);
                reconnect_count = 0;
                outer_reconnect.notify_one();
            }
            WsOutcome::Disconnected(e @ (AppError::NotInitialized | AppError::Auth(_))) => {
                eprintln!("[ws] auth error, waiting for login: {e}");
                app.emit("ws:disconnected", ()).ok();
                tokio::select! {
                    _ = state.auth_notify.notified() => {}
                    _ = ws_reconnect_notify().notified() => {}
                };
                outer_reconnect.notify_one();
                backoff = Duration::from_secs(1);
                reconnect_count = 0;
            }
            WsOutcome::Disconnected(e) => {
                eprintln!("[ws] error: {e}");
                app.emit("ws:disconnected", ()).ok();
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
                reconnect_count += 1;
                outer_reconnect.notify_one();
            }
        }
    }
}

async fn connect_and_run(
    app: &AppHandle,
    outer_reconnect: &Arc<Notify>,
) -> WsOutcome {
    let state = app.state::<AppState>();
    let client = match state.client() {
        Ok(c) => c,
        Err(e) => return WsOutcome::Disconnected(e),
    };

    let authorization = match client.authorization_header().await {
        Some(h) => h,
        None => return WsOutcome::Disconnected(AppError::Auth("Not logged in".to_owned())),
    };

    let session_id = match client.session.read().await.as_ref().map(|s| s.session_id.clone()) {
        Some(id) => id,
        None => return WsOutcome::Disconnected(AppError::Auth("Not logged in".to_owned())),
    };

    let our_profile_id = client
        .session
        .read()
        .await
        .as_ref()
        .map(|s| s.profile_id.clone())
        .unwrap_or_default();

    let fp = client.fingerprint().await;
    // Do not send L-Grindr-Roles. The official free client never claims
    // PREMIUM/UNLIMITED here; premium is injected client-side via response
    // rewriting only. Sending mismatched roles is a high-confidence ban signal.
    let headers = match GrindrHeaders::build(
        &fp.device,
        &fp.user_agent,
        Some(&authorization),
        None,
    ) {
        Ok(h) => h,
        Err(e) => return WsOutcome::Disconnected(e),
    };

    let mut builder = fp.ws_http.websocket(WS_URL);
    for (name, value) in &headers.items {
        builder = builder.header(name.clone(), value.clone());
    }

    let ws_future = builder.send();
    let response = match timeout(CONNECT_TIMEOUT, ws_future).await {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            return WsOutcome::Disconnected(AppError::Http(format!("WS connect failed: {e}")))
        }
        Err(_) => {
            return WsOutcome::Disconnected(AppError::Http(format!(
                "WS connect timed out after {}s",
                CONNECT_TIMEOUT.as_secs()
            )))
        }
    };

    let mut ws = match response.into_websocket().await {
        Ok(w) => w,
        Err(e) => return WsOutcome::Disconnected(AppError::Http(format!("WS upgrade failed: {e}"))),
    };

    app.emit("ws:connected", ()).ok();

    // Flush buffered outbound messages into the WS send channel.
    // Take ownership of the buffer first so we never re-lock `ws_buffer`
    // while already holding it (tokio Mutex is not re-entrant).
    {
        let state = app.state::<AppState>();
        let pending: Vec<WsCommand> = {
            let mut buf = state.ws_buffer.lock().await;
            std::mem::take(&mut *buf)
        };

        if !pending.is_empty() {
            eprintln!("[ws] flushing {} buffered messages", pending.len());
            let _ = app.emit(
                "ws:queue-draining",
                serde_json::json!({ "count": pending.len() }),
            );
        }

        let tx = &state.ws_tx;
        let mut overflow: Vec<WsCommand> = Vec::new();
        let mut sent = 0usize;

        for cmd in pending {
            if !overflow.is_empty() {
                overflow.push(cmd);
                continue;
            }
            match tx.try_send(cmd) {
                Ok(()) => sent += 1,
                Err(tokio::sync::mpsc::error::TrySendError::Full(cmd))
                | Err(tokio::sync::mpsc::error::TrySendError::Closed(cmd)) => {
                    eprintln!("[ws] flush: channel full/closed, re-queueing remainder");
                    overflow.push(cmd);
                }
            }
        }

        if !overflow.is_empty() {
            let requeued = overflow.len();
            let mut buf = state.ws_buffer.lock().await;
            for cmd in overflow {
                if buf.len() >= WsCommand::BUFFER_CAPACITY {
                    let dropped = buf.remove(0);
                    eprintln!(
                        "[ws] flush: buffer full — dropped oldest command: {}",
                        dropped.r#type
                    );
                }
                buf.push(cmd);
            }
            let _ = app.emit(
                "ws:queue-dropped",
                serde_json::json!({
                    "reason": "flush_channel_full",
                    "sent": sent,
                    "requeued": requeued
                }),
            );
        }
    }

    let mut cmd_rx = match state.ws_rx.lock().await.take() {
        Some(rx) => rx,
        None => return WsOutcome::Disconnected(AppError::Http("WS already running".to_owned())),
    };

    // Per-connection reconnect signal (inner loop exit trigger).
    let reconnect = Arc::new(Notify::new());

    let auth_notify = Arc::clone(&state.auth_notify);
    let ws_reconn = Arc::clone(ws_reconnect_notify());
    let outer_reconn = Arc::clone(outer_reconnect);
    let reconnect_watcher = Arc::clone(&reconnect);
    tauri::async_runtime::spawn(async move {
        tokio::select! {
            _ = auth_notify.notified() => {}
            _ = ws_reconn.notified() => {}
        };
        reconnect_watcher.notify_waiters();
        outer_reconn.notify_waiters();
    });

    let outcome = run_message_loop(&mut ws, &mut cmd_rx, &session_id, &our_profile_id, &reconnect, app).await;

    *state.ws_rx.lock().await = Some(cmd_rx);
    outcome
}

async fn run_message_loop(
    ws: &mut WebSocket,
    cmd_rx: &mut tokio::sync::mpsc::Receiver<WsCommand>,
    session_id: &str,
    our_profile_id: &str,
    reconnect: &Arc<Notify>,
    app: &AppHandle,
) -> WsOutcome {
    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
    heartbeat.tick().await; // consume the immediate first tick
    let mut waiting_for_pong = false;

    loop {
        tokio::select! {
            msg = ws.next() => match msg {
                Some(Ok(Message::Text(text))) => {
                    #[allow(clippy::collapsible_if)]
                    if let Ok(val) = serde_json::from_str::<Value>(text.as_str()) {
                        if let Some(event_type) = val["type"].as_str() {
                            let safe_type = event_type.replace('.', "_");
                            app.emit(&format!("grindr:{safe_type}"), &val).ok();

                            // Background push notifications — only when app is not in foreground
                            // and the event was sent by someone else.
                            if !app
                                .state::<AppState>()
                                .is_foreground
                                .load(Ordering::Relaxed)
                            {
                                let sender_is_self = match &val["payload"]["senderId"] {
                                    Value::String(s) => s.as_str() == our_profile_id,
                                    Value::Number(n) => n.to_string() == our_profile_id,
                                    _ => false,
                                };
                                if !sender_is_self {
                                    match event_type {
                                        "chat.v1.message_sent" => maybe_notify_message(app, &val),
                                        "tap.v1.tap_sent" => maybe_notify_tap(app, &val),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Ok(Message::Ping(data))) => {
                    if let Err(e) = ws.send(Message::Pong(data)).await {
                        return WsOutcome::Disconnected(AppError::Http(e.to_string()));
                    }
                }
                Some(Ok(Message::Pong(_))) => {
                    waiting_for_pong = false;
                }
                Some(Ok(Message::Close(_))) | None => {
                    app.emit("ws:disconnected", ()).ok();
                    return WsOutcome::Disconnected(AppError::Http(
                        "WS connection closed by server".to_owned(),
                    ));
                }
                Some(Err(e)) => {
                    app.emit("ws:disconnected", ()).ok();
                    return WsOutcome::Disconnected(AppError::Http(e.to_string()));
                }
                Some(Ok(_)) => {}
            },

            cmd = cmd_rx.recv() => match cmd {
                Some(cmd) => {
                    let json = serde_json::json!({
                        "type": cmd.r#type,
                        "ref": cmd.ref_id,
                        "token": session_id,
                        "payload": cmd.payload,
                    });
                    if let Err(e) = ws.send(Message::text(json.to_string())).await {
                        return WsOutcome::Disconnected(AppError::Http(e.to_string()));
                    }
                }
                None => return WsOutcome::ChannelClosed,
            },

            _ = reconnect.notified() => {
                eprintln!("[ws] reconnect requested, closing current connection");
                return WsOutcome::Reconnect;
            }

            _ = heartbeat.tick() => {
                if waiting_for_pong {
                    return WsOutcome::Disconnected(AppError::Http(
                        "WS heartbeat timeout — no pong received".to_owned(),
                    ));
                }
                if let Err(e) = ws.send(Message::Ping(vec![].into())).await {
                    return WsOutcome::Disconnected(AppError::Http(e.to_string()));
                }
                waiting_for_pong = true;
            }
        }
    }
}

/// Build a short, human-readable preview for a `chat.v1.message_sent` payload.
fn message_preview(val: &Value) -> String {
    match val["payload"]["type"].as_str() {
        Some("Text") => val["payload"]["body"]["text"]
            .as_str()
            .unwrap_or("New message")
            .chars()
            .take(80)
            .collect::<String>(),
        Some("Image") | Some("ExpiringImage") => "Sent you a photo".to_owned(),
        Some("Album") | Some("ExpiringAlbum") | Some("ExpiringAlbumV2") => {
            "Shared an album".to_owned()
        }
        Some("Audio") => "Sent you a voice message".to_owned(),
        Some("Video") | Some("PrivateVideo") | Some("NonExpiringVideo") => {
            "Sent you a video".to_owned()
        }
        Some("Gaymoji") => "Sent you a Gaymoji".to_owned(),
        Some("Giphy") => "Sent you a GIF".to_owned(),
        Some("Location") => "Shared a location".to_owned(),
        _ => "New message".to_owned(),
    }
}

fn maybe_notify_message(app: &AppHandle, val: &Value) {
    let body = message_preview(val);
    let conversation_id = val["payload"]["conversationId"].as_str().unwrap_or("");
    post_notification(app, "Open Grind", &body, conversation_id);
}

fn maybe_notify_tap(app: &AppHandle, val: &Value) {
    let title = val["payload"]["senderDisplayName"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("Open Grind");
    post_notification(app, title, "sent you a tap", "");
}

fn post_notification(app: &AppHandle, title: &str, body: &str, conversation_id: &str) {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .channel_id("open_grind_messages")
        .show()
        .ok();
    if !conversation_id.is_empty() {
        app.emit(
            "notification:posted",
            serde_json::json!({ "conversationId": conversation_id }),
        )
        .ok();
    }
}

#[tauri::command]
pub async fn ws_connect(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let has_session = match state.client() {
        Ok(c) => c.session.read().await.is_some(),
        Err(_) => false,
    };

    if has_session {
        state.auth_notify.notify_one();
        ws_reconnect_notify().notify_one();
    }
    Ok(())
}

#[tauri::command]
pub async fn ws_send(
    state: tauri::State<'_, AppState>,
    command: WsCommand,
) -> Result<(), AppError> {
    let client = state.client()?;
    if client.session.read().await.is_none() {
        return Err(AppError::Auth("Not logged in".to_owned()));
    }

    // Try to send directly. If the WS is not connected, buffer the message.
    if state.ws_tx.try_send(command.clone()).is_err() {
        let mut buf = state.ws_buffer.lock().await;

        if buf.len() < WsCommand::BUFFER_CAPACITY {
            buf.push(command);
        } else {
            let dropped = buf.remove(0);
            buf.push(command);

            // Emit event to frontend for UI feedback (e.g., show retry button)
            eprintln!("[ws] buffer full — dropped oldest command: {}", dropped.r#type);
        }
    }
    Ok(())
}