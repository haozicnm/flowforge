//! Delay node — pauses execution for a specified duration.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::FlowResult;
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct DelayNode;

#[async_trait]
impl NodeExecutor for DelayNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "delay".to_string(),
            display_name: "延时等待".to_string(),
            description: "暂停执行指定时间".to_string(),
            category: "控制流".to_string(),
            inputs: vec![],
            outputs: vec![PortDef { label: "elapsed".to_string(), data_type: "number".to_string(), required: false }],
            config_schema: serde_json::json!({"type": "object", "properties": {"duration_ms": {"type": "number", "default": 1000}}}),
        }
    }

    async fn execute(&self, _node: &Node, _ctx: &crate::nodes::traits::NodeContext, config: serde_json::Value, _inputs: HashMap<String, serde_json::Value>) -> FlowResult<HashMap<String, serde_json::Value>> {
        let ms = config["duration_ms"].as_u64().unwrap_or(1000);
        tracing::info!("Delaying {}ms", ms);
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        let mut out = HashMap::new();
        out.insert("elapsed".to_string(), serde_json::json!(ms));
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::traits::NodeContext;

    fn make_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            node_type: "delay".to_string(),
            label: "Test Delay".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_delay_basic() {
        let node = make_node("delay_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"duration_ms": 100});
        let inputs = HashMap::new();
        let start = std::time::Instant::now();
        let result = DelayNode.execute(&node, &ctx, config, inputs).await.unwrap();
        let elapsed = start.elapsed();
        assert_eq!(result["elapsed"], 100);
        assert!(elapsed.as_millis() >= 90); // Allow some tolerance
    }

    #[tokio::test]
    async fn test_delay_default() {
        let node = make_node("delay_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = DelayNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["elapsed"], 1000);
    }

    #[tokio::test]
    async fn test_delay_type_def() {
        let def = DelayNode.type_def();
        assert_eq!(def.type_name, "delay");
        assert_eq!(def.outputs.len(), 1);
    }
}
