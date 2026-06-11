//! 调度系统 - Cron 定时执行

use crate::engine::executor::Executor;
use crate::engine::storage::WorkflowStorage;
use crate::error::FlowError;
use crate::nodes::registry::NodeRegistry;
use crate::webbridge::WebBridgeState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 调度任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub name: String,
    pub workflow_id: String,
    pub cron_expr: String,
    pub enabled: bool,
    pub created_at: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub run_count: u64,
}

impl Schedule {
    pub fn new(name: String, workflow_id: String, cron_expr: String) -> Result<Self, FlowError> {
        cron::Schedule::from_str(&cron_expr)
            .map_err(|e| FlowError::ConfigError(format!("Invalid cron expression: {}", e)))?;

        let now = Utc::now().to_rfc3339();
        let next_run = Self::compute_next_run(&cron_expr);

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            workflow_id,
            cron_expr,
            enabled: true,
            created_at: now,
            last_run: None,
            next_run,
            run_count: 0,
        })
    }

    pub fn compute_next_run(cron_expr: &str) -> Option<String> {
        let schedule = cron::Schedule::from_str(cron_expr).ok()?;
        let next = schedule.after(&Utc::now()).next()?;
        Some(next.to_rfc3339())
    }

    pub fn update_cron(&mut self, cron_expr: String) -> Result<(), FlowError> {
        cron::Schedule::from_str(&cron_expr)
            .map_err(|e| FlowError::ConfigError(format!("Invalid cron expression: {}", e)))?;
        self.cron_expr = cron_expr.clone();
        self.next_run = Self::compute_next_run(&cron_expr);
        Ok(())
    }

    pub fn should_run(&self) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(ref next) = self.next_run {
            if let Ok(next_time) = chrono::DateTime::parse_from_rfc3339(next) {
                return Utc::now() >= next_time;
            }
        }
        false
    }

    pub fn mark_executed(&mut self) {
        self.last_run = Some(Utc::now().to_rfc3339());
        self.next_run = Self::compute_next_run(&self.cron_expr);
        self.run_count += 1;
    }
}

/// 调度器状态（可序列化）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchedulerState {
    pub schedules: HashMap<String, Schedule>,
}

/// 调度器
pub struct Scheduler {
    state: Mutex<SchedulerState>,
    state_path: String,
    storage: Arc<WorkflowStorage>,
    node_registry: Arc<NodeRegistry>,
    webbridge: WebBridgeState,
    webhook_store: Arc<Mutex<HashMap<String, Vec<serde_json::Value>>>>,
}

impl Scheduler {
    pub fn new(
        storage: Arc<WorkflowStorage>,
        node_registry: Arc<NodeRegistry>,
        webbridge: WebBridgeState,
        webhook_store: Arc<Mutex<HashMap<String, Vec<serde_json::Value>>>>,
        data_dir: &str,
    ) -> Self {
        let state_path = format!("{}/schedules.json", data_dir);

        let state = if let Ok(content) = std::fs::read_to_string(&state_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            SchedulerState::default()
        };

        Self {
            state: Mutex::new(state),
            state_path,
            storage,
            node_registry,
            webbridge,
            webhook_store,
        }
    }

    fn save_state(&self) -> Result<(), FlowError> {
        let state = self.state.lock()
            .map_err(|e| FlowError::ConfigError(e.to_string()))?;
        let json = serde_json::to_string_pretty(&*state)
            .map_err(|e| FlowError::ConfigError(e.to_string()))?;
        std::fs::write(&self.state_path, json)
            .map_err(|e| FlowError::ConfigError(e.to_string()))?;
        Ok(())
    }

    /// 创建调度
    pub fn create_schedule(&self, name: String, workflow_id: String, cron_expr: String) -> Result<Schedule, FlowError> {
        let schedule = Schedule::new(name, workflow_id, cron_expr)?;
        let mut state = self.state.lock()
            .map_err(|e| FlowError::ConfigError(e.to_string()))?;
        state.schedules.insert(schedule.id.clone(), schedule.clone());
        drop(state);
        self.save_state()?;
        Ok(schedule)
    }

    /// 更新调度
    pub fn update_schedule(&self, id: &str, updates: serde_json::Value) -> Result<Schedule, FlowError> {
        let mut state = self.state.lock()
            .map_err(|e| FlowError::ConfigError(e.to_string()))?;
        let schedule = state.schedules.get_mut(id)
            .ok_or_else(|| FlowError::ConfigError(format!("Schedule not found: {}", id)))?;

        if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
            schedule.name = name.to_string();
        }
        if let Some(cron_expr) = updates.get("cron_expr").and_then(|v| v.as_str()) {
            schedule.update_cron(cron_expr.to_string())?;
        }
        if let Some(enabled) = updates.get("enabled").and_then(|v| v.as_bool()) {
            schedule.enabled = enabled;
            if enabled {
                schedule.next_run = Schedule::compute_next_run(&schedule.cron_expr);
            } else {
                schedule.next_run = None;
            }
        }

