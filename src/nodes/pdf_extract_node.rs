//! PDF Extract node — extracts text from PDF files.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct PdfExtractNode;

#[async_trait]
impl NodeExecutor for PdfExtractNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "pdf_extract".to_string(),
            display_name: "PDF 文本提取".to_string(),
            description: "从 PDF 文件中提取文本内容".to_string(),
            category: "文档".to_string(),
            inputs: vec![
                PortDef { label: "path".to_string(), data_type: "string".to_string(), required: true },
            ],
            outputs: vec![
                PortDef { label: "text".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "pages".to_string(), data_type: "number".to_string(), required: false },
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "PDF file path" },
                    "pages": { "type": "string", "description": "Page range (e.g., '1-5', 'all')", "default": "all" },
                    "max_chars": { "type": "number", "default": 100000, "description": "Max characters to extract" }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let path = config["path"].as_str()
            .or_else(|| inputs.get("path").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: "path is required".to_string(),
            })?;

        let max_chars = config["max_chars"].as_u64().unwrap_or(100_000) as usize;

        tracing::info!("Extracting text from PDF: {}", path);

        // Check file exists
        if !Path::new(path).exists() {
            return Err(FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("PDF file not found: {}", path),
            });
        }

        // Extract text using pdf-extract
        let text = pdf_extract::extract_text(path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("PDF extract error: {}", e),
        })?;

        // Truncate if needed
        let truncated = if text.len() > max_chars {
            let mut s = text[..max_chars].to_string();
            s.push_str("\n... [truncated]");
            s
        } else {
            text
        };

        // Count pages (approximate by counting form feeds)
        let pages = truncated.matches('\x0C').count().max(1);

        let mut outputs = HashMap::new();
        outputs.insert("text".to_string(), serde_json::json!(truncated));
        outputs.insert("pages".to_string(), serde_json::json!(pages));
        outputs.insert("success".to_string(), serde_json::json!(true));
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
            node_type: "pdf_extract".to_string(),
            label: "Test PDF".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_pdf_no_path() {
        let node = make_node("pdf_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = PdfExtractNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pdf_file_not_found() {
        let node = make_node("pdf_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"path": "/tmp/nonexistent.pdf"});
        let inputs = HashMap::new();
        let result = PdfExtractNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pdf_type_def() {
        let def = PdfExtractNode.type_def();
        assert_eq!(def.type_name, "pdf_extract");
        assert_eq!(def.outputs.len(), 3);
    }
}
