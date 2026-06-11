//! Database query node — executes SQL against SQLite databases.
//!
//! Supports: SELECT (returns rows array), INSERT/UPDATE/DELETE (returns affected_rows).
//! Targets a local SQLite file path. Connection pool not needed — creates temp connection per execution.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct DatabaseNode;

#[async_trait]
impl NodeExecutor for DatabaseNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "database".to_string(),
            display_name: "数据库查询".to_string(),
            description: "执行 SQL 查询 (SQLite)。支持 SELECT / INSERT / UPDATE / DELETE".to_string(),
            category: "数据处理".to_string(),
            inputs: vec![
                PortDef { label: "params".to_string(), data_type: "array".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "rows".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "affected_rows".to_string(), data_type: "number".to_string(), required: false },
                PortDef { label: "column_names".to_string(), data_type: "array".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "db_path": {"type": "string", "description": "SQLite 数据库文件路径"},
                    "query": {"type": "string", "description": "要执行的 SQL 查询"}
                },
                "required": ["db_path", "query"]
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let db_path = config["db_path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "db_path is required".into(),
        })?;

        let query = config["query"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "query is required".into(),
        })?;

        let conn = rusqlite::Connection::open(db_path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("Failed to open database: {}", e),
        })?;

        let q_upper = query.trim().to_uppercase();

        if q_upper.starts_with("SELECT") || q_upper.starts_with("PRAGMA") {
            let mut stmt = conn.prepare(query).map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("Prepare: {}", e),
            })?;

            let column_names: Vec<String> = stmt
                .column_names()
                .iter()
                .map(|c| c.to_string())
                .collect();

            let rows_result = stmt.query_map([], |row| {
                let mut map = serde_json::Map::new();
                for (i, col) in column_names.iter().enumerate() {
                    let val: rusqlite::Result<String> = row.get(i);
                    map.insert(col.clone(), serde_json::json!(val.unwrap_or_default()));
                }
                Ok(serde_json::Value::Object(map))
            }).map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("Query: {}", e),
            })?;

            let mut rows = Vec::new();
            for row in rows_result {
                if let Ok(val) = row {
                    rows.push(val);
                }
            }

            let mut outputs = HashMap::new();
            outputs.insert("rows".into(), serde_json::json!(rows));
            outputs.insert("column_names".into(), serde_json::json!(column_names));
            outputs.insert("affected_rows".into(), serde_json::json!(rows.len()));
            Ok(outputs)
        } else {
            let affected = conn.execute(query, []).map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("Execute: {}", e),
            })?;

            let mut outputs = HashMap::new();
            outputs.insert("rows".into(), serde_json::json!([]));
            outputs.insert("column_names".into(), serde_json::json!([]));
            outputs.insert("affected_rows".into(), serde_json::json!(affected));
            Ok(outputs)
        }
    }
}
