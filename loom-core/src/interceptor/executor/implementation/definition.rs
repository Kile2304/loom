use std::sync::Arc;
use crate::interceptor::{ActiveInterceptor, InterceptorChain, InterceptorResult};
use crate::interceptor::context::InterceptorContext;
use crate::interceptor::engine::InterceptorEngine;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::ExecutorInterceptor;

pub struct DefinitionExecutorInterceptor(pub Vec<ActiveInterceptor>);

#[async_trait::async_trait]
impl ExecutorInterceptor for DefinitionExecutorInterceptor {
    fn name(&self) -> &str {
        "definition"
    }
    fn description(&self) -> &str {
        "Esegue una definition"
    }
    fn default_config(&self) -> ExecutorConfig {
        ExecutorConfig::default()
    }
    async fn intercept<'a>(
        &'a self,
        context: InterceptorContext<'a>,
        // TODO: Queste config mi potrebbero servie a qualcosa in questo livello
        _config: &ExecutorConfig,
        // TODO: Non dovrebbe esistere un NEXT perchè gli executor sono terminali e contengono altri interceptor
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        // TODO: Aggiungere hooks di "inizio", "fine", "success" e "error" definition
        // next(context, hook_registry)
        InterceptorEngine::execute_chain(context, &self.0).await
    }

}