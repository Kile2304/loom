use std::collections::HashMap;
use std::sync::Arc;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::hook::{HookHandler, HookPayload};
use crate::interceptor::result::HookResult;
use crate::interceptor::scope::ExecutionHook;

// TODO: Aggiungere meglio hook a interceptor e vedere se necessario una interazione finale una volta finita la chain.

/// Registry per hook handlers
pub struct HookRegistry {
    handlers: HashMap<ExecutionHook, Vec<Arc<dyn HookHandler>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_hook(&mut self, handler: Arc<dyn HookHandler>) {
        let hook_type = handler.hook_type();
        self.handlers
            .entry(hook_type.clone())
            .or_insert_with(Vec::new)
            .push(handler);

        // Ordina per priorità
        if let Some(handlers) = self.handlers.get_mut(&hook_type) {
            handlers.sort_by(|a, b| b.priority().cmp(&a.priority()));
        }
    }

    pub fn execute_hooks(
        &self,
        hook_type: ExecutionHook,
        context: &mut ExecutionContext,
        payload: &HookPayload,
    ) -> Result<(), String> {
        if let Some(handlers) = self.handlers.get(&hook_type) {
            for handler in handlers {
                match handler.handle(context, payload) {
                    HookResult::Continue => continue,
                    HookResult::ModifyContext { changes } => {
                        for (key, value) in changes {
                            context.metadata.insert(key, value);
                        }
                    }
                    HookResult::Block { reason } => {
                        return Err(reason);
                    }
                    HookResult::Retry { max_attempts } => {
                        context.metadata.insert("retry_max".to_string(), max_attempts.to_string());
                    }
                }
            }
        }
        Ok(())
    }
}
