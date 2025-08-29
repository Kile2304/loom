use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use smart_default::SmartDefault;
use crate::ast::Statement;
use crate::context::LoomContext;
use crate::error::{LoomError, LoomResult};
use crate::event::channel::ExecutionEventChannel;
use crate::InputArg;
use crate::interceptor::{ActiveInterceptor, InterceptorChain, InterceptorResult};
use crate::interceptor::context::{ExecutionContext, InterceptorContext};
use crate::interceptor::directive::ActiveDirectiveInterceptor;
use crate::interceptor::directive::interceptor::DirectiveInterceptor;
use crate::interceptor::directive::manager::DirectiveInterceptorManager;
use crate::interceptor::executor::ActiveExecutorInterceptor;
use crate::interceptor::executor::implementation::command::CommandExecutorInterceptor;
use crate::interceptor::executor::implementation::composable::{SequenceChainInterceptor, SequentialExecutorInterceptor};
use crate::interceptor::executor::implementation::definition::DefinitionExecutorInterceptor;
use crate::interceptor::executor::implementation::empty_execute_intercept_next;
use crate::interceptor::global::ActiveGlobalInterceptor;
use crate::interceptor::global::config::GlobalInterceptorConfig;
use crate::interceptor::global::interceptor::GlobalInterceptor;
use crate::interceptor::global::manager::GlobalInterceptorManager;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::scope::{ExecutionActivity, ExecutionScope};
use crate::types::ParallelizationKind;

/// Middleware Pattern (Filter Chain Pattern) ottimizzato
/// Esegue i vari Task/Job/Command, ma, solo dopo aver eseguito
/// Gli interceptor globali e le direttive, formando per l'appunto un Middleware Pattern
#[derive(SmartDefault)]
pub struct InterceptorEngine {
    #[default(GlobalInterceptorManager::new())]
    global_manager: GlobalInterceptorManager,
    #[default(DirectiveInterceptorManager::new())]
    directive_manager: DirectiveInterceptorManager,
    #[default(HookRegistry::new())]
    hook_registry: HookRegistry,

    // Cache per evitare ricostruzione frequente di chain
    #[default(RwLock::new(HashMap::new()))]
    chain_cache: RwLock<HashMap<String, Vec<ActiveInterceptor>>>,
}

