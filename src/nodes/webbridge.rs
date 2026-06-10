//! WebBridge client — sends commands to Chrome extension via shared state.
//!
//! Browser nodes use this module to send commands through the WebBridge
//! WebSocket relay. The actual WebSocket connection is managed by the
//! server-level webbridge module (src/webbridge/mod.rs).
//!
//! This module provides a convenience wrapper that browser nodes call.

use crate::error::{FlowError, FlowResult};
use crate::webbridge::WebBridgeState;

/// Send a command to the Chrome extension via the shared WebBridge state.
///
/// This is the function that browser nodes should call. It:
/// 1. Takes the shared WebBridgeState from AppState
/// 2. Sends the command via WebSocket to the connected Chrome extension
/// 3. Waits for the response (with timeout)
pub async fn send_browser_command(
    webbridge: &WebBridgeState,
    action: &str,
    params: serde_json::Value,
) -> FlowResult<serde_json::Value> {
    tracing::info!("WebBridge: sending command '{}'", action);

    webbridge
        .send_command(action, params, 30_000)
        .await
        .map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "webbridge".to_string(),
            detail: e,
        })
}
