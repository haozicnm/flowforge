//! Workflow data model.
//!
//! This is the canonical structure. Dart models must match this exactly.

use serde::{Deserialize, Serialize};

/// A complete workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub variables: Vec<Variable>,
    #[serde(default)]
    pub owner_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Workflow {
    /// Create a new workflow with auto-generated ID.
    #[allow(dead_code)]
    pub fn new(name: String, description: Option<String>) -> Self {
        Self::with_owner(name, description, None)
    }

    /// Create a new workflow with an owner.
    pub fn with_owner(name: String, description: Option<String>, owner_id: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: description.unwrap_or_default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            variables: Vec::new(),
            owner_id,
            created_at: chrono::Utc::now(),
        }
    }

    /// Get nodes (direct access).
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Get edges (direct access).
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
}

/// A single node in the workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique ID (e.g., "http_1", "shell_2"). Used as variable root key.
    pub id: String,

    /// Node type (e.g., "http", "shell", "condition").
    #[serde(rename = "type")]
    pub node_type: String,

    /// Human-readable label for display. NOT used for data flow.
    #[serde(default)]
    pub label: String,

    /// Node-specific configuration.
    #[serde(default)]
    pub config: serde_json::Value,

    /// Position on canvas (for UI).
    #[serde(default)]
    pub position: Position,
}

/// Position on the canvas.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// An edge connecting two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID.
    pub from: String,

    /// Source port label (NOT id — label is the data key).
    pub from_port: String,

    /// Target node ID.
    pub to: String,

    /// Target port label.
    pub to_port: String,
}

/// A workflow-level variable (user-defined).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: serde_json::Value,
    #[serde(default)]
    pub description: String,
}
