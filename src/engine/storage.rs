//! Workflow persistence — file-based storage.
//!
//! Stores workflows as JSON files in a data directory.
//! Each workflow is a single file: {data_dir}/workflows/{id}.json

use std::path::{Path, PathBuf};

use crate::engine::workflow::Workflow;
use crate::error::{FlowError, FlowResult};

/// File-based workflow storage.
pub struct WorkflowStorage {
    data_dir: PathBuf,
}

impl WorkflowStorage {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let dir = data_dir.into().join("workflows");
        Self { data_dir: dir }
    }

    /// Initialize the storage directory.
    pub fn init(&self) -> FlowResult<()> {
        std::fs::create_dir_all(&self.data_dir).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to create data dir: {}", e),
        })
    }

    /// List all saved workflows.
    pub fn list(&self) -> FlowResult<Vec<Workflow>> {
        let mut workflows = Vec::new();

        let entries = std::fs::read_dir(&self.data_dir).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to read data dir: {}", e),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| FlowError::StorageError {
                detail: format!("Failed to read entry: {}", e),
            })?;

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                match self.load_from_path(&path) {
                    Ok(wf) => workflows.push(wf),
                    Err(e) => {
                        tracing::warn!("Skipping corrupted workflow {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Sort by creation time (newest first)
        workflows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(workflows)
    }

    /// Load a workflow by ID.
    pub fn load(&self, id: &str) -> FlowResult<Workflow> {
        let path = self.data_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Err(FlowError::WorkflowNotFound(id.to_string()));
        }
        self.load_from_path(&path)
    }

    /// Save a workflow (create or update).
    pub fn save(&self, workflow: &Workflow) -> FlowResult<()> {
        self.init()?;
        let path = self.data_dir.join(format!("{}.json", workflow.id));
        let json = serde_json::to_string_pretty(workflow).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to serialize workflow: {}", e),
        })?;
        std::fs::write(&path, json).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to write workflow: {}", e),
        })?;
        Ok(())
    }

    /// Delete a workflow by ID.
    pub fn delete(&self, id: &str) -> FlowResult<()> {
        let path = self.data_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Err(FlowError::WorkflowNotFound(id.to_string()));
        }
        std::fs::remove_file(&path).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to delete workflow: {}", e),
        })?;
        Ok(())
    }

    fn load_from_path(&self, path: &Path) -> FlowResult<Workflow> {
        let content =
            std::fs::read_to_string(path).map_err(|e| FlowError::StorageError {
                detail: format!("Failed to read file: {}", e),
            })?;
        serde_json::from_str(&content).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to parse workflow: {}", e),
        })
    }
}
