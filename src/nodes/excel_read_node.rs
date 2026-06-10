//! Excel Read node — read data from .xlsx/.xls files using calamine.
use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ExcelReadNode;

#[async_trait]
impl NodeExecutor for ExcelReadNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "excel_read".to_string(),
            display_name: "读取 Excel".to_string(),
            description: "从 Excel 文件读取数据".to_string(),
            category: "Excel".to_string(),
            inputs: vec![
                PortDef { label: "path".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "rows".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "headers".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "count".to_string(), data_type: "number".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to Excel file" },
                    "sheet": { "type": "string", "description": "Sheet name (default: first sheet)" },
                    "range": { "type": "string", "description": "Cell range like A1:C10 (default: used range)" },
                    "has_header": { "type": "boolean", "default": true },
                    "max_rows": { "type": "number", "default": 10000 }
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
        use calamine::{Reader, open_workbook_auto};

        let path = config["path"].as_str()
            .or_else(|| inputs.get("path").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "excel_read".to_string(),
                detail: "path is required".to_string(),
            })?;

        let has_header = config["has_header"].as_bool().unwrap_or(true);
        let max_rows = config["max_rows"].as_u64().unwrap_or(10000) as usize;

        let mut workbook = open_workbook_auto(path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "excel_read".to_string(),
            detail: format!("failed to open Excel file '{}': {}", path, e),
        })?;

        let sheet_name = config["sheet"].as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                workbook.sheet_names().first().cloned().unwrap_or_default()
            });

        let range = workbook.worksheet_range(&sheet_name).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "excel_read".to_string(),
            detail: format!("failed to read sheet '{}': {}", sheet_name, e),
        })?;

        let rows_vec: Vec<Vec<serde_json::Value>> = range.rows()
            .take(max_rows)
            .map(|row| {
                row.iter().map(cell_to_json).collect()
            })
            .collect();

        let (headers, data_rows) = if has_header && rows_vec.len() > 1 {
            let h: Vec<serde_json::Value> = rows_vec[0].clone();
            let d: Vec<Vec<serde_json::Value>> = rows_vec[1..].to_vec();
            (h, d)
        } else {
            (vec![], rows_vec)
        };

        let count = data_rows.len();

        // Convert rows to objects if headers are available
        let result = if has_header && !headers.is_empty() {
            let header_strs: Vec<String> = headers.iter()
                .map(|h| match h {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect();
            let objects: Vec<serde_json::Value> = data_rows.iter()
                .map(|row| {
                    let mut obj = serde_json::Map::new();
                    for (i, val) in row.iter().enumerate() {
                        let key = header_strs.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
                        obj.insert(key, val.clone());
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();
            serde_json::json!(objects)
        } else {
            serde_json::json!(data_rows)
        };

        tracing::info!("Excel: read {} rows from '{}' sheet '{}'", count, path, sheet_name);

        let mut outputs = HashMap::new();
        outputs.insert("rows".to_string(), result);
        outputs.insert("headers".to_string(), serde_json::json!(headers));
        outputs.insert("count".to_string(), serde_json::json!(count));
        Ok(outputs)
    }
}

fn cell_to_json(cell: &calamine::Data) -> serde_json::Value {
    match cell {
        calamine::Data::Empty => serde_json::Value::Null,
        calamine::Data::String(s) => serde_json::json!(s),
        calamine::Data::Float(f) => serde_json::json!(f),
        calamine::Data::Int(i) => serde_json::json!(i),
        calamine::Data::Bool(b) => serde_json::json!(b),
        calamine::Data::Error(_) => serde_json::Value::Null,
        calamine::Data::DateTime(dt) => serde_json::json!(dt.to_string()),
        _ => serde_json::Value::Null,
    }
}
