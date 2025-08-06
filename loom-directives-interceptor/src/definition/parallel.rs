use std::collections::HashMap;
use std::sync::Arc;
use loom_core::ast::DirectiveCall;
use loom_core::context::LoomContext;
use loom_core::interceptor::context::{ExecutionContext, InterceptorContext};
use loom_core::interceptor::directive::interceptor::DirectiveInterceptor;
use loom_core::interceptor::{InterceptorChain, InterceptorResult};
use loom_core::types::{LoomValue, ParallelizationKind};

/// Interceptor di direttiva @parallel (priorità DIRECTIVE_NORMAL)
struct ParallelDirectiveInterceptor;

impl ParallelDirectiveInterceptor {
    fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl DirectiveInterceptor for ParallelDirectiveInterceptor {
    fn directive_name(&self) -> &str { "parallel" }

    async fn intercept<'a>(&self, mut context: InterceptorContext<'a>, next: Box<InterceptorChain<'a>>) -> InterceptorResult
    {
        println!("⚡ Parallel: Enabling parallel execution...");
        // context.metadata.insert("parallel".to_string(), "true".to_string());
        context.execution_context.to_mut().parallelization_kind = ParallelizationKind::Parallel { max_thread: 2 };
        next(context).await
    }

    fn parse_parameters(
        &self,
        _loom_context: &LoomContext,
        _execution_context: &ExecutionContext,
        _call: &DirectiveCall
    ) -> Result<HashMap<String, LoomValue>, String> {
        Ok(HashMap::new())
    }

    fn priority(&self) -> i32 { 4000 } // DIRECTIVE_NORMAL range
}