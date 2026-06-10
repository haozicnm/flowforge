//! WebBridge client — WebSocket connection to Chrome extension.
//!
//! Architecture:
//!   FlowForge backend → WebSocket server at /ws/browser
//!   Chrome extension connects TO this server
//!   Nodes send commands via this module, which routes to the extension
//!
//! Protocol (same as workflow-engine WebBridge):
//!   Server → Extension: {"id": "cmd-xxx", "action": "click", "params": {...}}
//!   Extension → Server: {"id": "cmd-xxx", "success": true, "data": {...}}
//!                       {"id": "cmd-xxx", "success": false, "error": "..."}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};


/// A pending command waiting for a response from the Chrome extension.
struct PendingCommand {
    respond_to: oneshot::Sender<Result<serde_json::Value, String>>,
}

/// Manages the WebSocket connection to the Chrome extension.
///
/// In a real deployment, this would be embedded in AppState and shared
/// across all browser nodes. For now, each node creates a command and
/// sends it through this client.
pub struct WebBridgeClient {
    /// Pending commands waiting for responses
    pending: Arc<Mutex<HashMap<String, PendingCommand>>>,
    /// URL of the WebSocket endpoint (default: ws://127.0.0.1:19529/ws/browser)
    ws_url: String,
}

impl Default for WebBridgeClient {
    fn default() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            ws_url: "ws://127.0.0.1:19529/ws/browser".to_string(),
        }
    }
}

impl WebBridgeClient {
    /// Send a command to the Chrome extension and wait for response.
    ///
    /// This is a synchronous simulation — in production, this would go through
    /// a real WebSocket connection. For now, we simulate by making an HTTP
    /// request to a companion API or return a mock.
    pub async fn send_command(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let cmd_id = format!("cmd-{}", uuid::Uuid::new_v4());
        let command = serde_json::json!({
            "id": cmd_id,
            "action": action,
            "params": params
        });

        tracing::info!("WebBridge: sending command {} -> {}", cmd_id, action);

        // In production, this would send via WebSocket to the Chrome extension.
        // For now, we use HTTP relay if available, or return a structured error.
        let client = reqwest::Client::new();
        let relay_url = "http://127.0.0.1:19529/api/browser/command";

        match client.post(relay_url)
            .json(&command)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
                    if body["success"].as_bool().unwrap_or(false) {
                        Ok(body.get("data").cloned().unwrap_or(serde_json::json!({})))
                    } else {
                        Err(body["error"].as_str().unwrap_or("unknown error").to_string())
                    }
                } else {
                    Err(format!("WebBridge HTTP error: {}", resp.status()))
                }
            }
            Err(e) => {
                Err(format!("WebBridge connection failed: {}. Is the Chrome extension connected?", e))
            }
        }
    }

    /// Set a custom WebSocket URL (for configuration).
    pub fn set_ws_url(&mut self, url: &str) {
        self.ws_url = url.to_string();
    }
}
