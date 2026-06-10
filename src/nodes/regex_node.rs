//! Regex node — pattern matching, extraction, and replacement.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct RegexNode;

#[async_trait]
impl NodeExecutor for RegexNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "regex".to_string(),
            display_name: "正则匹配".to_string(),
            description: "正则表达式匹配、提取、替换".to_string(),
            category: "数据操作".to_string(),
            inputs: vec![
                PortDef { label: "text".to_string(), data_type: "string".to_string(), required: true },
            ],
            outputs: vec![
                PortDef { label: "matches".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "result".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "count".to_string(), data_type: "number".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Regex pattern" },
                    "operation": {
                        "type": "string",
                        "enum": ["match", "replace", "split", "find_all"],
                        "default": "match"
                    },
                    "replacement": { "type": "string", "description": "Replacement string (for replace op)" },
                    "flags": { "type": "string", "description": "Flags: i=ignorecase, m=multiline", "default": "" }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let text = inputs.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let pattern = config["pattern"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: "regex".to_string(),
            detail: "pattern is required".to_string(),
        })?;
        let operation = config["operation"].as_str().unwrap_or("match");

        let re = regex::Regex::new(pattern).map_err(|e| FlowError::InvalidNodeConfig {
            node_id: "regex".to_string(),
            detail: format!("invalid regex: {}", e),
        })?;

        let mut outputs = HashMap::new();

        match operation {
            "match" => {
                if let Some(caps) = re.captures(text) {
                    let matches: Vec<String> = caps.iter()
                        .filter_map(|m| m.map(|m| m.as_str().to_string()))
                        .collect();
                    outputs.insert("matches".to_string(), serde_json::json!(matches));
                    outputs.insert("count".to_string(), serde_json::json!(matches.len()));
                    outputs.insert("result".to_string(), serde_json::json!(caps.get(0).map(|m| m.as_str()).unwrap_or("")));
                } else {
                    outputs.insert("matches".to_string(), serde_json::json!([]));
                    outputs.insert("count".to_string(), serde_json::json!(0));
                    outputs.insert("result".to_string(), serde_json::json!(""));
                }
            }
            "replace" => {
                let replacement = config["replacement"].as_str().unwrap_or("");
                let result = re.replace_all(text, replacement);
                outputs.insert("result".to_string(), serde_json::json!(result.as_ref()));
                outputs.insert("matches".to_string(), serde_json::json!([]));
                outputs.insert("count".to_string(), serde_json::json!(re.find_iter(text).count()));
            }
            "split" => {
                let parts: Vec<&str> = re.split(text).collect();
                outputs.insert("matches".to_string(), serde_json::json!(parts));
                outputs.insert("count".to_string(), serde_json::json!(parts.len()));
                outputs.insert("result".to_string(), serde_json::json!(parts.join("")));
            }
            "find_all" => {
                let all: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_string()).collect();
                outputs.insert("matches".to_string(), serde_json::json!(all));
                outputs.insert("count".to_string(), serde_json::json!(re.find_iter(text).count()));
                outputs.insert("result".to_string(), serde_json::json!(text));
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "regex".to_string(),
                    detail: format!("unknown operation: {}", operation),
                });
            }
        }
        Ok(outputs)
    }
}
