use std::collections::HashMap;
use std::sync::Arc;
use loom_core::ast::DirectiveCall;
use loom_core::context::LoomContext;
use loom_core::definition::{ArgDefinition, ParameterDefinition};
use loom_core::error::LoomResult;
use loom_core::interceptor::context::{ExecutionContext, InterceptorContext};
use loom_core::interceptor::directive::interceptor::DirectiveInterceptor;
use loom_core::interceptor::{InterceptorChain, InterceptorResult};
use loom_core::{bool_param, params};
use loom_core::definition::directive::definition::DirectiveDefinition;
use loom_core::definition::directive::scope::DirectiveScope;
use loom_core::types::LoomValue;

// @if(cond == other)
struct IfDirectiveInterceptor;

#[async_trait::async_trait]
impl DirectiveInterceptor for IfDirectiveInterceptor {
    fn directive_name(&self) -> &str {
        "if"
    }

    async fn intercept<'a>(&'a self, _context: InterceptorContext<'a>, _next: Box<InterceptorChain<'a>>) -> InterceptorResult {
        todo!()
    }

    fn parse_parameters(&self, _loom_context: &LoomContext, _execution_context: &ExecutionContext, _call: &DirectiveCall) -> LoomResult<HashMap<String, LoomValue>> {
        todo!()
    }
    fn need_chain(&self) -> bool {
        false
    }
}

impl DirectiveDefinition for IfDirectiveInterceptor {
    fn name(&self) -> &str {
        "if"
    }

    fn description(&self) -> &str {
        "Conditionally execute blocks of commands"
    }

    fn scope(&self) -> &[DirectiveScope] {
        &[DirectiveScope::Block]
    }

    fn parameters(&self) -> Vec<ParameterDefinition> {
        params![
            bool_param!(
                "condition",
                description => "Condizione per eseguire il blocco a cui è collegata la direttiva",
                positional_only
            )
        ]
    }

    fn validate_parameters(&self, args: &[ArgDefinition]) -> LoomResult<()> {
        if args.len() > 1 {

        } else if args.len() == 0 {

        } else {

        }
        todo!()
    }

}

// @else
struct ElseDirectiveInterceptor;

#[async_trait::async_trait]
impl DirectiveInterceptor for ElseDirectiveInterceptor {
    fn directive_name(&self) -> &str {
        "else"
    }

    async fn intercept<'a>(&'a self, context: InterceptorContext<'a>, next: Box<InterceptorChain<'a>>) -> InterceptorResult {
        todo!()
    }

    fn parse_parameters(&self, loom_context: &LoomContext, execution_context: &ExecutionContext, call: &DirectiveCall) -> LoomResult<HashMap<String, LoomValue>> {
        todo!()
    }

    fn need_chain(&self) -> bool {
        false
    }
}