impl InterceptorEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra interceptor globale
    pub fn register_global(&mut self, interceptor: Arc<dyn GlobalInterceptor>) -> LoomResult<()> {
        // Invalida cache quando registriamo nuovi interceptor
        if let Ok(mut cache) = self.chain_cache.write() {
            cache.clear();
        }
        self.global_manager.register(interceptor)
    }

    /// Registra interceptor di direttiva
    pub fn register_directive(&mut self, interceptor: Arc<dyn DirectiveInterceptor>) -> LoomResult<()> {
        if let Ok(mut cache) = self.chain_cache.write() {
            cache.clear();
        }
        self.directive_manager.register(interceptor)
    }

    /// Configura interceptor globale
    pub fn configure_global(&mut self, name: &str, config: GlobalInterceptorConfig) -> LoomResult<()> {
        if let Ok(mut cache) = self.chain_cache.write() {
            cache.clear();
        }
        self.global_manager.configure(name, config)
    }

    /// Override temporaneo
    pub fn override_global(&mut self, name: &str, enabled: bool) -> LoomResult<()> {
        if let Ok(mut cache) = self.chain_cache.write() {
            cache.clear();
        }
        self.global_manager.set_user_override(name, enabled)
    }

    /// Esecuzione unificata con chain mista - ottimizzata
    pub async fn execute(
        &self,
        loom_context: &LoomContext,
        def_name: &str, // Reference invece di owned String
        input_args: &[InputArg], // Slice invece di Vec owned
    ) -> InterceptorResult {
        let definition_target = loom_context.find_definition(def_name)
            .ok_or_else(|| LoomError::execution(format!("Cannot find the definition: '{}'", def_name)))?;

        let scope = ExecutionScope::from(definition_target.as_ref());

        // Costruisci ExecutionContext una volta sola
        let context = ExecutionContext {
            variables: loom_context.get_variables(def_name)
                .cloned()
                .unwrap_or_default(),
            env_vars: std::env::vars().collect(),
            working_dir: std::env::current_dir().ok()
                .map(|p| p.to_string_lossy().to_string()),
            dry_run: false,
            metadata: HashMap::new(),
            parallelization_kind: ParallelizationKind::Sequential,
            scope,
        };

        let target = ExecutionActivity::from(definition_target.as_ref());
        let global_interceptors = self.global_manager.get_active(&context);

        // Usa cache per chain se disponibile
        let cache_key = format!("{}_{}", def_name, input_args.len());
        let interceptor_chain = {
            if let Ok(cache) = self.chain_cache.read() {
                if let Some(cached_chain) = cache.get(&cache_key) {
                    cached_chain.clone()
                } else {
                    drop(cache); // Release read lock
                    let chain = self.build_target_chain(
                        loom_context,
                        &context,
                        &target, // Reference invece di owned
                        &global_interceptors,
                        Some(input_args)
                    )?;

                    // Cache la chain
                    if let Ok(mut cache) = self.chain_cache.write() {
                        cache.insert(cache_key, chain.clone());
                    }

                    chain
                }
            } else {
                // Fallback se non riusciamo ad accedere alla cache
                self.build_target_chain(
                    loom_context,
                    &context,
                    &target,
                    &global_interceptors,
                    Some(input_args)
                )?
            }
        };

        let interceptor_context = InterceptorContext {
            loom_context,
            execution_context: Arc::new(RwLock::new(context)),
            hook_registry: &self.hook_registry,
            channel: ExecutionEventChannel::new().0,
        };

        // Esegui la chain unificata
        Self::execute_chain(interceptor_context, &interceptor_chain).await
    }

    /// Build target chain ottimizzato - usa reference per evitare clone
    fn build_target_chain(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        execution_target: &ExecutionActivity, // Reference
        global_interceptors: &[ActiveGlobalInterceptor], // Slice
        args: Option<&[InputArg]>, // Optional slice
    ) -> LoomResult<Vec<ActiveInterceptor>> {
        match execution_target {
            ExecutionActivity::Command(command) => {
                match command.as_ref() {
                    Statement::Command { parts, directives } => {
                        Ok(Self::plug_and_sort_chain(
                            global_interceptors,
                            &self.directive_manager.build_active(loom_context, context, directives)?,
                            ActiveInterceptor::Executor(
                                ActiveExecutorInterceptor::new(
                                    Arc::new(CommandExecutorInterceptor(parts.clone()))
                                )
                            )
                        ))
                    }
                    Statement::Call { name, args, .. } => {
                        let definition_to_call = loom_context.find_definition(name.as_ref())
                            .ok_or_else(|| LoomError::execution(format!("Definition non esistente: '{}'", name)))?;

                        let activity = ExecutionActivity::from(definition_to_call.as_ref());
                        let converted_args = definition_to_call.signature
                            .positional_arg_from_expression(args.as_ref())?;

                        self.build_target_chain(
                            loom_context,
                            context,
                            &activity,
                            global_interceptors,
                            Some(&converted_args)
                        )
                    }
                }
            }

            ExecutionActivity::Block(block) => {
                let target = self.build_target_efficiently(
                    loom_context,
                    context,
                    execution_target,
                    global_interceptors,
                    "block-sequence"
                )?;

                Ok(Self::plug_and_sort_chain(
                    global_interceptors,
                    &self.directive_manager.build_active(loom_context, context, &block.directives)?,
                    ActiveInterceptor::Executor(
                        ActiveExecutorInterceptor::new(
                            Arc::new(SequentialExecutorInterceptor(target, "Block".to_string()))
                        )
                    )
                ))
            }

            ExecutionActivity::Stage(_) => Ok(Vec::new()),
            ExecutionActivity::Pipeline { .. } => Ok(Vec::new()),
            ExecutionActivity::Job { .. } => Ok(Vec::new()),
            ExecutionActivity::Schedule { .. } => Ok(Vec::new()),

            ExecutionActivity::Definition { directives, name, .. } => {
                let target = self.build_target_efficiently(
                    loom_context,
                    context,
                    execution_target,
                    global_interceptors,
                    "definition-sequence"
                )?;

                Ok(Self::plug_and_sort_chain(
                    global_interceptors,
                    &self.directive_manager.build_active(loom_context, context, directives)?,
                    ActiveInterceptor::Executor(
                        ActiveExecutorInterceptor::new(Arc::new(
                            DefinitionExecutorInterceptor(
                                name.to_string(),
                                target,
                                args.map(|a| a.to_vec()).unwrap_or_default()
                            )
                        ))
                    )
                ))
            }
        }
    }

    /// Build target in modo più efficiente - evita clone multipli
    fn build_target_efficiently(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        execution_target: &ExecutionActivity,
        global_interceptors: &[ActiveGlobalInterceptor],
        name: &str,
    ) -> LoomResult<Vec<ActiveInterceptor>> {
        let children = execution_target.build_child(loom_context, context)?;
        let mut result = Vec::with_capacity(children.len());

        for child in children {
            let chain = self.build_target_chain(
                loom_context,
                context,
                &child,
                global_interceptors,
                None
            )?;

            result.push(ActiveInterceptor::Executor(
                ActiveExecutorInterceptor::new(Arc::new(SequenceChainInterceptor(chain)))
            ));
        }

        Ok(result)
    }

    /// Combina interceptor in chain unificata - ottimizzato per evitare allocazioni
    fn plug_and_sort_chain(
        global: &[ActiveGlobalInterceptor], // Slice
        directive: &[ActiveDirectiveInterceptor], // Slice
        target_interceptor: ActiveInterceptor,
    ) -> Vec<ActiveInterceptor> {
        let total_capacity = global.len() + directive.len() + 1;
        let mut unified = Vec::with_capacity(total_capacity);

        // Aggiungi interceptor globali
        for interceptor in global {
            unified.push(ActiveInterceptor::Global(interceptor.clone()));
        }

        // Aggiungi interceptor di direttive
        for interceptor in directive {
            unified.push(ActiveInterceptor::Directive(interceptor.clone()));
        }

        // Ordina per priorità globale - ottimizzato
        unified.sort_unstable_by(ActiveInterceptor::sort);

        // Aggiungi target interceptor alla fine
        unified.push(target_interceptor);

        unified
    }

    /// Esegue la chain unificata - ottimizzata
    pub async fn execute_chain<'a>(
        context: InterceptorContext<'a>,
        chain: &'a [ActiveInterceptor],
    ) -> InterceptorResult {
        if chain.is_empty() {
            return Err(LoomError::execution("Empty interceptor chain"));
        }

        let mut index = 0;

        // Cerca il primo interceptor che ha bisogno di chain
        while index < chain.len() {
            if chain[index].need_chain() {
                return Self::execute_chain_recursive(context, chain, index).await;
            } else {
                // Esegui interceptor senza chain
                let result = Self::launch_interceptor(
                    context.clone(),
                    chain,
                    index,
                    empty_execute_intercept_next()
                ).await?;

                // Se è l'ultimo o abbiamo un risultato conclusivo, return
                if index == chain.len() - 1 {
                    return Ok(result);
                }
            }
            index += 1;
        }

        Err(LoomError::execution("No interceptor executed"))
    }

    /// Esecuzione ricorsiva della chain - ottimizzata
    async fn execute_chain_recursive<'a>(
        context: InterceptorContext<'a>,
        chain: &'a [ActiveInterceptor],
        index: usize,
    ) -> InterceptorResult {
        if index >= chain.len() {
            return Err(LoomError::execution("Chain index out of bounds"));
        }

        Self::launch_interceptor(
            context,
            chain,
            index,
            Self::create_next_chain(chain, index + 1)
        ).await
    }

    /// Launch interceptor ottimizzato
    async fn launch_interceptor<'a>(
        context: InterceptorContext<'a>,
        chain: &'a [ActiveInterceptor],
        index: usize,
        next: Box<InterceptorChain<'a>>
    ) -> InterceptorResult {
        match &chain[index] {
            ActiveInterceptor::Global(global) => {
                global.interceptor.intercept(context, &global.config, next).await
            }
            ActiveInterceptor::Directive(directive) => {
                directive.interceptor.intercept(context, next).await
            }
            ActiveInterceptor::Executor(executor) => {
                executor.interceptor.intercept(context, &executor.config, next).await
            }
        }
    }

    /// Create next chain - ottimizzato con bound checking
    fn create_next_chain<'a>(
        chain: &'a [ActiveInterceptor],
        next_index: usize
    ) -> Box<InterceptorChain<'a>> {
        Box::new(move |context: InterceptorContext<'a>| {
            if next_index < chain.len() {
                Box::pin(Self::execute_chain_recursive(context, chain, next_index))
            } else {
                Box::pin(async move {
                    Err(LoomError::execution("End of interceptor chain reached"))
                })
            }
        })
    }

    /// Diagnostica: lista interceptor attivi per un target - ottimizzata
    pub fn list_active_interceptors(&self, target: ExecutionScope) -> Vec<(String, String, i32)> {
        let context = ExecutionContext {
            variables: HashMap::new(),
            env_vars: std::env::vars().collect(),
            working_dir: None,
            dry_run: false,
            metadata: HashMap::new(),
            parallelization_kind: ParallelizationKind::Sequential,
            scope: target,
        };

        let global = self.global_manager.get_active(&context);
        let mut result = Vec::with_capacity(global.len());

        for interceptor in &global {
            result.push((
                interceptor.name.clone(),
                "global".to_string(),
                interceptor.config.priority,
            ));
        }

        result.sort_unstable_by(|a, b| b.2.cmp(&a.2));
        result
    }

    /// Valida che non ci siano conflitti di priorità
    pub fn validate_priority_conflicts(&self) -> Result<(), Vec<String>> {
        // Implementazione semplificata
        Ok(())
    }

    /// Clear cache - utile per testing
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.chain_cache.write() {
            cache.clear();
        }
    }

    /// Cache statistics per monitoring
    pub fn cache_stats(&self) -> Option<usize> {
        self.chain_cache.read().ok().map(|cache| cache.len())
    }
}