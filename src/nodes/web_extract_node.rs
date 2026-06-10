//! Web Extract node — extract data from the page via WebBridge.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};
use super::webbridge::WebBridgeClient;

#[derive(Default)]
pub struct WebExtractNode;

#[async_trait]
impl NodeExecutor for WebExtractNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "web_extract".to_string(),
            display_name: "提取数据".to_string(),
            description: "从页面提取文本、HTML、属性、表格、链接等".to_string(),
            category: "网页自动化".to_string(),
            inputs: vec![
                PortDef { label: "selector".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "text".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "html".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "items".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "table".to_string(), data_type: "object".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "selector": { "type": "string", "description": "CSS selector" },
                    "extract_type": {
                        "type": "string",
                        "enum": ["text", "html", "attribute", "links", "table", "title", "url"],
                        "default": "text"
                    },
                    "attribute": { "type": "string", "description": "Attribute name (for extract_type=attribute)" }
                },
                "required": ["extract_type"]
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let extract_type = config["extract_type"].as_str().unwrap_or("text");
        let selector = config["selector"].as_str()
            .or_else(|| inputs.get("selector").and_then(|v| v.as_str()))
            .unwrap_or("body");

        let client = WebBridgeClient::default();

        let (action, params) = match extract_type {
            "text" => ("extract_text", serde_json::json!({ "selector": selector })),
            "html" => ("extract_html", serde_json::json!({ "selector": selector })),
            "attribute" => {
                let attr = config["attribute"].as_str().unwrap_or("href");
                ("extract_attribute", serde_json::json!({
                    "selector": selector, "attribute": attr
                }))
            }
            "links" => ("extract_links", serde_json::json!({ "selector": selector })),
            "table" => ("extract_table", serde_json::json!({ "selector": selector })),
            "title" => ("get_title", serde_json::json!({})),
            "url" => ("current_url", serde_json::json!({})),
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "web_extract".to_string(),
                    detail: format!("unknown extract_type: {}", extract_type),
                });
            }
        };

        let data = client.send_command(action, params).await
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "web_extract".to_string(),
                detail: e,
            })?;

        let mut outputs = HashMap::new();
        match extract_type {
            "text" => {
                outputs.insert("text".to_string(), serde_json::json!(
                    data["text"].as_str().unwrap_or("")
                ));
            }
            "html" => {
                outputs.insert("html".to_string(), serde_json::json!(
                    data["html"].as_str().unwrap_or("")
                ));
            }
            "attribute" => {
                outputs.insert("text".to_string(), serde_json::json!(
                    data["value"].as_str().unwrap_or("")
                ));
            }
            "links" | "table" => {
                outputs.insert("items".to_string(), data.clone());
                outputs.insert("table".to_string(), data);
            }
            "title" => {
                outputs.insert("text".to_string(), serde_json::json!(
                    data["title"].as_str().unwrap_or("")
                ));
            }
            "url" => {
                outputs.insert("text".to_string(), serde_json::json!(
                    data["url"].as_str().unwrap_or("")
                ));
            }
            _ => {}
        }
        Ok(outputs)
    }
}
