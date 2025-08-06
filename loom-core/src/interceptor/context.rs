use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use crate::context::LoomContext;
use crate::event::channel::ExecutionEventChannel;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::scope::ExecutionTarget;
use crate::types::{LoomValue, ParallelizationKind};

/// Execution context for runtime
#[derive(Debug, Default, Clone)]
pub struct ExecutionContext<'a> {
    // TODO: Valutare,     variables: Cow<'a, HashMap<String, LoomValue>>,
    // pub variables: HashMap<String, LoomValue>,
    pub variables: Cow<'a, HashMap<String, LoomValue>>,
    pub env_vars: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub dry_run: bool,
    pub target: ExecutionTarget,
    pub parallelization_kind: ParallelizationKind,
    pub metadata: HashMap<String, String>,
}


impl<'a> ExecutionContext<'a> {
    pub fn get_variable(&self, name: &str) -> Option<LoomValue> {
        self.variables.get(name).map(|it| it.clone())
    }
}

#[derive(Clone)]
pub struct InterceptorContext<'a> {
    pub loom_context: &'a LoomContext,
    pub execution_context: Cow<'a, ExecutionContext<'a>>,
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