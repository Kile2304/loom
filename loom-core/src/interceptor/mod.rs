use std::pin::Pin;
use crate::context::LoomContext;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::directive::ActiveDirectiveInterceptor;
use crate::interceptor::directive::interceptor::DirectiveInterceptor;
use crate::interceptor::executor::ActiveExecutorInterceptor;
use crate::interceptor::global::ActiveGlobalInterceptor;
use crate::interceptor::global::interceptor::GlobalInterceptor;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::result::ExecutionResult;

pub mod context;
pub mod result;
pub mod directive;
pub mod global;
pub mod scope;
pub mod engine;
pub mod hook;
pub mod executor;
pub mod priority;

/// **LoomContext**:        The general context with every, enum, definition, variable...
/// **ExecutionContext**:   The context for the current execution, it's mutable.
/// **HookRegistry**:       The registry with all the hooks.
pub type InterceptorChain<'a> = dyn FnOnce(&'a LoomContext, &'a mut ExecutionContext, &'a HookRegistry)
    -> Pin<Box<dyn Future<Output = Result<ExecutionResult, String>> + Send + 'a>> + Send + 'a;

pub type InterceptorResult = Result<ExecutionResult, String>;


/// Enum unificato per l'execution chain
#[derive(Clone)]
pub enum ActiveInterceptor {
    Global(ActiveGlobalInterceptor),
    Directive(ActiveDirectiveInterceptor),
    Executor(ActiveExecutorInterceptor),
}

impl ActiveInterceptor {
    pub fn priority(&self) -> i32 {
        match self {
            Self::Global(global) => global.config.priority,
            Self::Directive(directive) => directive.priority,
            Self::Executor(_) => i32::MAX,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Global(global) => &global.name,
            Self::Directive(directive) => &directive.name,
            Self::Executor(executor) => &executor.name,
        }
    }

    pub fn interceptor_type(&self) -> &str {
        match self {
            Self::Global(_) => "global",
            Self::Directive(_) => "directive",
            Self::Executor(_) => "executor",
        }
    }

    pub fn sort(a: &ActiveInterceptor, b: &ActiveInterceptor) -> std::cmp::Ordering {
        b.priority().cmp(&a.priority())
    }
}

#[macro_export]
macro_rules! interceptor_result {
    ($expr:expr) => {
        Box::pin(std::future::ready($expr)) as std::pin::Pin<Box<dyn Future<Output = Result<$crate::interceptor::result::ExecutionResult, String>> + Send>>
    };

    // Con lifetime
    ($expr:expr, $lifetime:lifetime) => {
        Box::pin(std::future::ready($expr)) as std::pin::Pin<Box<dyn Future<Output = Result<$crate::interceptor::result::ExecutionResult, String>> + Send + $lifetime>>
    };
}