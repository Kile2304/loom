use crate::context::LoomContext;
use crate::interceptor::{ActiveInterceptor, InterceptorChain, InterceptorResult};
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::engine::InterceptorEngine;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::ExecutorInterceptor;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::result::ExecutionResult;

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
        loom_context: &'a LoomContext,
        context: &'a mut ExecutionContext,
        hook_registry: &'a HookRegistry,
        // TODO: Queste config mi potrebbero servie a qualcosa in questo livello
        _config: &'a ExecutorConfig,
        // TODO: Non dovrebbe esistere un NEXT perchè gli executor sono terminali e contengono altri interceptor
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        // TODO: Aggiungere hooks di "inizio", "fine", "success" e "error" definition
        // next(context, hook_registry)
        InterceptorEngine::execute_chain(loom_context, context, hook_registry, &self.0).await
    }

}