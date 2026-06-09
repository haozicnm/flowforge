//! HTTP node — makes HTTP requests.
//!
//! This is the most commonly used node. It demonstrates the correct pattern:
//! - Execute receives already-resolved config
//! - Returns outputs keyed by port label
//! - No variable resolution logic inside

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct HttpNode;

#[async_trait]
impl NodeExecutor for HttpNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "http".to_string(),
            display_name: "HTTP 请求".to_string(),
            description: "发送 HTTP 请求并返回响应".to_string(),
            category: "网络".to_string(),
            inputs: vec![PortDef {
                label: "url".to_string(),
                data_type: "string".to_string(),
                required: true,
            }],
            outputs: vec![
                PortDef {
                    label: "status".to_string(),
                    data_type: "number".to_string(),
                    required: false,
                },
                PortDef {
                    label: "body".to_string(),
                    data_type: "string".to_string(),
                    required: false,
                },
                PortDef {
                    label: "headers".to_string(),
                    data_type: "object".to_string(),
                    required: false,
                },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string"},
                    "method": {"type": "string", "default": "GET"},
                    "headers": {"type": "object"},
                    "body": {"type": "string"},
                    "timeout_secs": {"type": "number", "default": 30}
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let url = config["url"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "http".to_string(),
                detail: "url is required and must be a string".to_string(),
            })?;

        let method = config["method"].as_str().unwrap_or("GET");
        let timeout = config["timeout_secs"].as_u64().unwrap_or(30);

        // TODO: actual HTTP request
        tracing::info!("HTTP {} {} (timeout: {}s)", method, url, timeout);

        let mut outputs = HashMap::new();
        outputs.insert("status".to_string(), serde_json::json!(200));
        outputs.insert("body".to_string(), serde_json::json!("{}"));
        outputs.insert("headers".to_string(), serde_json::json!({}));
        Ok(outputs)
    }
}
