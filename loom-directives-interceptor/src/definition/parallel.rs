use std::collections::HashMap;
use loom_core::ast::DirectiveCall;
use loom_core::context::LoomContext;
use loom_core::interceptor::context::ExecutionContext;
use loom_core::interceptor::directive::interceptor::DirectiveInterceptor;
use loom_core::interceptor::hook::registry::HookRegistry;
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

    async fn intercept<'a>(&'a self, loom_context: &'a LoomContext, context: &'a mut ExecutionContext, _hooks: &'a HookRegistry, next: Box<InterceptorChain<'a>>) -> InterceptorResult
    {
        println!("⚡ Parallel: Enabling parallel execution...");
        // context.metadata.insert("parallel".to_string(), "true".to_string());
        context.parallelization_kind = ParallelizationKind::Parallel { max_thread: 2 };
        next(loom_context, context, _hooks).await
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