use std::collections::HashMap;
use crate::interceptor::result::ExecutionResult;
use crate::interceptor::scope::ExecutionTarget;
use crate::types::{LoomValue, ParallelizationKind};

/// Execution context for runtime
#[derive(Debug, Default)]
pub struct ExecutionContext {
    // TODO: Valutare,     variables: Cow<'a, HashMap<String, LoomValue>>,
    pub variables: HashMap<String, LoomValue>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub dry_run: bool,
    pub target: ExecutionTarget,
    pub parallelization_kind: ParallelizationKind,
    pub metadata: HashMap<String, String>,
    pub previous_result: Option<ExecutionResult>,
}

impl ExecutionContext {
    pub fn get_variable(&self, name: &str) -> Option<LoomValue> {
        self.variables.get(name).map(|it| it.clone())
    }
}