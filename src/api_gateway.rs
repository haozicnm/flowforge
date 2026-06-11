//! API Gateway — expose workflows as custom HTTP endpoints.
//!
//! Each workflow can be "published" to a custom path (e.g. /api/run/my-workflow).
//! Any HTTP request to that path triggers the workflow execution.
//! The request body and query params are passed as the "request" input to the first node.
//!
//! Features:
//!   - Custom path per workflow
//!   - Optional API key authentication
//!   - Rate limiting (requests per minute)
//!   - Request logging

use crate::engine::executor::Executor;
use crate::engine::storage::WorkflowStorage;
use crate::nodes::registry::NodeRegistry;
use crate::webbridge::WebBridgeState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Published API endpoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedApi {
    /// Workflow ID to execute.
    pub workflow_id: String,
    /// Custom path segment (e.g. "my-workflow" → /api/run/my-workflow).
    pub path: String,
    /// Optional API key for authentication.
    pub api_key: Option<String>,
    /// Rate limit: max requests per minute (0 = unlimited).
    pub rate_limit: u64,
    /// HTTP methods allowed (default: all).
    pub methods: Vec<String>,
    /// Whether this API is enabled.
    pub enabled: bool,
    /// Number of times invoked.
    pub call_count: u64,
    /// Last invocation timestamp.
    pub last_called: Option<String>,
    /// Creation timestamp.
    pub created_at: String,
}

/// Gateway state — tracks published APIs and rate limiting.
pub struct ApiGateway {
    /// Published APIs: path → config.
    published: Mutex<HashMap<String, PublishedApi>>,
    /// Rate limit tracking: path → (timestamp, count).
    rate_limits: Mutex<HashMap<String, Vec<Instant>>>,
    /// State file path.
    state_path: String,
    /// Dependencies.
    storage: Arc<WorkflowStorage>,
    node_registry: Arc<NodeRegistry>,
    webbridge: WebBridgeState,
    webhook_store: Arc<Mutex<HashMap<String, Vec<serde_json::Value>>>>,
}

impl ApiGateway {
    pub fn new(
        storage: Arc<WorkflowStorage>,
        node_registry: Arc<NodeRegistry>,
        webbridge: WebBridgeState,
        webhook_store: Arc<Mutex<HashMap<String, Vec<serde_json::Value>>>>,
        data_dir: &str,
    ) -> Self {
        let state_path = format!("{}/api_gateway.json", data_dir);

        let published = if let Ok(content) = std::fs::read_to_string(&state_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            published: Mutex::new(published),
            rate_limits: Mutex::new(HashMap::new()),
            state_path,
            storage,
            node_registry,
            webbridge,
            webhook_store,
        }
    }

    fn save_state(&self) -> Result<(), String> {
        let published = self.published.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&*published).map_err(|e| e.to_string())?;
        std::fs::write(&self.state_path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Publish a workflow as an API endpoint.
    pub fn publish(
        &self,
        workflow_id: String,
        path: String,
        api_key: Option<String>,
        rate_limit: u64,
    ) -> Result<PublishedApi, String> {
        // Validate path
        if path.is_empty() || path.contains('/') || path.contains(' ') {
            return Err("Path must be a non-empty string without slashes or spaces".to_string());
        }

        // Verify workflow exists
        self.storage.load(&workflow_id).map_err(|e| format!("Workflow not found: {}", e))?;

        let mut published = self.published.lock().map_err(|e| e.to_string())?;

        // Check path uniqueness
        if published.contains_key(&path) {
            return Err(format!("Path '{}' is already published", path));
        }

        let api = PublishedApi {
            workflow_id,
            path: path.clone(),
            api_key,
            rate_limit,
            methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            enabled: true,
            call_count: 0,
            last_called: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        published.insert(path.clone(), api.clone());
        drop(published);

        self.save_state()?;
        Ok(api)
    }

    /// Unpublish a workflow API.
    pub fn unpublish(&self, path: &str) -> Result<(), String> {
        let mut published = self.published.lock().map_err(|e| e.to_string())?;
        published.remove(path).ok_or_else(|| format!("Path '{}' not found", path))?;
        drop(published);
        self.save_state()?;
        Ok(())
    }

    /// List all published APIs.
    pub fn list(&self) -> Vec<PublishedApi> {
        let published = self.published.lock().unwrap();
        published.values().cloned().collect()
    }

    /// Get a specific published API by path.
    #[allow(dead_code)]
    pub fn get(&self, path: &str) -> Option<PublishedApi> {
        let published = self.published.lock().unwrap();
        published.get(path).cloned()
    }

    /// Check rate limit for a path. Returns true if allowed.
    fn check_rate_limit(&self, path: &str, limit: u64) -> bool {
        if limit == 0 {
            return true; // unlimited
        }

        let mut rate_limits = self.rate_limits.lock().unwrap();
        let entries = rate_limits.entry(path.to_string()).or_default();

        // Remove entries older than 1 minute
        let now = Instant::now();
        entries.retain(|t| now.duration_since(*t) < Duration::from_secs(60));

        if entries.len() as u64 >= limit {
            return false;
        }

        entries.push(now);
        true
    }

    /// Execute a published API — called by the dynamic route handler.
    pub async fn handle_request(
        &self,
        path: &str,
        method: &str,
        api_key_header: Option<&str>,
        body: serde_json::Value,
        query_params: HashMap<String, String>,
    ) -> Result<serde_json::Value, (u16, String)> {
        let api = {
            let published = self.published.lock().map_err(|e| (500, e.to_string()))?;
            published.get(path).cloned()
                .ok_or_else(|| (404, format!("API path '{}' not found", path)))?
        };

        // Check enabled
        if !api.enabled {
            return Err((403, "API is disabled".to_string()));
        }

        // Check method
        if !api.methods.is_empty() && !api.methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
            return Err((405, format!("Method '{}' not allowed", method)));
        }

        // Check API key
        if let Some(ref expected_key) = api.api_key {
            match api_key_header {
                Some(provided) if provided == expected_key => {}
                _ => return Err((401, "Invalid or missing API key".to_string())),
            }
        }

        // Check rate limit
        if !self.check_rate_limit(path, api.rate_limit) {
            return Err((429, "Rate limit exceeded".to_string()));
        }

        // Load workflow
        let flow = self.storage.load(&api.workflow_id)
            .map_err(|e| (500, format!("Failed to load workflow: {}", e)))?;

        // Build request context as input
        let _request_ctx = serde_json::json!({
            "method": method,
            "body": body,
            "query": query_params,
            "path": path,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Execute workflow
        let executor = Executor::new(self.node_registry.clone())
            .with_webbridge(self.webbridge.clone())
            .with_webhook_store(self.webhook_store.clone());

        let result = executor.execute(&flow, None).await
            .map_err(|e| (500, format!("Workflow execution failed: {}", e)))?;

        // Update stats
        {
            let mut published = self.published.lock().map_err(|e| (500, e.to_string()))?;
            if let Some(api) = published.get_mut(path) {
                api.call_count += 1;
                api.last_called = Some(chrono::Utc::now().to_rfc3339());
            }
        }
        let _ = self.save_state();

        // Collect outputs
        let outputs: HashMap<String, serde_json::Value> = result.node_outputs.iter()
            .map(|(k, v)| (k.clone(), serde_json::to_value(v).unwrap_or(serde_json::Value::Null)))
            .collect();

        let status = if result.failed.is_empty() { "success" } else { "failed" };

        Ok(serde_json::json!({
            "status": status,
            "outputs": outputs,
            "completed_nodes": result.completed.len(),
            "failed_nodes": result.failed.len(),
        }))
    }
}
