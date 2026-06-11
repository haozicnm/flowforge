//! Data transform node — converts between JSON, CSV, and XML formats.
//!
//! Supported transforms:
//!   - json_to_csv: JSON array of objects → CSV string
//!   - csv_to_json: CSV string → JSON array of objects
//!   - json_to_xml: JSON object/array → XML string
//!   - xml_to_json: XML string → JSON object
//!   - csv_to_xml: CSV string → XML string (via JSON intermediate)
//!   - xml_to_csv: XML string → CSV string (via JSON intermediate)

use async_trait::async_trait;
use std::collections::HashMap;
use std::io::Cursor;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct TransformNode;

#[async_trait]
impl NodeExecutor for TransformNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "transform".to_string(),
            display_name: "数据转换".to_string(),
            description: "在 JSON / CSV / XML 之间互相转换".to_string(),
            category: "数据处理".to_string(),
            inputs: vec![
                PortDef {
                    label: "input".to_string(),
                    data_type: "string".to_string(),
                    required: true,
                },
            ],
            outputs: vec![
                PortDef {
                    label: "output".to_string(),
                    data_type: "string".to_string(),
                    required: false,
                },
                PortDef {
                    label: "output_json".to_string(),
                    data_type: "object".to_string(),
                    required: false,
                },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "transform": {
                        "type": "string",
                        "enum": ["json_to_csv", "csv_to_json", "json_to_xml", "xml_to_json", "csv_to_xml", "xml_to_csv"],
                        "description": "转换方向"
                    },
                    "root_tag": {
                        "type": "string",
                        "description": "XML 根标签名（json_to_xml 时使用，默认 'root'）"
                    },
                    "item_tag": {
                        "type": "string",
                        "description": "XML 项标签名（json_to_xml 数组时使用，默认 'item'）"
                    },
                    "delimiter": {
                        "type": "string",
                        "description": "CSV 分隔符（默认逗号）"
                    }
                },
                "required": ["transform"]
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
        let transform = config
            .get("transform")
            .and_then(|v| v.as_str())
            .unwrap_or("json_to_csv");

        let input = inputs
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if input.is_empty() {
            return Err(FlowError::ExecutionError(
                "transform: input is empty".to_string(),
            ));
        }

        let root_tag = config
            .get("root_tag")
            .and_then(|v| v.as_str())
            .unwrap_or("root");
        let item_tag = config
            .get("item_tag")
            .and_then(|v| v.as_str())
            .unwrap_or("item");
        let delimiter = config
            .get("delimiter")
            .and_then(|v| v.as_str())
            .unwrap_or(",");

        let (output_str, output_json) = match transform {
            "json_to_csv" => json_to_csv(&input, delimiter)?,
            "csv_to_json" => csv_to_json(&input, delimiter)?,
            "json_to_xml" => json_to_xml(&input, root_tag, item_tag)?,
            "xml_to_json" => xml_to_json(&input)?,
            "csv_to_xml" => {
                let (json_str, json_val) = csv_to_json(&input, delimiter)?;
                json_to_xml(&json_str, root_tag, item_tag)?
            }
            "xml_to_csv" => {
                let (json_str, json_val) = xml_to_json(&input)?;
                json_to_csv(&json_str, delimiter)?
            }
            _ => {
                return Err(FlowError::ExecutionError(format!(
                    "transform: unknown transform type '{}'",
                    transform
                )))
            }
        };

        let mut out = HashMap::new();
        out.insert("output".to_string(), serde_json::Value::String(output_str));
        out.insert("output_json".to_string(), output_json);
        Ok(out)
    }
}

