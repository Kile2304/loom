use std::sync::Arc;
use crate::error::LoomError;
use crate::interceptor::{ActiveInterceptor, InterceptorChain, InterceptorResult};
use crate::interceptor::context::InterceptorContext;
use crate::interceptor::engine::InterceptorEngine;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::ExecutorInterceptor;
use crate::interceptor::executor::implementation::empty_execute_intercept_next;
use crate::interceptor::result::ExecutionResult;

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
        context: InterceptorContext<'a>,
        _config: &ExecutorConfig,
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        InterceptorEngine::execute_chain(context, &self.0).await
    }

    fn need_chain(&self) -> bool {
        false
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
        context: InterceptorContext<'a>,
        config: &ExecutorConfig,
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        let mut result: Option<ExecutionResult> = None;
        for interceptor in &self.0 {
            match interceptor {
                ActiveInterceptor::Executor(executor) => {
                    result = Some(executor.interceptor.intercept(context.clone(), config, empty_execute_intercept_next()).await?);
                }
                _ => {
                    Err("SequentialExecutor should contain only executor Interceptor".to_string())?;
                }
            }
        }
        result.ok_or(LoomError::execution("The result of a SequentialExecutor should not be None".to_string()))
        // context.execution_context.previous_result.take().ok_or("The result of a SequentialExecutor should not be None".to_string())
        // InterceptorEngine::execute_chain(loom_context, context, hook_registry, &self.0)
    }

    fn need_chain(&self) -> bool {
        false
    }
}