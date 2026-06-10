//! File operations node — read, write, append, delete, move, list files.
//!
//! All paths are relative to the configured `work_dir` (defaults to "data/").

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct FileNode;

#[async_trait]
impl NodeExecutor for FileNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "file".to_string(),
            display_name: "文件操作".to_string(),
            description: "文件操作：读取、写入、追加、删除、移动、列出目录".to_string(),
            category: "文件".to_string(),
            inputs: vec![
                PortDef { label: "content".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "content".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "files".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
                PortDef { label: "file_path".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["read", "write", "append", "delete", "move", "list"],
                        "default": "read"
                    },
                    "path": {"type": "string"},
                    "content": {"type": "string"},
                    "dest_path": {"type": "string"},
                    "work_dir": {"type": "string", "default": "data"},
                    "max_bytes": {"type": "number", "default": 1048576}
                },
                "required": ["operation", "path"]
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
        let operation = config["operation"].as_str().unwrap_or("read");
        let path = config["path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "path is required".into(),
        })?;
        let work_dir = config["work_dir"].as_str().unwrap_or("data");
        let max_bytes = config["max_bytes"].as_u64().unwrap_or(1_048_576) as usize;

        let base = PathBuf::from(work_dir);
        let canonical_base = std::fs::canonicalize(&base).unwrap_or(base.clone());

        // Resolve path relative to work_dir
        let full_path = base.join(path);
        // Basic path traversal guard
        let canonical_path = std::fs::canonicalize(&full_path).unwrap_or(full_path.clone());
        if !canonical_path.starts_with(&canonical_base) && operation != "write" && operation != "append" {
            return Err(FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: "path traversal detected".into(),
            });
        }

        match operation {
            "read" => read_file(&full_path, node, max_bytes),
            "write" => write_file(&full_path, &config, &inputs, node, false),
            "append" => write_file(&full_path, &config, &inputs, node, true),
            "delete" => delete_file(&full_path, node),
            "move" => move_file(&base, &config, node),
            "list" => list_files(&full_path, node),
            _ => Err(FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: format!("unknown operation: {}", operation),
            }),
        }
    }
}

fn read_file(path: &PathBuf, node: &Node, max_bytes: usize) -> FlowResult<HashMap<String, serde_json::Value>> {
    let metadata = std::fs::metadata(path).map_err(|e| FlowError::NodeExecutionFailed {
        node_id: node.id.clone(),
        detail: format!("stat: {}", e),
    })?;

    if metadata.len() > max_bytes as u64 {
        let content = std::fs::read_to_string(path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("read: {}", e),
        })?;
        let truncated = content.chars().take(max_bytes).collect::<String>();
        let mut outputs = HashMap::new();
        outputs.insert("content".into(), serde_json::json!(truncated + "\n... [truncated]"));
        outputs.insert("success".into(), serde_json::json!(true));
        outputs.insert("file_path".into(), serde_json::json!(path.to_string_lossy().to_string()));
        return Ok(outputs);
    }

    let content = std::fs::read_to_string(path).map_err(|e| FlowError::NodeExecutionFailed {
        node_id: node.id.clone(),
        detail: format!("read: {}", e),
    })?;

    let mut outputs = HashMap::new();
    outputs.insert("content".into(), serde_json::json!(content));
    outputs.insert("success".into(), serde_json::json!(true));
    outputs.insert("file_path".into(), serde_json::json!(path.to_string_lossy().to_string()));
    Ok(outputs)
}

fn write_file(
    path: &PathBuf,
    config: &serde_json::Value,
    inputs: &HashMap<String, serde_json::Value>,
    node: &Node,
    append: bool,
) -> FlowResult<HashMap<String, serde_json::Value>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("mkdir: {}", e),
        })?;
    }

    let content = config["content"].as_str()
        .or_else(|| inputs.get("content").and_then(|v| v.as_str()))
        .unwrap_or("");

    if append {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("open: {}", e),
            })?;
        file.write_all(content.as_bytes()).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("write: {}", e),
        })?;
    } else {
        std::fs::write(path, content).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("write: {}", e),
        })?;
    }

    let mut outputs = HashMap::new();
    outputs.insert("success".into(), serde_json::json!(true));
    outputs.insert("file_path".into(), serde_json::json!(path.to_string_lossy().to_string()));
    Ok(outputs)
}

fn delete_file(path: &PathBuf, node: &Node) -> FlowResult<HashMap<String, serde_json::Value>> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("rmdir: {}", e),
        })?;
    } else {
        std::fs::remove_file(path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("rm: {}", e),
        })?;
    }

    let mut outputs = HashMap::new();
    outputs.insert("success".into(), serde_json::json!(true));
    outputs.insert("file_path".into(), serde_json::json!(path.to_string_lossy().to_string()));
    Ok(outputs)
}

fn move_file(base: &PathBuf, config: &serde_json::Value, node: &Node) -> FlowResult<HashMap<String, serde_json::Value>> {
    let from = config["path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
        node_id: node.id.clone(),
        detail: "path is required for source".into(),
    })?;
    let to = config["dest_path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
        node_id: node.id.clone(),
        detail: "dest_path is required for move".into(),
    })?;

    let from_path = base.join(from);
    let to_path = base.join(to);

    if let Some(parent) = to_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("mkdir: {}", e),
        })?;
    }

    std::fs::rename(&from_path, &to_path).map_err(|e| FlowError::NodeExecutionFailed {
        node_id: node.id.clone(),
        detail: format!("move: {}", e),
    })?;

    let mut outputs = HashMap::new();
    outputs.insert("success".into(), serde_json::json!(true));
    outputs.insert("file_path".into(), serde_json::json!(to_path.to_string_lossy().to_string()));
    Ok(outputs)
}

fn list_files(path: &PathBuf, node: &Node) -> FlowResult<HashMap<String, serde_json::Value>> {
    let entries = std::fs::read_dir(path).map_err(|e| FlowError::NodeExecutionFailed {
        node_id: node.id.clone(),
        detail: format!("readdir: {}", e),
    })?;

    let mut files = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        files.push(serde_json::json!({
            "name": name,
            "is_dir": is_dir,
            "size": size,
        }));
    }

    let mut outputs = HashMap::new();
    outputs.insert("files".into(), serde_json::json!(files));
    outputs.insert("success".into(), serde_json::json!(true));
    outputs.insert("file_path".into(), serde_json::json!(path.to_string_lossy().to_string()));
    Ok(outputs)
}
