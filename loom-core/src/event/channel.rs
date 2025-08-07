use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Channel per comunicare eventi durante l'esecuzione
#[derive(Debug, Clone)]
pub struct ExecutionEventChannel {
    pub execution_id: Arc<String>,
    pub sender: mpsc::UnboundedSender<ExecutionEvent>,
}

impl ExecutionEventChannel {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ExecutionEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let execution_id = Uuid::new_v4().to_string();

        (Self { execution_id: Arc::new(execution_id), sender }, receiver)
    }

    pub fn emit(&self, event: ExecutionEvent) -> Result<(), String> {
        self.sender.send(event)
            .map_err(|_| "Failed to send execution event".to_string())
    }

    pub fn emit_with_context(&self, kind: ExecutionEventKind, metadata: HashMap<String, String>) -> Result<(), String> {
        let event = ExecutionEvent {
            id: Uuid::new_v4().to_string(),
            execution_id: self.execution_id.to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_millis() as u64,
            kind,
            metadata,
        };
        self.emit(event)
    }
}

/// Eventi di esecuzione che possono essere emessi durante il workflow
#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    pub id: String,
    pub execution_id: String,
    pub timestamp: u64,
    pub kind: ExecutionEventKind,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ExecutionEventKind {
    // Lifecycle Events
    ExecutionStarted {
        definition_name: String,
        definition_type: String,
    },
    ExecutionCompleted {
        definition_name: String,
        success: bool,
        duration_ms: u64,
    },
    ExecutionFailed {
        definition_name: String,
        error: String,
        duration_ms: u64,
    },

    // Command Events
    CommandStarted {
        command: String,
        working_dir: Option<String>,
    },
    CommandCompleted {
        command: String,
        exit_code: Option<i32>,
        duration_ms: u64,
        output_lines: usize,
    },
    CommandFailed {
        command: String,
        error: String,
        exit_code: Option<i32>,
        duration_ms: u64,
    },

    // Interceptor Events
    InterceptorTriggered {
        interceptor_name: String,
        interceptor_type: String, // "global", "directive", "executor"
        priority: i32,
    },
    InterceptorCompleted {
        interceptor_name: String,
        duration_ms: u64,
        success: bool,
    },

    // Directive Events
    DirectiveEvaluated {
        directive_name: String,
        parameters: HashMap<String, String>,
        result: String,
    },

    // Pipeline/Job Events (per il futuro)
    StageStarted {
        stage_name: String,
        pipeline_name: String,
    },
    StageCompleted {
        stage_name: String,
        pipeline_name: String,
        success: bool,
        duration_ms: u64,
    },
    JobStarted {
        job_name: String,
        stage_name: Option<String>,
    },
    JobCompleted {
        job_name: String,
        success: bool,
        duration_ms: u64,
    },

    // Hook Events
    HookTriggered {
        hook_type: String,
        handler_name: String,
        payload_type: String,
    },

    // Progress Events
    ProgressUpdate {
        current_step: usize,
        total_steps: usize,
        current_task: String,
        percentage: f32,
    },

    // Resource Events
    ResourceUsage {
        cpu_percent: f32,
        memory_mb: u64,
        disk_io_mb: u64,
    },

    // Custom Events (per plugin e user code)
    Custom {
        event_type: String,
        data: serde_json::Value,
    },

    // Debug/Diagnostics
    VariableResolved {
        variable_name: String,
        value: String,
        scope: String,
    },
    ExpressionEvaluated {
        expression: String,
        result: String,
        evaluation_time_ms: u64,
    },
}

impl ExecutionEvent {
    pub fn is_error(&self) -> bool {
        matches!(self.kind,
            ExecutionEventKind::ExecutionFailed { .. } |
            ExecutionEventKind::CommandFailed { .. }
        )
    }

    pub fn is_lifecycle(&self) -> bool {
        matches!(self.kind,
            ExecutionEventKind::ExecutionStarted { .. } |
            ExecutionEventKind::ExecutionCompleted { .. } |
            ExecutionEventKind::ExecutionFailed { .. }
        )
    }

    pub fn duration(&self) -> Option<u64> {
        match &self.kind {
            ExecutionEventKind::ExecutionCompleted { duration_ms, .. } => Some(*duration_ms),
            ExecutionEventKind::ExecutionFailed { duration_ms, .. } => Some(*duration_ms),
            ExecutionEventKind::CommandCompleted { duration_ms, .. } => Some(*duration_ms),
            ExecutionEventKind::CommandFailed { duration_ms, .. } => Some(*duration_ms),
            ExecutionEventKind::InterceptorCompleted { duration_ms, .. } => Some(*duration_ms),
            ExecutionEventKind::StageCompleted { duration_ms, .. } => Some(*duration_ms),
            ExecutionEventKind::JobCompleted { duration_ms, .. } => Some(*duration_ms),
            ExecutionEventKind::ExpressionEvaluated { evaluation_time_ms, .. } => Some(*evaluation_time_ms),
            _ => None,
        }
    }
}

// Utility per filtering/aggregation eventi
pub struct EventFilter {
    pub execution_ids: Option<Vec<String>>,
    pub event_types: Option<Vec<String>>,
    pub time_range: Option<(u64, u64)>,
    pub only_errors: bool,
}

impl EventFilter {
    pub fn matches(&self, event: &ExecutionEvent) -> bool {
        if let Some(ref ids) = self.execution_ids {
            if !ids.contains(&event.execution_id) {
                return false;
            }
        }

        if let Some(ref types) = self.event_types {
            let event_type = std::mem::discriminant(&event.kind);
            // Simplified type checking - in real impl you'd want proper type names
            // if !types.contains(&format!("{:?}", event_type)) {
            //     return false;
            // }
        }

        if let Some((start, end)) = self.time_range {
            if event.timestamp < start || event.timestamp > end {
                return false;
            }
        }

        if self.only_errors && !event.is_error() {
            return false;
        }

        true
    }
}