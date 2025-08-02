use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub(crate) output: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) metadata: HashMap<String, String>,
}

/// Risultato di un hook
#[derive(Debug, Clone)]
pub enum HookResult {
    Continue,
    ModifyContext { changes: HashMap<String, String> },
    Block { reason: String },
    Retry { max_attempts: u32 },
}