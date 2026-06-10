//! DOCX Read node — extract text from .docx files.
//!
//! Uses the zip crate to extract document.xml from the .docx archive
//! and quick-xml to parse the Word XML format.
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct DocxReadNode;

#[async_trait]
impl NodeExecutor for DocxReadNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "docx_read".to_string(),
            display_name: "读取 Word".to_string(),
            description: "从 .docx 文件提取文本内容".to_string(),
            category: "Word".to_string(),
            inputs: vec![
                PortDef { label: "path".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "text".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "paragraphs".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "count".to_string(), data_type: "number".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to .docx file" },
                    "include_tables": { "type": "boolean", "default": true }
                },
                "required": ["path"]
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
        let path = config["path"].as_str()
            .or_else(|| inputs.get("path").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "docx_read".to_string(),
                detail: "path is required".to_string(),
            })?;

        let file = File::open(path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "docx_read".to_string(),
            detail: format!("failed to open '{}': {}", path, e),
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "docx_read".to_string(),
            detail: format!("not a valid zip/docx: {}", e),
        })?;

        // Read document.xml from the docx archive
        let mut xml_content = String::new();
        {
            let mut doc_file = archive.by_name("word/document.xml").map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "docx_read".to_string(),
                detail: format!("missing word/document.xml: {}", e),
            })?;
            doc_file.read_to_string(&mut xml_content).map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "docx_read".to_string(),
                detail: format!("failed to read document.xml: {}", e),
            })?;
        }

        // Parse paragraphs from Word XML
        let paragraphs = extract_paragraphs(&xml_content);
        let full_text = paragraphs.join("\n");
        let count = paragraphs.len();

        tracing::info!("Docx: read {} paragraphs from '{}'", count, path);

        let mut outputs = HashMap::new();
        outputs.insert("text".to_string(), serde_json::json!(full_text));
        outputs.insert("paragraphs".to_string(), serde_json::json!(paragraphs));
        outputs.insert("count".to_string(), serde_json::json!(count));
        Ok(outputs)
    }
}

/// Extract paragraph text from Word XML using quick-xml.
///
/// Word XML structure:
/// <w:document>
///   <w:body>
///     <w:p>
///       <w:r>
///         <w:t>Hello</w:t>
///       </w:r>
///     </w:p>
///   </w:body>
/// </w:document>
fn extract_paragraphs(xml: &str) -> Vec<String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut in_paragraph = false;
    let mut in_table = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                let name = String::from_utf8_lossy(&name_bytes);
                if name == "w:p" {
                    in_paragraph = true;
                    current_text.clear();
                } else if name == "w:tbl" {
                    in_table = true;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_paragraph && !in_table {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                let name = String::from_utf8_lossy(&name_bytes);
                if name == "w:p" {
                    if in_paragraph && !current_text.trim().is_empty() {
                        paragraphs.push(current_text.trim().to_string());
                    }
                    in_paragraph = false;
                    current_text.clear();
                } else if name == "w:tbl" {
                    in_table = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    paragraphs
}
