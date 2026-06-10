//! Web Wait node — wait for an element or condition on the page.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct WebWaitNode;

#[async_trait]
impl NodeExecutor for WebWaitNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "web_wait".to_string(),
            display_name: "等待元素".to_string(),
            description: "等待页面元素出现或文本出现".to_string(),
            category: "网页自动化".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef { label: "found".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "selector": { "type": "string", "description": "CSS selector to wait for" },
                    "text": { "type": "string", "description": "Text content to wait for" },
                    "timeout_ms": { "type": "number", "default": 10000 },
                    "wait_type": {
                        "type": "string",
                        "enum": ["selector", "text", "navigation"],
                        "default": "selector"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let timeout_ms = config["timeout_ms"].as_u64().unwrap_or(10000);
        let wait_type = config["wait_type"].as_str().unwrap_or("selector");

        let params = match wait_type {
            "selector" => {
                let selector = config["selector"].as_str().unwrap_or("body");
                serde_json::json!({
                    "selector": selector,
                    "timeout": timeout_ms
                })
            }
            "text" => {
                let text = config["text"].as_str().unwrap_or("");
                serde_json::json!({
                    "text": text,
                    "timeout": timeout_ms
                })
            }
            "navigation" => serde_json::json!({ "timeout": timeout_ms }),
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "web_wait".to_string(),
                    detail: format!("unknown wait_type: {}", wait_type),
                });
            }
        };

        let wb = ctx.webbridge.as_ref().ok_or_else(|| FlowError::NodeExecutionFailed {
            node_id: "web".to_string(),
            detail: "WebBridge not configured. Browser automation requires a connected Chrome extension.".to_string(),
        })?;
        let found = super::webbridge::send_browser_command(wb, "wait_for", params).await.is_ok();

        let mut outputs = HashMap::new();
        outputs.insert("found".to_string(), serde_json::json!(found));
        Ok(outputs)
    }
}
