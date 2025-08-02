use std::collections::HashMap;
use loom_core::ast::DirectiveCall;
use loom_core::context::LoomContext;
use loom_core::interceptor::context::ExecutionContext;
use loom_core::interceptor::directive::interceptor::DirectiveInterceptor;
use loom_core::interceptor::hook::registry::HookRegistry;
use loom_core::interceptor::{InterceptorChain, InterceptorResult};
use loom_core::interceptor::result::ExecutionResult;
use loom_core::types::LoomValue;

struct IfElseDirectiveInterceptor;

#[async_trait::async_trait]
impl DirectiveInterceptor for IfElseDirectiveInterceptor {
    fn directive_name(&self) -> &str {
        "if_else"
    }

    async fn intercept<'a>(&'a self, loom_context: &'a LoomContext, context: &'a mut ExecutionContext, hooks: &'a HookRegistry, next: Box<InterceptorChain<'a>>) -> InterceptorResult {
        todo!()
    }

    fn parse_parameters(&self, loom_context: &LoomContext, execution_context: &ExecutionContext, call: &DirectiveCall) -> Result<HashMap<String, LoomValue>, String> {
        todo!()
    }
}