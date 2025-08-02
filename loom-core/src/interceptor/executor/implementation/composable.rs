use crate::context::LoomContext;
use crate::interceptor::{ActiveInterceptor, InterceptorChain, InterceptorResult};
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::engine::InterceptorEngine;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::ExecutorInterceptor;
use crate::interceptor::executor::implementation::empty_execute_intercept_next;
use crate::interceptor::hook::registry::HookRegistry;

pub struct SequenceChainInterceptor(pub Vec<ActiveInterceptor>);

#[async_trait::async_trait]
impl ExecutorInterceptor for SequenceChainInterceptor {
    fn name(&self) -> &str {
        "SequenceChainInterceptor"
    }
    fn description(&self) -> &str {
        "SequenceChainInterceptor"
    }
    fn default_config(&self) -> ExecutorConfig {
        ExecutorConfig::default()
    }
    async fn intercept<'a>(
        &'a self,
        loom_context: &'a LoomContext,
        context: &'a mut ExecutionContext,
        hook_registry: &'a HookRegistry,
        _config: &'a ExecutorConfig,
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        InterceptorEngine::execute_chain(loom_context, context, hook_registry, &self.0).await
    }
}

pub struct SequentialExecutorInterceptor(pub Vec<ActiveInterceptor>, pub String);

#[async_trait::async_trait]
impl ExecutorInterceptor for SequentialExecutorInterceptor {
    fn name(&self) -> &str {
        self.1.as_str()
    }
    fn description(&self) -> &str {
        "SequentialExecutorInterceptor"
    }
    fn default_config(&self) -> ExecutorConfig {
        ExecutorConfig::default()
    }
    async fn intercept<'a>(
        &'a self,
        loom_context: &'a LoomContext,
        context: &'a mut ExecutionContext,
        hook_registry: &'a HookRegistry,
        config: &'a ExecutorConfig,
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        for interceptor in &self.0 {
            match interceptor {
                ActiveInterceptor::Executor(executor) => {
                    let result = executor.interceptor.intercept(loom_context, context, hook_registry, config, empty_execute_intercept_next()).await?;
                    context.previous_result = Some(result);
                }
                _ => {
                    Err("SequentialExecutor should contain only executor Interceptor".to_string())?;
                }
            }
        }
        context.previous_result.take().ok_or("The result of a SequentialExecutor should not be None".to_string())
        // InterceptorEngine::execute_chain(loom_context, context, hook_registry, &self.0)
    }
}

fn launch_interceptor(
    interceptors: &Vec<ActiveInterceptor>,
) {
    
}