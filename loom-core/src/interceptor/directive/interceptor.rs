use std::collections::HashMap;
use crate::ast::DirectiveCall;
use crate::context::LoomContext;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::{InterceptorChain, InterceptorResult};
use crate::types::LoomValue;

#[async_trait::async_trait]
pub trait DirectiveInterceptor: Send + Sync {
    fn directive_name(&self) -> &str;

    /// Intercetta con accesso al hook registry
    async fn intercept<'a>(
        &'a self,
        loom_context: &'a LoomContext,
        context: &'a mut ExecutionContext,
        hooks: &'a HookRegistry,
        next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult;

    // TODO: Mancano gli arg della signature in input
    // L'evaluation dei parametri delle directive, viene fatto in fase di creazione degli interceptor,
    // Quindi, l'ExecutionContext sarebbe vuoto
    fn parse_parameters(
        &self,
        loom_context: &LoomContext,
        execution_context: &ExecutionContext,
        call: &DirectiveCall
    ) -> Result<HashMap<String, LoomValue>, String>;

    fn priority(&self) -> i32 { 100 }

}