//! WebBridge server — WebSocket relay between browser nodes and Chrome extension.
//!
//! Architecture:
//!   Chrome Extension → WebSocket → /ws/browser (this module stores the sender)
//!   Browser Node → HTTP POST → /api/browser/command → WebSocket → Extension
//!   Extension processes command → WebSocket response → HTTP response back to node
//!
//! Protocol:
//!   Server → Extension: {"id": "cmd-xxx", "action": "navigate", "params": {...}}
//!   Extension → Server: {"id": "cmd-xxx", "success": true, "data": {...}}
//!                        {"id": "cmd-xxx", "success": false, "error": "..."}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};

/// A pending command waiting for a response from the Chrome extension.
struct PendingCommand {
    respond_to: oneshot::Sender<Result<serde_json::Value, String>>,
}

/// Shared WebBridge state — holds the WebSocket sender and pending commands.
#[derive(Clone)]
pub struct WebBridgeState {
    /// The WebSocket sender to the connected Chrome extension.
    /// None if no extension is connected.
    sender: Arc<Mutex<Option<futures_util::stream::SplitSink<WebSocket, Message>>>>,
    /// Pending commands waiting for responses, keyed by command ID.
    pending: Arc<Mutex<HashMap<String, PendingCommand>>>,
}

impl Default for WebBridgeState {
    fn default() -> Self {
        Self::new()
    }
}

impl WebBridgeState {
    pub fn new() -> Self {
        Self {
            sender: Arc::new(Mutex::new(None)),
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a Chrome extension is connected.
    pub async fn is_connected(&self) -> bool {
        self.sender.lock().await.is_some()
    }

    /// Handle a new WebSocket connection from the Chrome extension.
    pub async fn handle_connection(&self, socket: WebSocket) {
        let (ws_sender, mut ws_receiver) = socket.split();

        // Store the sender
        {
            let mut sender = self.sender.lock().await;
            *sender = Some(ws_sender);
        }
        tracing::info!("WebBridge: Chrome extension connected");

        // Process incoming messages (command responses)
        loop {
            match ws_receiver.next().await {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        // Handle registration
                        if msg["type"].as_str() == Some("register") {
                            tracing::info!(
                                "WebBridge: extension registered (v{}, capabilities: {:?})",
                                msg["version"].as_str().unwrap_or("?"),
                                msg["capabilities"]
                            );
                            continue;
                        }

                        // Handle command response
                        if let Some(id) = msg["id"].as_str() {
                            let mut pending = self.pending.lock().await;
                            if let Some(cmd) = pending.remove(id) {
                                let result = if msg["success"].as_bool().unwrap_or(false) {
                                    Ok(msg["data"].clone())
                                } else {
                                    Err(msg["error"]
                                        .as_str()
                                        .unwrap_or("unknown error")
                                        .to_string())
                                };
                                let _ = cmd.respond_to.send(result);
                            }
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(e)) => {
                    tracing::warn!("WebBridge: WebSocket error: {}", e);
                    break;
                }
                _ => {} // Ping/Pong/Binary — ignore
            }
        }

        // Extension disconnected
        let mut sender = self.sender.lock().await;
        *sender = None;
        tracing::warn!("WebBridge: Chrome extension disconnected");

        // Fail all pending commands
        let mut pending = self.pending.lock().await;
        for (_, cmd) in pending.drain() {
            let _ = cmd.respond_to.send(Err("extension disconnected".to_string()));
        }
    }

    /// Send a command to the Chrome extension and wait for response.
    pub async fn send_command(
        &self,
        action: &str,
        params: serde_json::Value,
        timeout_ms: u64,
    ) -> Result<serde_json::Value, String> {
        let cmd_id = format!("cmd-{}", uuid::Uuid::new_v4());
        let command = serde_json::json!({
            "id": cmd_id,
            "action": action,
            "params": params
        });

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(cmd_id.clone(), PendingCommand { respond_to: tx });
        }

        // Send the command via WebSocket
        let msg_text = serde_json::to_string(&command)
            .map_err(|e| format!("failed to serialize command: {}", e))?;
        {
            let mut sender_guard = self.sender.lock().await;
            let sender = sender_guard
                .as_mut()
                .ok_or_else(|| "Chrome extension not connected. Please install and connect the WebBridge extension.".to_string())?;

            sender
                .send(Message::Text(msg_text))
                .await
                .map_err(|e| format!("failed to send command: {}", e))?;
        }
        // sender_guard dropped here, releasing the lock

        // Wait for response with timeout
        match tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            rx,
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err("response channel closed".to_string()),
            Err(_) => {
                // Timeout — remove from pending
                let mut pending = self.pending.lock().await;
                pending.remove(&cmd_id);
                Err(format!(
                    "command '{}' timed out after {}ms",
                    action, timeout_ms
                ))
            }
        }
    }
}

/// WebSocket upgrade handler for /ws/browser
pub async fn ws_handler(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl axum::response::IntoResponse {
    let bridge = state.webbridge.clone();
    ws.on_upgrade(move |socket| async move { bridge.handle_connection(socket).await; })
}

/// HTTP handler for POST /api/browser/command
/// Relays commands from browser nodes to the Chrome extension.
pub async fn browser_command(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    axum::Json(command): axum::Json<serde_json::Value>,
) -> Result<axum::Json<serde_json::Value>, (axum::http::StatusCode, axum::Json<serde_json::Value>)> {
    let action = command["action"]
        .as_str()
        .ok_or_else(|| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"error": "missing action field"})),
            )
        })?;

    let params = command["params"].clone();
    let timeout_ms = command["timeout_ms"].as_u64().unwrap_or(30_000);

    tracing::info!("WebBridge: relaying command '{}' to extension", action);

    match state.webbridge.send_command(action, params, timeout_ms).await {
        Ok(data) => Ok(axum::Json(serde_json::json!({
            "success": true,
            "data": data
        }))),
        Err(e) => Ok(axum::Json(serde_json::json!({
            "success": false,
            "error": e
        }))),
    }
}
