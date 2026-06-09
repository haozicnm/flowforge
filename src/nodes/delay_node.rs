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

    async fn execute(&self, _node: &Node, config: serde_json::Value, _inputs: HashMap<String, serde_json::Value>) -> FlowResult<HashMap<String, serde_json::Value>> {
        let ms = config["duration_ms"].as_u64().unwrap_or(1000);
        tracing::info!("Delaying {}ms", ms);
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
        let mut out = HashMap::new();
        out.insert("elapsed".to_string(), serde_json::json!(ms));
        Ok(out)
    }
}
