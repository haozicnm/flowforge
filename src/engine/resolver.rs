//! Unified variable resolver — THE SINGLE LAYER.
//!
//! This is the most critical module in FlowForge. The old project had three
//! layers of variable resolution (global resolve, container-internal resolve,
//! Rhai engine), each with different rules. That caused variables to not expand
//! in containers, action.id ≠ action.label data断流, and countless bugs.
//!
//! Rule: ALL nodes (including containers) go through this ONE resolver.
//! No exceptions. No "skip resolve for containers" hacks.
//!
//! ## Variable Reference Format
//!
//! - Storage: `{{stepId.portLabel}}` — step.id as root, port label as key
//! - Display: `步骤名 › 端口名` — human-friendly
//! - These are SEPARATE. Never mix them.
//!
//! ## Two-Phase Resolution
//!
//! Phase 1: Scan config for `{{...}}` patterns, replace with safe placeholders.
//!          This prevents type corruption during deserialization.
//!
//! Phase 2: After deserialization, replace placeholders with actual values.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use crate::engine::workflow::{Node, Variable};
use crate::error::{FlowError, FlowResult};

/// A resolved variable reference.
#[derive(Debug, Clone)]
pub struct ResolvedRef {
    /// The step ID (root key).
    pub step_id: String,
    /// The port label (sub key).
    pub port_label: String,
    /// The placeholder that replaced this reference in the config.
    pub _placeholder: String,
}

/// The result of Phase 1 (placeholder insertion).
#[derive(Debug)]
pub struct PlaceholderMap {
    /// placeholder → (step_id, port_label)
    pub map: HashMap<String, (String, String)>,
}

/// Regex to match `{{stepId.portLabel}}` patterns outside of code fences.
static VAR_REF_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*\.[a-zA-Z0-9_]+)\}\}").expect("invalid regex")
});

/// Regex to match `${env.VAR_NAME}` patterns (environment variables).
static ENV_VAR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{env\.([a-zA-Z_][a-zA-Z0-9_]*)\}").expect("invalid env regex")
});

/// Regex to match `${var_name}` patterns (workflow global variables).
/// Note: does NOT match `${env.*}` which is handled separately.
static GLOBAL_VAR_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").expect("invalid global regex")
});

/// Phase 1: Scan a JSON value for variable references and replace them with placeholders.
///
/// Returns the modified value and a map of placeholders to their (step_id, port_label).
pub fn phase1_insert_placeholders(
    value: &serde_json::Value,
) -> (serde_json::Value, PlaceholderMap) {
    let mut map = HashMap::new();
    let mut counter = 0;
    let modified = replace_refs_recursive(value, &mut map, &mut counter);
    (modified, PlaceholderMap { map })
}

/// Phase 2: Replace placeholders with actual values from step outputs.
///
/// This runs AFTER deserialization, so types are preserved.
pub fn phase2_resolve_placeholders(
    value: &serde_json::Value,
    placeholders: &PlaceholderMap,
    step_outputs: &HashMap<String, serde_json::Value>,
) -> FlowResult<serde_json::Value> {
    resolve_recursive(value, placeholders, step_outputs)
}

/// High-level: resolve all variables in a node's config.
///
/// This is the ONLY function that nodes should call for variable resolution.
/// Do NOT implement your own resolve logic.
pub fn resolve_node_config(
    node: &Node,
    step_outputs: &HashMap<String, serde_json::Value>,
) -> FlowResult<serde_json::Value> {
    resolve_node_config_with_context(node, step_outputs, &[], true)
}

