use crate::context::LoomContext;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::{InterceptorChain, InterceptorResult};

#[async_trait::async_trait]
/// Trait per interceptor globali
pub trait ExecutorInterceptor: Send + Sync {
    /// Nome dell'interceptor
    fn name(&self) -> &str;

    /// Descrizione per debug/help
    fn description(&self) -> &str;

    /// Configurazione di default
    fn default_config(&self) -> ExecutorConfig;

    /// Intercetta l'esecuzione (stesso pattern degli interceptor normali)
    async fn intercept<'a>(
        &'a self,
        loom_context: &'a LoomContext,
        context: &'a mut ExecutionContext,
        hook_registry: &'a HookRegistry,
        config: &'a ExecutorConfig,
        next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult;

}
