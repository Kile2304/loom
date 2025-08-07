use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use log::log;
use crate::InputArg;
use crate::interceptor::{ActiveInterceptor, InterceptorChain, InterceptorResult};
use crate::interceptor::context::{ExecutionContext, InterceptorContext};
use crate::interceptor::engine::InterceptorEngine;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::ExecutorInterceptor;

pub struct DefinitionExecutorInterceptor(pub String, pub Vec<ActiveInterceptor>, pub Vec<InputArg>);

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
        mut context: InterceptorContext<'a>,
        // TODO: Queste config mi potrebbero servie a qualcosa in questo livello
        _config: &ExecutorConfig,
        // TODO: Non dovrebbe esistere un NEXT perchè gli executor sono terminali e contengono altri interceptor
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        // TODO: Aggiungere hooks di "inizio", "fine", "success" e "error" definition

        context.loom_context.find_definition(&self.0).as_ref().unwrap().signature
            .args_into_variable(
                context.loom_context,
                context.execution_context.read()
                    .map_err(|_| format!("Couldn't borrow"))?
                    .deref(),
                &self.2
            )?.into_iter()
            .try_for_each::<_, Result<(), String>>(|(variable_name, value)| {
                // context.execution_context.variables.to_mut().insert(variable_name, value);
                context.execution_context.write()
                    .map_err(|_| format!("Couldn't borrow"))?
                    .deref_mut()
                    .variables
                .insert(variable_name, value);
                Ok(())
                // context.execution_context.get_mut().variables.insert(variable_name, value);
            })?;

        // next(context, hook_registry)
        InterceptorEngine::execute_chain(context, &self.1).await
    }

    fn need_chain(&self) -> bool {
        false
    }

}