/// Resolve all variables with full context: step outputs + workflow variables + env vars.
///
/// - `step_outputs`: outputs from upstream nodes (`{{stepId.portLabel}}`)
/// - `workflow_vars`: workflow-level variables (`${var_name}`)
/// - `resolve_env`: whether to resolve `${env.VAR_NAME}` from process environment
pub fn resolve_node_config_with_context(
    node: &Node,
    step_outputs: &HashMap<String, serde_json::Value>,
    workflow_vars: &[Variable],
    resolve_env: bool,
) -> FlowResult<serde_json::Value> {
    // Phase 1: insert placeholders for {{stepId.portLabel}}
    let (intermediate, placeholders) = phase1_insert_placeholders(&node.config);

    // Phase 2: resolve step output placeholders
    let mut resolved = phase2_resolve_placeholders(&intermediate, &placeholders, step_outputs)?;

    // Phase 3: resolve ${env.VAR_NAME} and ${var_name}
    resolve_extra_vars(&mut resolved, workflow_vars, resolve_env)?;

    Ok(resolved)
}

/// Phase 3: Resolve `${env.VAR_NAME}` and `${var_name}` patterns in-place.
fn resolve_extra_vars(
    value: &mut serde_json::Value,
    workflow_vars: &[Variable],
    resolve_env: bool,
) -> FlowResult<()> {
    match value {
        serde_json::Value::String(s) => {
            let mut result = s.clone();

            // Resolve ${env.VAR_NAME} first (more specific pattern)
            if resolve_env {
                for cap in ENV_VAR_RE.captures_iter(s.clone().as_str()) {
                    let full_match = &cap[0];
                    let var_name = &cap[1];
                    if let Ok(val) = std::env::var(var_name) {
                        result = result.replace(full_match, &val);
                    }
                }
            }

            // Resolve ${var_name} from workflow variables
            for cap in GLOBAL_VAR_RE.captures_iter(s.clone().as_str()) {
                let full_match = &cap[0];
                let var_name = &cap[1];
                // Skip ${env.*} — already handled above
                if full_match.starts_with("${env.") {
                    continue;
                }
                if let Some(wf_var) = workflow_vars.iter().find(|v| v.name == var_name) {
                    let val_str = match &wf_var.value {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    result = result.replace(full_match, &val_str);
                }
            }

            *s = result;
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                resolve_extra_vars(v, workflow_vars, resolve_env)?;
            }
        }
        serde_json::Value::Object(obj) => {
            for v in obj.values_mut() {
                resolve_extra_vars(v, workflow_vars, resolve_env)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Extract all variable references from a JSON value.
///
/// Returns a list of (step_id, port_label) pairs found in the config.
pub fn extract_refs(value: &serde_json::Value) -> Vec<ResolvedRef> {
    let mut refs = Vec::new();
    extract_refs_recursive(value, &mut refs);
    refs
}

// ── Internal helpers ──

fn extract_refs_recursive(value: &serde_json::Value, refs: &mut Vec<ResolvedRef>) {
    match value {
        serde_json::Value::String(s) => {
            for cap in VAR_REF_RE.captures_iter(s) {
                let ref_expr = &cap[1];
                let parts: Vec<&str> = ref_expr.splitn(2, '.').collect();
                if parts.len() == 2 {
                    refs.push(ResolvedRef {
                        step_id: parts[0].to_string(),
                        port_label: parts[1].to_string(),
                        _placeholder: String::new(),
                    });
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                extract_refs_recursive(v, refs);
            }
        }
        serde_json::Value::Object(obj) => {
            for v in obj.values() {
                extract_refs_recursive(v, refs);
            }
        }
        _ => {}
    }
}

fn replace_refs_recursive(
    value: &serde_json::Value,
    map: &mut HashMap<String, (String, String)>,
    counter: &mut usize,
) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            let mut result = s.clone();
            for cap in VAR_REF_RE.captures_iter(s) {
                let full_match = &cap[0];
                let ref_expr = &cap[1];
                let parts: Vec<&str> = ref_expr.splitn(2, '.').collect();

                if parts.len() == 2 {
                    let placeholder = format!("__FF_VAR_{}__", counter);
                    *counter += 1;

                    result = result.replace(full_match, &placeholder);
                    map.insert(
                        placeholder,
                        (parts[0].to_string(), parts[1].to_string()),
                    );
                }
            }
            serde_json::Value::String(result)
        }
        serde_json::Value::Array(arr) => {
            let new_arr: Vec<_> = arr
                .iter()
                .map(|v| replace_refs_recursive(v, map, counter))
                .collect();
            serde_json::Value::Array(new_arr)
        }
        serde_json::Value::Object(obj) => {
            let new_obj: serde_json::Map<_, _> = obj
                .iter()
                .map(|(k, v)| (k.clone(), replace_refs_recursive(v, map, counter)))
                .collect();
            serde_json::Value::Object(new_obj)
        }
        other => other.clone(),
    }
}

fn resolve_recursive(
    value: &serde_json::Value,
    placeholders: &PlaceholderMap,
    step_outputs: &HashMap<String, serde_json::Value>,
) -> FlowResult<serde_json::Value> {
    match value {
        serde_json::Value::String(s) => {
            let mut result = s.clone();

            for (placeholder, (step_id, port_label)) in &placeholders.map {
                if result.contains(placeholder.as_str()) {
                    let output_key = format!("{}.{}", step_id, port_label);
                    let resolved_value = step_outputs.get(&output_key).ok_or_else(|| {
                        FlowError::UndefinedVariable {
                            ref_expr: output_key.clone(),
                        }
                    })?;

                    // If the string is EXACTLY one placeholder, preserve the raw type
                    if result == *placeholder {
                        return Ok(resolved_value.clone());
                    }

                    // Otherwise, replace in string (serialize non-strings)
                    let replacement = match resolved_value {
                        serde_json::Value::String(s) => s.clone(),
                        other => serde_json::to_string(other).unwrap_or_default(),
                    };
                    result = result.replace(placeholder.as_str(), &replacement);
                }
            }

            Ok(serde_json::Value::String(result))
        }
        serde_json::Value::Array(arr) => {
            let new_arr: Vec<_> = arr
                .iter()
                .map(|v| resolve_recursive(v, placeholders, step_outputs))
                .collect::<FlowResult<_>>()?;
            Ok(serde_json::Value::Array(new_arr))
        }
        serde_json::Value::Object(obj) => {
            let new_obj: serde_json::Map<_, _> = obj
                .iter()
                .map(|(k, v)| {
                    resolve_recursive(v, placeholders, step_outputs)
                        .map(|resolved| (k.clone(), resolved))
                })
                .collect::<FlowResult<_>>()?;
            Ok(serde_json::Value::Object(new_obj))
        }
        other => Ok(other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_phase1_simple_string() {
        let config = json!({"url": "{{http1.response}}"});
        let (modified, map) = phase1_insert_placeholders(&config);

        assert_eq!(modified["url"], "__FF_VAR_0__");
        assert_eq!(map.map.len(), 1);
        assert_eq!(
            map.map["__FF_VAR_0__"],
            ("http1".to_string(), "response".to_string())
        );
    }

    #[test]
    fn test_phase1_multiple_refs() {
        let config = json!({
            "method": "POST",
            "url": "{{api.endpoint}}",
            "body": {"user": "{{auth.username}}"}
        });
        let (modified, map) = phase1_insert_placeholders(&config);

        assert_eq!(map.map.len(), 2);
        assert_eq!(modified["method"], "POST"); // unchanged
        assert!(modified["url"].as_str().unwrap().starts_with("__FF_VAR_"));
    }

    #[test]
    fn test_phase2_resolve_string() {
        let config = json!({"url": "{{http1.response}}"});
        let (modified, placeholders) = phase1_insert_placeholders(&config);

        let mut step_outputs = HashMap::new();
        step_outputs.insert(
            "http1.response".to_string(),
            json!("https://example.com"),
        );

        let resolved =
            phase2_resolve_placeholders(&modified, &placeholders, &step_outputs).unwrap();
        assert_eq!(resolved["url"], "https://example.com");
    }

    #[test]
    fn test_phase2_preserves_type() {
        // When a string is EXACTLY one placeholder, the raw value type is preserved
        let config = json!({"count": "{{step1.num}}"});
        let (modified, placeholders) = phase1_insert_placeholders(&config);

        let mut step_outputs = HashMap::new();
        step_outputs.insert("step1.num".to_string(), json!(42));

        let resolved =
            phase2_resolve_placeholders(&modified, &placeholders, &step_outputs).unwrap();
        // Should be 42 (number), not "42" (string)
        assert_eq!(resolved["count"], 42);
    }

    #[test]
    fn test_phase2_partial_string_replacement() {
        // When a placeholder is part of a larger string, it's replaced as text
        let config = json!({"url": "https://api.com/{{api.path}}/data"});
        let (modified, placeholders) = phase1_insert_placeholders(&config);

        let mut step_outputs = HashMap::new();
        step_outputs.insert("api.path".to_string(), json!("v2"));

        let resolved =
            phase2_resolve_placeholders(&modified, &placeholders, &step_outputs).unwrap();
        assert_eq!(resolved["url"], "https://api.com/v2/data");
    }

    #[test]
    fn test_no_variables_passes_through() {
        let config = json!({"url": "https://example.com", "timeout": 30});
        let (modified, map) = phase1_insert_placeholders(&config);

        assert_eq!(map.map.len(), 0);
        assert_eq!(modified, config);
    }

    #[test]
    fn test_extract_refs() {
        let config = json!({
            "url": "{{http1.response}}",
            "body": {"user": "{{auth.username}}"},
            "static": "no ref here"
        });
        let refs = extract_refs(&config);
        assert_eq!(refs.len(), 2);
        // Order may vary, so check both exist
        let ref_strs: Vec<String> = refs.iter().map(|r| format!("{}.{}", r.step_id, r.port_label)).collect();
        assert!(ref_strs.contains(&"http1.response".to_string()));
        assert!(ref_strs.contains(&"auth.username".to_string()));
    }
}

    #[test]
    fn test_resolve_env_var() {
        // Set a test env var
        std::env::set_var("FLOWFORGE_TEST_VAR", "hello_env");

        let config = serde_json::json!({"url": "${env.FLOWFORGE_TEST_VAR}/data"});
        let node = crate::engine::workflow::Node {
            id: "test".to_string(),
            node_type: "http".to_string(),
            label: "Test".to_string(),
            config,
            position: Default::default(),
        };

        let resolved = resolve_node_config(&node, &HashMap::new()).unwrap();
        assert_eq!(resolved["url"], "hello_env/data");

        std::env::remove_var("FLOWFORGE_TEST_VAR");
    }

    #[test]
    fn test_resolve_global_var() {
        use crate::engine::workflow::Variable;

        let config = serde_json::json!({"endpoint": "${api_url}/v1"});
        let node = crate::engine::workflow::Node {
            id: "test".to_string(),
            node_type: "http".to_string(),
            label: "Test".to_string(),
            config,
            position: Default::default(),
        };

        let vars = vec![Variable {
            name: "api_url".to_string(),
            value: serde_json::json!("https://api.example.com"),
            description: String::new(),
        }];

        let resolved = resolve_node_config_with_context(&node, &HashMap::new(), &vars, false).unwrap();
        assert_eq!(resolved["endpoint"], "https://api.example.com/v1");
    }

    #[test]
    fn test_resolve_mixed_vars() {
        use crate::engine::workflow::Variable;

        // Mix of step output, global var, and static text
        let config = serde_json::json!({
            "url": "${api_url}/users/{{http1.response}}",
            "timeout": "${timeout}"
        });
        let node = crate::engine::workflow::Node {
            id: "test".to_string(),
            node_type: "http".to_string(),
            label: "Test".to_string(),
            config,
            position: Default::default(),
        };

        let mut step_outputs = HashMap::new();
        step_outputs.insert("http1.response".to_string(), serde_json::json!("123"));

        let vars = vec![
            Variable {
                name: "api_url".to_string(),
                value: serde_json::json!("https://api.example.com"),
                description: String::new(),
            },
            Variable {
                name: "timeout".to_string(),
                value: serde_json::json!(30),
                description: String::new(),
            },
        ];

        let resolved = resolve_node_config_with_context(&node, &step_outputs, &vars, false).unwrap();
        assert_eq!(resolved["url"], "https://api.example.com/users/123");
        assert_eq!(resolved["timeout"], "30"); // replaced as string
    }
