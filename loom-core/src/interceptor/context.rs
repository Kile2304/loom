use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use crate::context::LoomContext;
use crate::event::channel::ExecutionEventChannel;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::scope::{ExecutionActivity, ExecutionScope};
use crate::types::{LoomValue, ParallelizationKind};

/// Execution context for runtime
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    // TODO: Valutare,     variables: Cow<'a, HashMap<String, LoomValue>>,
    pub variables: HashMap<Arc<str>, LoomValue>,
    // pub variables: Cow<'a, HashMap<String, LoomValue>>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub dry_run: bool,
    pub scope: ExecutionScope,
    pub parallelization_kind: ParallelizationKind,
    pub metadata: HashMap<String, String>,
}


impl ExecutionContext {
    pub fn get_variable(&self, name: &str) -> Option<LoomValue> {
        self.variables.get(name).map(|it| it.clone())
    }
}

#[derive(Clone)]
pub struct InterceptorContext<'a> {
    pub loom_context: &'a LoomContext,
    pub execution_context: Arc<RwLock<ExecutionContext>>,
    pub hook_registry: &'a HookRegistry,
    pub channel: ExecutionEventChannel,
}

// impl<'a> Clone for InterceptorContext<'a> {
//     fn clone(&self) -> Self {
//         Self {
//             loom_context: self.loom_context,
//             execution_context: self.execution_context.to_owned(),
//             hook_registry: self.hook_registry,
//             channel: self.channel.clone(),
//         }
//     }
// }