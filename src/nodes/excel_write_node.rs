//! Excel Write node — write data to .xlsx files using umya-spreadsheet.
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ExcelWriteNode;

#[async_trait]
impl NodeExecutor for ExcelWriteNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "excel_write".to_string(),
            display_name: "写入 Excel".to_string(),
            description: "将数据写入 Excel 文件".to_string(),
            category: "Excel".to_string(),
            inputs: vec![
                PortDef { label: "data".to_string(), data_type: "array".to_string(), required: true },
                PortDef { label: "path".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "path".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "rows_written".to_string(), data_type: "number".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Output file path" },
                    "sheet": { "type": "string", "default": "Sheet1" },
                    "write_headers": { "type": "boolean", "default": true },
                    "mode": {
                        "type": "string",
                        "enum": ["create", "append"],
                        "default": "create"
                    }
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
        use umya_spreadsheet::*;

        let path = config["path"].as_str()
            .or_else(|| inputs.get("path").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "excel_write".to_string(),
                detail: "path is required".to_string(),
            })?;

        let sheet_name = config["sheet"].as_str().unwrap_or("Sheet1");
        let write_headers = config["write_headers"].as_bool().unwrap_or(true);
        let mode = config["mode"].as_str().unwrap_or("create");

        let data = inputs.get("data")
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: "excel_write".to_string(),
                detail: "data input is required".to_string(),
            })?;

        let arr = data.as_array().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: "excel_write".to_string(),
            detail: "data must be an array".to_string(),
        })?;

        // Load or create workbook
        let mut book = if mode == "append" && Path::new(path).exists() {
            reader::xlsx::read(path).map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "excel_write".to_string(),
                detail: format!("failed to open for append: {}", e),
            })?
        } else {
            new_file()
        };

        // Get or create sheet
        let sheet = if let Some(s) = book.get_sheet_by_name_mut(sheet_name) {
            s
        } else {
            let _idx = book.get_sheet_count();
            book.new_sheet(sheet_name).map_err(|_| FlowError::NodeExecutionFailed {
                node_id: "excel_write".to_string(),
                detail: "failed to create sheet".to_string(),
            })?;
            book.get_sheet_by_name_mut(sheet_name).unwrap()
        };

        let mut row_num: u32 = 1;
        let mut rows_written = 0u32;

        // Collect headers from first object
        let headers: Vec<String> = if let Some(first) = arr.first() {
            if let Some(obj) = first.as_object() {
                obj.keys().cloned().collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Write headers
        if write_headers && !headers.is_empty() {
            for (col, header) in headers.iter().enumerate() {
                let cell = sheet.get_cell_mut(((col as u32) + 1, row_num));
                cell.set_value(header.as_str());
            }
            row_num += 1;
        }

        // Write data rows
        for item in arr {
            if let Some(obj) = item.as_object() {
                for (col, header) in headers.iter().enumerate() {
                    let val = obj.get(header).unwrap_or(&serde_json::Value::Null);
                    let cell = sheet.get_cell_mut(((col as u32) + 1, row_num));
                    {
                        let s = match val {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
                            serde_json::Value::Null => String::new(),
                            other => other.to_string(),
                        };
                        cell.set_value(s);
                    }
                }
            } else {
                // Array of arrays
                if let Some(arr_vals) = item.as_array() {
                    for (col, val) in arr_vals.iter().enumerate() {
                        let cell = sheet.get_cell_mut(((col as u32) + 1, row_num));
                        {
                            let s = match val {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                other => other.to_string(),
                            };
                            cell.set_value(s);
                        }
                    }
                }
            }
            row_num += 1;
            rows_written += 1;
        }

        // Save
        writer::xlsx::write(&book, path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: "excel_write".to_string(),
            detail: format!("failed to save: {}", e),
        })?;

        tracing::info!("Excel: wrote {} rows to '{}'", rows_written, path);

        let mut outputs = HashMap::new();
        outputs.insert("path".to_string(), serde_json::json!(path));
        outputs.insert("rows_written".to_string(), serde_json::json!(rows_written));
        Ok(outputs)
    }
}
