//! Webhook trigger — listens for incoming HTTP requests.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::FlowResult;
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct WebhookNode;

#[async_trait]
impl NodeExecutor for WebhookNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "webhook".to_string(),
            display_name: "Webhook".to_string(),
            description: "监听 HTTP 请求触发工作流".to_string(),
            category: "触发器".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef { label: "body".to_string(), data_type: "object".to_string(), required: false },
                PortDef { label: "headers".to_string(), data_type: "object".to_string(), required: false },
            ],
            config_schema: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}, "method": {"type": "string", "default": "POST"}}}),
        }
    }

    async fn execute(&self, _node: &Node, _ctx: &crate::nodes::traits::NodeContext, _config: serde_json::Value, _inputs: HashMap<String, serde_json::Value>) -> FlowResult<HashMap<String, serde_json::Value>> {
        let mut out = HashMap::new();
        out.insert("body".to_string(), serde_json::json!({}));
        out.insert("headers".to_string(), serde_json::json!({}));
        Ok(out)
    }
}
