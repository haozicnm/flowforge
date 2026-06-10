//! JSON node — parse, extract, merge, and manipulate JSON data.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct JsonNode;

#[async_trait]
impl NodeExecutor for JsonNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "json".to_string(),
            display_name: "JSON 处理".to_string(),
            description: "解析、提取、合并 JSON 数据".to_string(),
            category: "数据操作".to_string(),
            inputs: vec![
                PortDef { label: "in".to_string(), data_type: "any".to_string(), required: true },
            ],
            outputs: vec![
                PortDef { label: "out".to_string(), data_type: "any".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["extract", "stringify", "parse", "merge", "keys", "values", "length", "flatten"],
                        "default": "extract"
                    },
                    "path": { "type": "string", "description": "JSONPath-like dot notation: data.items.0.name" },
                    "merge_with": { "description": "Object to merge with (for merge operation)" }
                },
                "required": ["operation"]
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let operation = config["operation"].as_str().unwrap_or("extract");
        let input = inputs.get("in").cloned().unwrap_or(serde_json::Value::Null);

        let result = match operation {
            "extract" => {
                let path = config["path"].as_str().unwrap_or("");
                extract_path(&input, path)
            }
            "stringify" => {
                serde_json::json!(serde_json::to_string(&input).unwrap_or_default())
            }
            "parse" => {
                let s = input.as_str().unwrap_or("");
                serde_json::from_str(s).unwrap_or(serde_json::Value::Null)
            }
            "merge" => {
                if let (Some(mut obj), Some(merge)) = (input.as_object().cloned(), config.get("merge_with")) {
                    if let Some(merge_obj) = merge.as_object() {
                        for (k, v) in merge_obj {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                    serde_json::Value::Object(obj)
                } else {
                    input
                }
            }
            "keys" => {
                if let Some(obj) = input.as_object() {
                    serde_json::json!(obj.keys().collect::<Vec<_>>())
                } else {
                    serde_json::json!([])
                }
            }
            "values" => {
                if let Some(obj) = input.as_object() {
                    serde_json::json!(obj.values().collect::<Vec<_>>())
                } else {
                    serde_json::json!([])
                }
            }
            "length" => {
                let len = match &input {
                    serde_json::Value::Array(a) => a.len(),
                    serde_json::Value::Object(o) => o.len(),
                    serde_json::Value::String(s) => s.len(),
                    _ => 0,
                };
                serde_json::json!(len)
            }
            "flatten" => {
                let mut flat = Vec::new();
                flatten_value(&input, &mut flat);
                serde_json::json!(flat)
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "json".to_string(),
                    detail: format!("unknown operation: {}", operation),
                });
            }
        };

        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), result);
        Ok(outputs)
    }
}

/// Extract value using dot-notation path: "data.items.0.name"
fn extract_path(value: &serde_json::Value, path: &str) -> serde_json::Value {
    if path.is_empty() {
        return value.clone();
    }
    let mut current = value;
    for part in path.split('.') {
        current = match current {
            serde_json::Value::Object(obj) => obj.get(part).unwrap_or(&serde_json::Value::Null),
            serde_json::Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    arr.get(idx).unwrap_or(&serde_json::Value::Null)
                } else {
                    return serde_json::Value::Null;
                }
            }
            _ => return serde_json::Value::Null,
        };
    }
    current.clone()
}

fn flatten_value(value: &serde_json::Value, out: &mut Vec<serde_json::Value>) {
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                flatten_value(item, out);
            }
        }
        _ => out.push(value.clone()),
    }
}
