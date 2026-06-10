//! Unified error types for FlowForge.
//!
//! Rule: NO unwrap() anywhere in the codebase. All errors flow through here.

use thiserror::Error;

/// Top-level error type for all FlowForge operations.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum FlowError {
    // ── Workflow errors ──
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Invalid workflow YAML: {0}")]
    InvalidYaml(String),

    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: String },

    // ── Node errors ──
    #[error("Node type not found: {0}")]
    NodeTypeNotFound(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Node {node_id}: {message}")]
    NodeError { node_id: String, message: String },

    #[error("Node {node_id}: invalid config — {detail}")]
    InvalidNodeConfig { node_id: String, detail: String },

    // ── Variable errors ──
    #[error("Undefined variable: {{{{{ref_expr}}}}}")]
    UndefinedVariable { ref_expr: String },

    #[error("Variable resolution failed for node {node_id}: {reason}")]
    VariableResolutionFailed { node_id: String, reason: String },

    #[error("Variable not found in node {node_id}: {var_ref}")]
    VariableNotFound { node_id: String, var_ref: String },

    // ── Execution errors ──
    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Execution timeout after {seconds}s for node {node_id}")]
    ExecutionTimeout { node_id: String, seconds: u64 },

    #[error("Execution aborted: {reason}")]
    ExecutionAborted { reason: String },

    // ── Storage errors ──
    #[error("Storage error: {detail}")]
    StorageError { detail: String },

    // ── Node execution errors ──
    #[error("Node {node_id} execution failed: {detail}")]
    NodeExecutionFailed { node_id: String, detail: String },

    // ── IO errors ──
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // ── JSON errors ──
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type alias.
pub type FlowResult<T> = Result<T, FlowError>;
