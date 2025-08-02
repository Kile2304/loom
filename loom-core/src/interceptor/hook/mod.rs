use std::collections::HashMap;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::result::{ExecutionResult, HookResult};
use crate::interceptor::scope::ExecutionHook;

pub mod registry;

/// Payload generico per gli hook
#[derive(Debug, Clone)]
pub enum HookPayload {
    Command { command: Vec<String> },
    Result { result: ExecutionResult },
    Error { error: String },
    Custom { data: HashMap<String, serde_json::Value> },
}
/// Handler per hook specifici
pub trait HookHandler: Send + Sync {
    fn hook_type(&self) -> ExecutionHook;
    fn handle(&self, context: &mut ExecutionContext, payload: &HookPayload) -> HookResult;
    fn priority(&self) -> i32 { 100 }
}