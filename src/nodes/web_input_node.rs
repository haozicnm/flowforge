//! Web Input node — type text into an input field via WebBridge.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct WebInputNode;

#[async_trait]
impl NodeExecutor for WebInputNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "web_input".to_string(),
            display_name: "输入文本".to_string(),
            description: "在页面输入框中输入文本".to_string(),
            category: "网页自动化".to_string(),
            inputs: vec![
                PortDef { label: "selector".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "text".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "out".to_string(), data_type: "object".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "selector": { "type": "string", "description": "CSS selector for the input field" },
                    "text": { "type": "string", "description": "Text to type" },
                    "clear_first": { "type": "boolean", "default": true },
                    "press_enter": { "type": "boolean", "default": false },
                    "delay_ms": { "type": "number", "default": 0, "description": "Delay between keystrokes" }
                },
                "required": ["selector"]
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let selector = config["selector"].as_str()
            .or_else(|| inputs.get("selector").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "web_input".to_string(),
                detail: "selector is required".to_string(),
            })?;

        let text = config["text"].as_str()
            .or_else(|| inputs.get("text").and_then(|v| v.as_str()))
            .unwrap_or("");

        let clear_first = config["clear_first"].as_bool().unwrap_or(true);
        let press_enter = config["press_enter"].as_bool().unwrap_or(false);

        let wb = ctx.webbridge.as_ref().ok_or_else(|| FlowError::NodeExecutionFailed {
            node_id: "web".to_string(),
            detail: "WebBridge not configured. Browser automation requires a connected Chrome extension.".to_string(),
        })?;

        // Clear field first if requested
        if clear_first {
            let _ = super::webbridge::send_browser_command(wb, "fill", serde_json::json!({
                "selector": selector, "content": ""
            })).await;
        }

        // Type the text
        let data = super::webbridge::send_browser_command(wb, "fill", serde_json::json!({
            "selector": selector,
            "content": text
        })).await.map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "web_input".to_string(),
            detail: e.to_string(),
                       })?;

        // Press Enter if requested
        if press_enter {
            let _ = super::webbridge::send_browser_command(wb, "send_keys", serde_json::json!({
                "selector": selector,
                "keys": "Enter"
            })).await;
        }

        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), data);
        Ok(outputs)
    }
}
