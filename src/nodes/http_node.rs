//! HTTP node — makes real HTTP requests using reqwest.
//!
//! This is the most commonly used node. It demonstrates the correct pattern:
//! - Execute receives already-resolved config
//! - Returns outputs keyed by port label
//! - No variable resolution logic inside

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct HttpNode;

#[async_trait]
impl NodeExecutor for HttpNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "http".to_string(),
            display_name: "HTTP 请求".to_string(),
            description: "发送 HTTP 请求并返回响应".to_string(),
            category: "网络".to_string(),
            inputs: vec![PortDef {
                label: "url".to_string(),
                data_type: "string".to_string(),
                required: true,
            }],
            outputs: vec![
                PortDef {
                    label: "status".to_string(),
                    data_type: "number".to_string(),
                    required: false,
                },
                PortDef {
                    label: "body".to_string(),
                    data_type: "string".to_string(),
                    required: false,
                },
                PortDef {
                    label: "headers".to_string(),
                    data_type: "object".to_string(),
                    required: false,
                },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string"},
                    "method": {"type": "string", "default": "GET"},
                    "headers": {"type": "object"},
                    "body": {"type": "string"},
                    "timeout_secs": {"type": "number", "default": 30}
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let url = config["url"]
            .as_str()
            .or_else(|| inputs.get("url").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "http".to_string(),
                detail: "url is required and must be a string".to_string(),
            })?;

        let method = config["method"].as_str().unwrap_or("GET");
        let timeout = config["timeout_secs"].as_u64().unwrap_or(30);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "http".to_string(),
                detail: format!("failed to create HTTP client: {}", e),
            })?;

        let mut req_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            "HEAD" => client.head(url),
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "http".to_string(),
                    detail: format!("unsupported method: {}", method),
                });
            }
        };

        // Add headers
        if let Some(headers) = config["headers"].as_object() {
            for (key, value) in headers {
                if let Some(val) = value.as_str() {
                    req_builder = req_builder.header(key.as_str(), val);
                }
            }
        }

        // Add body
        if let Some(body) = config["body"].as_str() {
            req_builder = req_builder
                .header("content-type", "application/json")
                .body(body.to_string());
        }

        tracing::info!("HTTP {} {} (timeout: {}s)", method, url, timeout);

        let response = req_builder.send().await.map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "http".to_string(),
            detail: format!("request failed: {}", e),
        })?;

        let status = response.status().as_u16();
        let resp_headers: serde_json::Value = serde_json::json!(
            response.headers().iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect::<HashMap<String, String>>()
        );

        let body_text = response.text().await.map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "http".to_string(),
            detail: format!("failed to read response body: {}", e),
        })?;

        tracing::info!("HTTP response: {} ({} bytes)", status, body_text.len());

        let mut outputs = HashMap::new();
        outputs.insert("status".to_string(), serde_json::json!(status));
        outputs.insert("body".to_string(), serde_json::json!(body_text));
        outputs.insert("headers".to_string(), resp_headers);
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
            node_type: "http".to_string(),
            label: "Test HTTP".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_http_no_url() {
        let node = make_node("http_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = HttpNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_http_invalid_method() {
        let node = make_node("http_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"url": "http://example.com", "method": "INVALID"});
        let inputs = HashMap::new();
        let result = HttpNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_http_type_def() {
        let def = HttpNode.type_def();
        assert_eq!(def.type_name, "http");
        assert_eq!(def.outputs.len(), 3);
    }
}
