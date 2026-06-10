//! Web Click node — click an element on the page via WebBridge.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct WebClickNode;

#[async_trait]
impl NodeExecutor for WebClickNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "web_click".to_string(),
            display_name: "点击元素".to_string(),
            description: "点击页面上的元素（CSS 选择器或 ref）".to_string(),
            category: "网页自动化".to_string(),
            inputs: vec![
                PortDef { label: "selector".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "out".to_string(), data_type: "object".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "selector": { "type": "string", "description": "CSS selector or @e ref" },
                    "click_type": {
                        "type": "string",
                        "enum": ["single", "double", "right"],
                        "default": "single"
                    },
                    "wait_after_ms": { "type": "number", "default": 0 }
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
                node_id: "web_click".to_string(),
                detail: "selector is required".to_string(),
            })?;

        let click_type = config["click_type"].as_str().unwrap_or("single");
        let action = match click_type {
            "double" => "double_click",
            "right" => "context_menu",
            _ => "click",
        };

        let params = serde_json::json!({ "selector": selector });
        let wb = ctx.webbridge.as_ref().ok_or_else(|| FlowError::NodeExecutionFailed {
            node_id: "web".to_string(),
            detail: "WebBridge not configured. Browser automation requires a connected Chrome extension.".to_string(),
        })?;
        let data = super::webbridge::send_browser_command(wb, action, params).await
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "web_click".to_string(),
                detail: e.to_string(),
            })?;

        // Optional wait after click
        let wait_ms = config["wait_after_ms"].as_u64().unwrap_or(0);
        if wait_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
        }

        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), data);
        Ok(outputs)
    }
}
