//! Webhook node — receives external HTTP triggers.
//!
//! When a POST/GET request arrives at /api/webhook/:workflow_id/:node_id,
//! the payload is stored in WebhookStore. This node pops the pending payload
//! and outputs it during execution.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct WebhookNode;

#[async_trait]
impl NodeExecutor for WebhookNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "webhook".to_string(),
            display_name: "Webhook".to_string(),
            description: "接收外部 HTTP 请求作为触发器".to_string(),
            category: "触发器".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef { label: "body".to_string(), data_type: "object".to_string(), required: false },
                PortDef { label: "headers".to_string(), data_type: "object".to_string(), required: false },
                PortDef { label: "method".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "has_payload".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "description": {"type": "string", "description": "描述此 Webhook 的用途"}
                }
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        ctx: &crate::nodes::traits::NodeContext,
        _config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let mut outputs = HashMap::new();

        // Try to get the pending webhook payload from the store
        // The store key is computed dynamically — we look it up from the executor's
        // webhook_store using a key convention: "workflow_id:node_id".
        // Since we don't know the workflow_id at node level, we scan for any key
        // ending with ":node_id".
        if let Some(store) = &ctx.webhook_store {
            let mut store_guard = store.lock().map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("webhook store lock error: {}", e),
            })?;

            // Find the first key matching this node_id
            let matching_key = store_guard
                .keys()
                .find(|k| k.ends_with(&format!(":{}", node.id)))
                .cloned();

            if let Some(key) = matching_key {
                if let Some(payloads) = store_guard.get_mut(&key) {
                    if let Some(payload) = payloads.pop() {
                        if payloads.is_empty() {
                            store_guard.remove(&key);
                        }
                        drop(store_guard);

                        let body = payload.get("body").cloned().unwrap_or(serde_json::json!({}));
                        let headers = payload.get("headers").cloned().unwrap_or(serde_json::json!({}));
                        let method = payload.get("method").and_then(|v| v.as_str()).unwrap_or("");

                        outputs.insert("body".into(), body);
                        outputs.insert("headers".into(), headers);
                        outputs.insert("method".into(), serde_json::json!(method));
                        outputs.insert("has_payload".into(), serde_json::json!(true));

                        tracing::info!("Webhook node {}: processed payload", node.id);
                        return Ok(outputs);
                    }
                }
            }
        }

        // No pending payload — return empty
        outputs.insert("body".into(), serde_json::json!({}));
        outputs.insert("headers".into(), serde_json::json!({}));
        outputs.insert("method".into(), serde_json::json!(""));
        outputs.insert("has_payload".into(), serde_json::json!(false));

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::traits::NodeContext;

    fn make_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            node_type: "webhook".to_string(),
            label: "Test Webhook".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_webhook_no_payload() {
        let node = make_node("webhook_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = WebhookNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["has_payload"], false);
        assert_eq!(result["body"], serde_json::json!({}));
    }

    #[tokio::test]
    async fn test_webhook_type_def() {
        let def = WebhookNode.type_def();
        assert_eq!(def.type_name, "webhook");
        assert_eq!(def.outputs.len(), 4);
    }
}
