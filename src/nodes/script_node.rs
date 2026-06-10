//! Script node — runs Rhai scripts for data transformation.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ScriptNode;

#[async_trait]
impl NodeExecutor for ScriptNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "script".to_string(),
            display_name: "脚本".to_string(),
            description: "运行 Rhai 脚本进行数据转换".to_string(),
            category: "数据处理".to_string(),
            inputs: vec![PortDef { label: "input".to_string(), data_type: "any".to_string(), required: false }],
            outputs: vec![PortDef { label: "result".to_string(), data_type: "any".to_string(), required: false }],
            config_schema: serde_json::json!({"type": "object", "properties": {"script": {"type": "string"}, "language": {"type": "string", "default": "rhai"}}, "required": ["script"]}),
        }
    }

    async fn execute(&self, _node: &Node, _ctx: &crate::nodes::traits::NodeContext, config: serde_json::Value, inputs: HashMap<String, serde_json::Value>) -> FlowResult<HashMap<String, serde_json::Value>> {
        let script = config["script"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: "script".to_string(), detail: "script is required".to_string(),
        })?;
        let engine = rhai::Engine::new();
        let mut scope = rhai::Scope::new();
        for (k, v) in &inputs {
            scope.push_dynamic(k.clone(), rhai::Dynamic::from(v.clone()));
        }
        let result: rhai::Dynamic = engine.eval_with_scope(&mut scope, script)
            .map_err(|e| FlowError::ExecutionError(format!("Rhai error: {}", e)))?;
        let mut out = HashMap::new();
        out.insert("result".to_string(), serde_json::json!(result.to_string()));
        Ok(out)
    }
}
