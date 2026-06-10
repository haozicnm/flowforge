//! DOCX Create node — generate a .docx file from text content.
//!
//! Creates a minimal but valid .docx file using the zip crate.
//! A .docx file is a ZIP archive containing XML files following the
//! Open Packaging Convention (OPC).
use async_trait::async_trait;
use std::collections::HashMap;
use std::io::Write;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct DocxCreateNode;

#[async_trait]
impl NodeExecutor for DocxCreateNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "docx_create".to_string(),
            display_name: "创建 Word".to_string(),
            description: "从文本内容生成 .docx 文件".to_string(),
            category: "Word".to_string(),
            inputs: vec![
                PortDef { label: "text".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "paragraphs".to_string(), data_type: "array".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "path".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Output .docx file path" },
                    "title": { "type": "string", "description": "Document title" },
                    "font_size": { "type": "number", "default": 24, "description": "Font size in half-points (24 = 12pt)" },
                    "font_name": { "type": "string", "default": "宋体" }
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
                node_id: "docx_create".to_string(),
                detail: "path is required".to_string(),
            })?;

        let title = config["title"].as_str().unwrap_or("");
        let font_size = config["font_size"].as_u64().unwrap_or(24);
        let font_name = config["font_name"].as_str().unwrap_or("宋体");

        // Get paragraphs from input or split text
        let paragraphs: Vec<String> = if let Some(para_input) = inputs.get("paragraphs") {
            if let Some(arr) = para_input.as_array() {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            } else {
                vec![para_input.to_string()]
            }
        } else if let Some(text) = inputs.get("text").and_then(|v| v.as_str()) {
            text.lines().map(|l| l.to_string()).collect()
        } else if let Some(text) = config["text"].as_str() {
            text.lines().map(|l| l.to_string()).collect()
        } else {
            vec![]
        };

        // Build document.xml
        let mut doc_xml = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas"
            xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            xmlns:o="urn:schemas-microsoft-com:office:office"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
            xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"
            xmlns:v="urn:schemas-microsoft-com:vml"
            xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
            xmlns:w10="urn:schemas-microsoft-com:office:word"
            xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"
            xmlns:wpg="http://schemas.microsoft.com/office/word/2010/wordprocessingGroup"
            xmlns:wpi="http://schemas.microsoft.com/office/word/2010/wordprocessingInk"
            xmlns:wne="http://schemas.microsoft.com/office/word/2006/wordml"
            xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"
            mc:Ignorable="w14 wp14">
<w:body>"#);

        // Add title paragraph if specified
        if !title.is_empty() {
            doc_xml.push_str(&format!(
                r#"<w:p>
  <w:pPr><w:jc w:val="center"/><w:rPr><w:b/><w:sz w:val="36"/></w:rPr></w:pPr>
  <w:r><w:rPr><w:b/><w:sz w:val="36"/><w:rFonts w:ascii="{font}" w:eastAsia="{font}"/></w:rPr><w:t xml:space="preserve">{title}</w:t></w:r>
</w:p>"#,
                font = xml_escape(font_name),
                title = xml_escape(title)
            ));
        }

        // Add content paragraphs
        for para in &paragraphs {
            doc_xml.push_str(&format!(
                r#"<w:p>
  <w:r><w:rPr><w:sz w:val="{size}"/><w:rFonts w:ascii="{font}" w:eastAsia="{font}"/></w:rPr><w:t xml:space="preserve">{text}</w:t></w:r>
</w:p>"#,
                size = font_size,
                font = xml_escape(font_name),
                text = xml_escape(para)
            ));
        }

        doc_xml.push_str("</w:body></w:document>");

        // Create the .docx ZIP archive
        let docx_file = std::fs::File::create(path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "docx_create".to_string(),
            detail: format!("failed to create file: {}", e),
        })?;

        let mut zip = zip::ZipWriter::new(docx_file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let write_entry = |zip: &mut zip::ZipWriter<std::fs::File>, name: &str, data: &[u8]| -> FlowResult<()> {
            zip.start_file(name, options).map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "docx_create".to_string(),
                detail: format!("zip entry '{}' error: {}", name, e),
            })?;
            zip.write_all(data).map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "docx_create".to_string(),
                detail: format!("zip write '{}' error: {}", name, e),
            })?;
            Ok(())
        };

        write_entry(&mut zip, "[Content_Types].xml", CONTENT_TYPES.as_bytes())?;
        write_entry(&mut zip, "_rels/.rels", RELS.as_bytes())?;
        write_entry(&mut zip, "word/_rels/document.xml.rels", DOC_RELS.as_bytes())?;
        write_entry(&mut zip, "word/document.xml", doc_xml.as_bytes())?;
        write_entry(&mut zip, "word/styles.xml", STYLES.as_bytes())?;

        zip.finish().map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "docx_create".to_string(),
            detail: format!("failed to finalize zip: {}", e),
        })?;

        tracing::info!("Docx: created '{}' with {} paragraphs", path, paragraphs.len());

        let mut outputs = HashMap::new();
        outputs.insert("path".to_string(), serde_json::json!(path));
        Ok(outputs)
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

const RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const DOC_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/>
  </w:style>
</w:styles>"#;
