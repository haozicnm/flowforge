//! Web Navigate node — open a URL in the browser via WebBridge.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};
use super::webbridge::WebBridgeClient;

#[derive(Default)]
pub struct WebNavigateNode;

#[async_trait]
impl NodeExecutor for WebNavigateNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "web_navigate".to_string(),
            display_name: "打开网页".to_string(),
            description: "在浏览器中打开指定 URL".to_string(),
            category: "网页自动化".to_string(),
            inputs: vec![
                PortDef { label: "url".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "title".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "url".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "URL to navigate to" },
                    "wait_until": {
                        "type": "string",
                        "enum": ["load", "domcontentloaded", "networkidle"],
                        "default": "load"
                    },
                    "new_tab": { "type": "boolean", "default": false }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let url = config["url"].as_str()
            .or_else(|| inputs.get("url").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "web_navigate".to_string(),
                detail: "url is required".to_string(),
            })?;

        let new_tab = config["new_tab"].as_bool().unwrap_or(false);

        let params = serde_json::json!({
            "url": url,
            "newTab": new_tab
        });

        let client = WebBridgeClient::default();
        let data = client.send_command("navigate", params).await
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "web_navigate".to_string(),
                detail: e,
            })?;

        let mut outputs = HashMap::new();
        outputs.insert("title".to_string(), serde_json::json!(
            data["title"].as_str().unwrap_or("")
        ));
        outputs.insert("url".to_string(), serde_json::json!(
            data["url"].as_str().unwrap_or(url)
        ));
        Ok(outputs)
    }
}