        let result = schedule.clone();
        drop(state);
        self.save_state()?;
        Ok(result)
    }

    /// 删除调度
    pub fn delete_schedule(&self, id: &str) -> Result<(), FlowError> {
        let mut state = self.state.lock()
            .map_err(|e| FlowError::ConfigError(e.to_string()))?;
        state.schedules.remove(id)
            .ok_or_else(|| FlowError::ConfigError(format!("Schedule not found: {}", id)))?;
        drop(state);
        self.save_state()?;
        Ok(())
    }

    /// 获取所有调度
    pub fn list_schedules(&self) -> Vec<Schedule> {
        let state = self.state.lock().unwrap();
        state.schedules.values().cloned().collect()
    }

    /// 获取单个调度
    pub fn get_schedule(&self, id: &str) -> Result<Schedule, FlowError> {
        let state = self.state.lock()
            .map_err(|e| FlowError::ConfigError(e.to_string()))?;
        state.schedules.get(id)
            .cloned()
            .ok_or_else(|| FlowError::ConfigError(format!("Schedule not found: {}", id)))
    }

    /// 立即执行调度
    pub async fn trigger_schedule(&self, id: &str) -> Result<serde_json::Value, FlowError> {
        let workflow_id = {
            let state = self.state.lock()
                .map_err(|e| FlowError::ConfigError(e.to_string()))?;
            let schedule = state.schedules.get(id)
                .ok_or_else(|| FlowError::ConfigError(format!("Schedule not found: {}", id)))?;
            schedule.workflow_id.clone()
        };

        let flow = self.storage.load(&workflow_id)?;
        let executor = Executor::new(self.node_registry.clone())
            .with_webbridge(self.webbridge.clone())
            .with_webhook_store(self.webhook_store.clone());

        let result = executor.execute(&flow, None).await?;

        // Update schedule state
        {
            let mut state = self.state.lock()
                .map_err(|e| FlowError::ConfigError(e.to_string()))?;
            if let Some(schedule) = state.schedules.get_mut(id) {
                schedule.mark_executed();
            }
        }
        self.save_state()?;

        let status = if result.failed.is_empty() {
            "completed"
        } else {
            "failed"
        };

        Ok(serde_json::json!({
            "schedule_id": id,
            "workflow_id": workflow_id,
            "status": status,
            "completed": result.completed.len(),
            "executed_at": Utc::now().to_rfc3339(),
        }))
    }

    /// 检查并执行到期的调度任务
    pub async fn tick(&self) {
        let to_run: Vec<(String, String)> = {
            let state = match self.state.lock() {
                Ok(s) => s,
                Err(_) => return,
            };
            state.schedules.iter()
                .filter(|(_, s)| s.should_run())
                .map(|(id, s)| (id.clone(), s.workflow_id.clone()))
                .collect()
        };

        for (schedule_id, workflow_id) in to_run {
            tracing::info!("Executing scheduled workflow: {} ({})", schedule_id, workflow_id);

            match self.storage.load(&workflow_id) {
                Ok(flow) => {
                    let executor = Executor::new(self.node_registry.clone())
                        .with_webbridge(self.webbridge.clone())
                        .with_webhook_store(self.webhook_store.clone());

                    match executor.execute(&flow, None).await {
                        Ok(result) => {
                            let status = if result.failed.is_empty() {
                                "completed"
                            } else {
                                "failed"
                            };
                            tracing::info!("Scheduled workflow {} finished: {}", workflow_id, status);

                            let mut state = self.state.lock().unwrap();
                            if let Some(schedule) = state.schedules.get_mut(&schedule_id) {
                                schedule.mark_executed();
                            }
                            drop(state);
                            let _ = self.save_state();
                        }
                        Err(e) => {
                            tracing::error!("Scheduled workflow {} execution failed: {}", workflow_id, e);
                            // Still mark as executed to avoid retry storm
                            let mut state = self.state.lock().unwrap();
                            if let Some(schedule) = state.schedules.get_mut(&schedule_id) {
                                schedule.mark_executed();
                            }
                            drop(state);
                            let _ = self.save_state();
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to load workflow {} for schedule {}: {}", workflow_id, schedule_id, e);
                }
            }
        }
    }
}