/// JSON array of objects → CSV string
fn json_to_csv(input: &str, delimiter: &str) -> FlowResult<(String, serde_json::Value)> {
    let value: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| FlowError::ExecutionError(format!("json_to_csv: invalid JSON: {}", e)))?;

    let arr = match &value {
        serde_json::Value::Array(a) => a.clone(),
        serde_json::Value::Object(_) => vec![value.clone()],
        _ => {
            return Err(FlowError::ExecutionError(
                "json_to_csv: input must be JSON array or object".to_string(),
            ))
        }
    };

    if arr.is_empty() {
        return Ok(("".to_string(), serde_json::json!([])));
    }

    // Collect all keys from all objects
    let mut headers: Vec<String> = Vec::new();
    for item in &arr {
        if let serde_json::Value::Object(obj) = item {
            for key in obj.keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter.as_bytes().first().copied().unwrap_or(b','))
        .from_writer(Vec::new());

    // Write header
    wtr.write_record(&headers)
        .map_err(|e| FlowError::ExecutionError(format!("json_to_csv: write header: {}", e)))?;

    // Write rows
    for item in &arr {
        let obj = match item {
            serde_json::Value::Object(o) => o,
            _ => continue,
        };
        let row: Vec<String> = headers
            .iter()
            .map(|h| {
                obj.get(h)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();
        wtr.write_record(&row)
            .map_err(|e| FlowError::ExecutionError(format!("json_to_csv: write row: {}", e)))?;
    }

    let csv_str = String::from_utf8(wtr.into_inner()
        .map_err(|e| FlowError::ExecutionError(format!("json_to_csv: flush: {}", e)))?)
        .map_err(|e| FlowError::ExecutionError(format!("json_to_csv: utf8: {}", e)))?;

    Ok((csv_str, value))
}

/// CSV string → JSON array of objects
fn csv_to_json(input: &str, delimiter: &str) -> FlowResult<(String, serde_json::Value)> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter.as_bytes().first().copied().unwrap_or(b','))
        .from_reader(Cursor::new(input));

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| FlowError::ExecutionError(format!("csv_to_json: read headers: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut records = Vec::new();
    for result in rdr.records() {
        let record =
            result.map_err(|e| FlowError::ExecutionError(format!("csv_to_json: read row: {}", e)))?;
        let mut obj = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
            // Try to parse numbers and booleans
            let value = if field.is_empty() {
                serde_json::Value::Null
            } else if let Ok(n) = field.parse::<i64>() {
                serde_json::json!(n)
            } else if let Ok(n) = field.parse::<f64>() {
                serde_json::json!(n)
            } else if field == "true" {
                serde_json::json!(true)
            } else if field == "false" {
                serde_json::json!(false)
            } else {
                serde_json::Value::String(field.to_string())
            };
            obj.insert(key, value);
        }
        records.push(serde_json::Value::Object(obj));
    }

    let json_val = serde_json::Value::Array(records);
    let json_str = serde_json::to_string_pretty(&json_val)
        .map_err(|e| FlowError::ExecutionError(format!("csv_to_json: serialize: {}", e)))?;

    Ok((json_str, json_val))
}

/// JSON → XML string
fn json_to_xml(
    input: &str,
    root_tag: &str,
    item_tag: &str,
) -> FlowResult<(String, serde_json::Value)> {
    let value: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| FlowError::ExecutionError(format!("json_to_xml: invalid JSON: {}", e)))?;

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    json_value_to_xml(&value, root_tag, item_tag, 0, &mut xml);

    Ok((xml, value))
}

fn json_value_to_xml(
    value: &serde_json::Value,
    tag: &str,
    item_tag: &str,
    indent: usize,
    out: &mut String,
) {
    let pad = "  ".repeat(indent);
    match value {
        serde_json::Value::Object(map) => {
            out.push_str(&format!("{}<{}>\n", pad, tag));
            for (key, val) in map {
                json_value_to_xml(val, key, item_tag, indent + 1, out);
            }
            out.push_str(&format!("{}</{}>\n", pad, tag));
        }
        serde_json::Value::Array(arr) => {
            out.push_str(&format!("{}<{}>\n", pad, tag));
            for item in arr {
                json_value_to_xml(item, item_tag, item_tag, indent + 1, out);
            }
            out.push_str(&format!("{}</{}>\n", pad, tag));
        }
        serde_json::Value::String(s) => {
            out.push_str(&format!("{}<{}>{}</{}>\n", pad, tag, escape_xml(s), pad));
        }
        serde_json::Value::Number(n) => {
            out.push_str(&format!("{}<{}>{}</{}>\n", pad, tag, n, pad));
        }
        serde_json::Value::Bool(b) => {
            out.push_str(&format!("{}<{}>{}</{}>\n", pad, tag, b, pad));
        }
        serde_json::Value::Null => {
            out.push_str(&format!("{}<{} />\n", pad, tag));
        }
    }
}

