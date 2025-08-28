use std::collections::HashMap;
use std::sync::Arc;
use crate::ast::DirectiveCall;
use crate::context::LoomContext;
use crate::error::LoomResult;
use crate::interceptor::context::{ExecutionContext, InterceptorContext};
use crate::interceptor::{InterceptorChain, InterceptorResult};
use crate::types::LoomValue;

#[async_trait::async_trait]
pub trait DirectiveInterceptor: Send + Sync {
    fn directive_name(&self) -> &str;

    /// Intercetta con accesso al hook registry
    async fn intercept<'a>(
        &'a self,
        context: InterceptorContext<'a>,
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
    ) -> LoomResult<HashMap<String, LoomValue>>;

    fn priority(&self) -> i32 { 100 }

    fn need_chain(&self) -> bool;

}