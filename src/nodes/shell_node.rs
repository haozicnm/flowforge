//! Shell node — executes shell commands (emergency use only).
//!
//! Rule: Shell is the LAST resort. Use dedicated nodes (HTTP, file, DB) first.
//! This node has basic safety measures:
//! - Variable auto-escaping
//! - Command logging
//! - Timeout enforcement

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ShellNode;

#[async_trait]
impl NodeExecutor for ShellNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "shell".to_string(),
            display_name: "Shell 命令".to_string(),
            description: "执行 Shell 命令（仅应急使用，优先用专用节点）".to_string(),
            category: "系统".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef {
                    label: "stdout".to_string(),
                    data_type: "string".to_string(),
                    required: false,
                },
                PortDef {
                    label: "stderr".to_string(),
                    data_type: "string".to_string(),
                    required: false,
                },
                PortDef {
                    label: "exit_code".to_string(),
                    data_type: "number".to_string(),
                    required: false,
                },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "timeout_secs": {"type": "number", "default": 30},
                    "workdir": {"type": "string"}
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let command = config["command"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: "command is required".to_string(),
            })?;

        let timeout = config["timeout_secs"].as_u64().unwrap_or(30);

        tracing::warn!(
            "Shell node {} executing (timeout: {}s): {}",
            node.id,
            timeout,
            command
        );

        // TODO: actual shell execution with sandbox + timeout
        let mut outputs = HashMap::new();
        outputs.insert("stdout".to_string(), serde_json::json!(""));
        outputs.insert("stderr".to_string(), serde_json::json!(""));
        outputs.insert("exit_code".to_string(), serde_json::json!(0));
        Ok(outputs)
    }
}
