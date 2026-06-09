//! Node trait — the "building block" interface.
//!
//! Every node type implements this trait. The trait defines:
//! - What inputs the node accepts
//! - What outputs it produces
//! - How to execute it
//!
//! Rule: Nodes do NOT resolve their own variables. The executor calls
//! resolver::resolve_node_config() before passing config to execute().

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::FlowResult;
use crate::engine::workflow::Node;

/// Definition of a node type (metadata for the UI and registry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTypeDef {
    /// Unique type name (e.g., "http", "shell", "condition").
    pub type_name: String,

    /// Human-readable display name.
    pub display_name: String,

    /// Description.
    pub description: String,

    /// Category for the node palette.
    pub category: String,

    /// Input port definitions.
    pub inputs: Vec<PortDef>,

    /// Output port definitions.
    pub outputs: Vec<PortDef>,

    /// JSON Schema for the node's config field.
    pub config_schema: serde_json::Value,
}

/// Definition of an input or output port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    /// Port label. This is the key used in variable references: {{nodeId.portLabel}}.
    pub label: String,

    /// Data type hint (e.g., "string", "number", "object", "any").
    #[serde(default = "default_port_type")]
    pub data_type: String,

    /// Whether this port is required.
    #[serde(default)]
    pub required: bool,
}

fn default_port_type() -> String {
    "any".to_string()
}

/// The core trait that all nodes must implement.
#[async_trait]
pub trait NodeExecutor: Send + Sync {
    /// Return the type definition for this node.
    fn type_def(&self) -> NodeTypeDef;

    /// Execute this node with resolved config.
    ///
    /// `resolved_config` has already been through the variable resolver.
    /// Do NOT attempt to resolve variables yourself.
    ///
    /// `inputs` contains the data from input ports (from upstream nodes).
    ///
    /// Returns a map of port_label → output_value.
    async fn execute(
        &self,
        node: &Node,
        resolved_config: serde_json::Value,
        inputs: std::collections::HashMap<String, serde_json::Value>,
    ) -> FlowResult<std::collections::HashMap<String, serde_json::Value>>;
}