/// XML string → JSON
fn xml_to_json(input: &str) -> FlowResult<(String, serde_json::Value)> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(input);
    let mut stack: Vec<serde_json::Map<String, serde_json::Value>> = Vec::new();
    let mut current_text = String::new();
    let mut root_obj: Option<serde_json::Map<String, serde_json::Value>> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                stack.push(serde_json::Map::new());
                current_text.clear();
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let text = current_text.trim().to_string();
                current_text.clear();

                if let Some(mut obj) = stack.pop() {
                    let value = if obj.is_empty() && !text.is_empty() {
                        // Text-only element
                        if let Ok(n) = text.parse::<i64>() {
                            serde_json::json!(n)
                        } else if let Ok(n) = text.parse::<f64>() {
                            serde_json::json!(n)
                        } else if text == "true" {
                            serde_json::json!(true)
                        } else if text == "false" {
                            serde_json::json!(false)
                        } else {
                            serde_json::Value::String(text)
                        }
                    } else {
                        serde_json::Value::Object(obj)
                    };

                    if let Some(parent) = stack.last_mut() {
                        // Add to parent
                        if let Some(existing) = parent.get_mut(&tag) {
                            // Convert to array if duplicate tag
                            match existing {
                                serde_json::Value::Array(arr) => arr.push(value),
                                _ => {
                                    let old = existing.clone();
                                    *existing = serde_json::Value::Array(vec![old, value]);
                                }
                            }
                        } else {
                            parent.insert(tag, value);
                        }
                    } else {
                        // Root element
                        let mut root = serde_json::Map::new();
                        root.insert(tag, value);
                        root_obj = Some(root);
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                current_text.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    let json_val = root_obj
        .map(serde_json::Value::Object)
        .unwrap_or(serde_json::Value::Null);

    let json_str = serde_json::to_string_pretty(&json_val)
        .map_err(|e| FlowError::ExecutionError(format!("xml_to_json: serialize: {}", e)))?;

    Ok((json_str, json_val))
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_csv() {
        let input = r#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#;
        let (csv, _) = json_to_csv(input, ",").unwrap();
        assert!(csv.contains("name"));
        assert!(csv.contains("age"));
        assert!(csv.contains("Alice"));
        assert!(csv.contains("30"));
    }

    #[test]
    fn test_csv_to_json() {
        let input = "name,age\nAlice,30\nBob,25";
        let (_, json) = csv_to_json(input, ",").unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "Alice");
        assert_eq!(arr[0]["age"], 30);
    }

    #[test]
    fn test_json_to_xml() {
        let input = r#"{"user":{"name":"Alice","age":30}}"#;
        let (xml, _) = json_to_xml(input, "root", "item").unwrap();
        assert!(xml.contains("<root>"));
        assert!(xml.contains("<name>"));
        assert!(xml.contains("Alice"));
        assert!(xml.contains("<age>"));
        assert!(xml.contains("30"));
    }

    #[test]
    fn test_xml_to_json() {
        let input = r#"<?xml version="1.0"?><root><name>Alice</name><age>30</age></root>"#;
        let (_, json) = xml_to_json(input).unwrap();
        assert_eq!(json["root"]["name"], "Alice");
        assert_eq!(json["root"]["age"], 30);
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a<b>&c"), "a&lt;b&gt;&amp;c");
    }
}

