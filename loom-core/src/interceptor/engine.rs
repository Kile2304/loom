use std::collections::HashMap;
use std::sync::Arc;
use crate::context::LoomContext;
use crate::InputArg;
use crate::interceptor::{ActiveInterceptor, InterceptorChain, InterceptorResult};
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::directive::ActiveDirectiveInterceptor;
use crate::interceptor::directive::interceptor::DirectiveInterceptor;
use crate::interceptor::directive::manager::DirectiveInterceptorManager;
use crate::interceptor::executor::ActiveExecutorInterceptor;
use crate::interceptor::executor::implementation::command::CommandExecutorInterceptor;
use crate::interceptor::executor::implementation::composable::{SequenceChainInterceptor, SequentialExecutorInterceptor};
use crate::interceptor::global::ActiveGlobalInterceptor;
use crate::interceptor::global::config::GlobalInterceptorConfig;
use crate::interceptor::global::interceptor::GlobalInterceptor;
use crate::interceptor::global::manager::GlobalInterceptorManager;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::result::ExecutionResult;
use crate::interceptor::scope::ExecutionTarget;
use crate::types::ParallelizationKind;

// TODO: Attivare il registering e unregistering degli interceptor a runtime, per farlo probabilmente, dovrò aggiungere dei riferimenti al plugin.
// TODO: Ovviamente questa cosa potrà essere fatta solo se non c'è nulla in esecuzione, altrimenti, bisognerà aggiungerlo in pending,
// TODO: Ergo, purtroppo, dovrò gestire uno stack di esecuzioni...

/// Middleware Pattern (Filter Chain Pattern)
/// Esegue i vari Task/Job/Command, ma, solo dopo aver eseguito
/// Gli interceptor globali e le direttive, formando per l'appunto un Middleware Pattern
pub struct InterceptorEngine {
    global_manager: GlobalInterceptorManager,
    directive_manager: DirectiveInterceptorManager,
    hook_registry: HookRegistry,
}

impl InterceptorEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            global_manager: GlobalInterceptorManager::new(),
            directive_manager: DirectiveInterceptorManager::new(),
            hook_registry: HookRegistry::new(),
        };

        // Registra interceptor built-in
        // engine.register_builtin_interceptors();

        engine
    }

    /// Registra interceptor globale
    pub fn register_global(&mut self, interceptor: Arc<dyn GlobalInterceptor>) -> Result<(), String> {
        self.global_manager.register(interceptor)
    }

    /// Registra interceptor di direttiva
    pub fn register_directive(&mut self, interceptor: Arc<dyn DirectiveInterceptor>) -> Result<(), String> {
        self.directive_manager.register(interceptor)
    }

    /// Configura interceptor globale
    pub fn configure_global(&mut self, name: &str, config: GlobalInterceptorConfig) -> Result<(), String> {
        self.global_manager.configure(name, config)
    }

    /// Override temporaneo
    pub fn override_global(&mut self, name: &str, enabled: bool) -> Result<(), String> {
        self.global_manager.set_user_override(name, enabled)
    }

    /// Esecuzione unificata con chain mista
    pub async fn execute(
        &self,
        // Contesto globale "immutabile", non può essere modificato dall'esecuzione di una definition.
        // Mi serve per l'evaluate di expression: Es: StringInterpolation su valore di enum, o esecuzione di una definition come command!
        loom_context: &LoomContext,
        // Definition Name
        def_name: String,
        input_args: Vec<InputArg>,
    ) -> Result<ExecutionResult, String> {
        // Dovrebbero essere fatti controlli a monte, quindi, dovrei SEMPRE trovare la definition
        let definition_target =
            loom_context.find_definition(&def_name)
                .ok_or(format!("Cannot find the definition: '{def_name}'"))?;

        let target = ExecutionTarget::Definition { name: "".to_string(), blocks: vec![], directives: vec![] };

        let mut context = ExecutionContext {
            variables: loom_context.get_variables(&def_name).unwrap().clone(),
            env_vars: std::env::vars().collect(),
            working_dir: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
            dry_run: false,
            metadata: HashMap::new(),
            parallelization_kind: ParallelizationKind::Sequential,
            target,
            previous_result: None,
        };

        definition_target.signature.args_into_variable(loom_context, &context, input_args)?.into_iter()
            .for_each(|(variable_name, value)| { context.variables.insert(variable_name, value); });

        let interceptor_chain =
            self.build_target_chain(
                loom_context,
                &context,
                &context.target,
                &self.global_manager.get_active(&context)
            )?;

        // Esegui la chain unificata
        Self::execute_chain(loom_context, &mut context, &self.hook_registry, &interceptor_chain).await
    }

    fn build_target_chain(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        execution_target: &ExecutionTarget,
        global_interceptor: &Vec<ActiveGlobalInterceptor>,
    ) -> Result<Vec<ActiveInterceptor>, String> {
        match execution_target {
            ExecutionTarget::Command(command) => {
                Ok(
                    Self::plug_and_sort_chain(
                        global_interceptor,
                        self.directive_manager.build_active(loom_context, context, &command.directives)?,
                        ActiveInterceptor::Executor(
                            ActiveExecutorInterceptor::new(
                                Arc::new(CommandExecutorInterceptor(command.command.clone(), command.workind_dir.clone()))
                            )
                        )
                    )
                )
            }
            ExecutionTarget::Block(block) => {
                let target =
                    self.build_target(
                        loom_context,
                        context,
                        execution_target,
                        global_interceptor,
                        "block-sequence"
                    )?;
                Ok(
                    Self::plug_and_sort_chain(
                        global_interceptor,
                        self.directive_manager.build_active(loom_context, context, &block.directives)?,
                        // TODO: Valutare se serve avere un vero e proprio: BlockExecutor
                        ActiveInterceptor::Executor(
                            ActiveExecutorInterceptor::new(
                                Arc::new(SequentialExecutorInterceptor(target, "Block".to_string()))
                            )
                        )
                    )
                )
            }
            ExecutionTarget::Stage(_) => { Ok(vec![]) }
            ExecutionTarget::Pipeline { .. } => { Ok(vec![]) }
            ExecutionTarget::Job { .. } => { Ok(vec![]) }
            ExecutionTarget::Schedule { .. } => { Ok(vec![]) }
            ExecutionTarget::Definition { directives, .. } => {
                let target =
                    self.build_target(
                        loom_context,
                        context,
                        execution_target,
                        global_interceptor,
                        "definition-sequence"
                    )?;
                Ok(
                    Self::plug_and_sort_chain(
                        global_interceptor,
                        self.directive_manager.build_active(loom_context, context, directives)?,
                        ActiveInterceptor::Executor(
                            ActiveExecutorInterceptor::new(Arc::new(SequentialExecutorInterceptor(target, "definition".to_string())))
                        )
                    )
                )
            }
            ExecutionTarget::None => {
                panic!("Non dovrebbe mai entrare qui, infatti più avanti verrà rimosso!")
            }
        }
    }

    fn build_target(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        execution_target: &ExecutionTarget,
        global_interceptor: &Vec<ActiveGlobalInterceptor>,
        name: &str,
    ) -> Result<Vec<ActiveInterceptor>, String> {
        execution_target.build_child().iter()
            .map(|it|
                self.build_target_chain(loom_context, context, it, global_interceptor)
                    .map(SequenceChainInterceptor)
                    .map(|it| ActiveExecutorInterceptor {
                        interceptor: Arc::new(it),
                        config: Default::default(),
                        name: name.to_string(),
                    }).map(ActiveInterceptor::Executor)
            )
        .collect::<Result<Vec<_>, _>>()
    }

    /// Combina interceptor in chain unificata
    fn plug_and_sort_chain(
        global: &Vec<ActiveGlobalInterceptor>,
        directive: Vec<ActiveDirectiveInterceptor>,
        target_interceptor: ActiveInterceptor,
    ) -> Vec<ActiveInterceptor> {
        let mut unified = Vec::new();

        // Aggiungi interceptor globali
        for interceptor in global {
            unified.push(ActiveInterceptor::Global(interceptor.clone()));
        }

        // Aggiungi interceptor di direttive
        for interceptor in directive {
            unified.push(ActiveInterceptor::Directive(interceptor));
        }

        // Ordina per priorità globale
        unified.sort_by(ActiveInterceptor::sort);

        // Aggiungo al fondo delle esecuzioni gli interceptor che eseguono il task vero e proprio e gli interceptor a più basso livello
        // Come per esempio, gli interceptor dei job, o, gli interceptor dei command...
        // In questo modo ho:
        // - interceptor definition -> definition   -> [interceptor command -> command]
        // - interceptor pipeline   -> pipeline     -> [interceptor stage -> stage      -> [interceptor job -> job -> [interceptor command -> command]]
        unified.push(target_interceptor);

        unified
    }

    /// Esegue la chain unificata
    pub async fn execute_chain(
        loom_context: &LoomContext,
        context: &mut ExecutionContext,
        hook_registry: &HookRegistry,
        chain: &[ActiveInterceptor],
    ) -> Result<ExecutionResult, String> {
        Self::execute_chain_recursive(loom_context, context, hook_registry, chain, 0).await
    }

    // TODO: IMPORTANTE!!! La catena allo stesso livello dell'albero, ma, successiva, deve avere come parametro, il risultato della precedente!!!

    /// Esecuzione ricorsiva della chain
    async fn execute_chain_recursive(
        loom_context: &LoomContext,
        context: &mut ExecutionContext,
        hook_registry: &HookRegistry,
        chain: &[ActiveInterceptor],
        index: usize,
    ) -> InterceptorResult {

        let interceptor = &chain[index];

        let next = Self::create_next_chain(chain, index + 1);

        match interceptor {
            ActiveInterceptor::Global(global) => {
                global.interceptor.intercept(loom_context, context, hook_registry, &global.config, Box::new(next)).await
            }
            ActiveInterceptor::Directive(directive) => {
                // Inietta parametri nel context
                // context.directive_params = directive.params.clone();

                directive.interceptor.intercept(loom_context, context, hook_registry, Box::new(next)).await
            }
            ActiveInterceptor::Executor(executor) => {
                executor.interceptor.intercept(loom_context, context, hook_registry, &executor.config, Box::new(next)).await
            }
        }
    }

    fn create_next_chain<'a>(
        chain: &'a [ActiveInterceptor],
        next_index: usize
    ) -> Box<InterceptorChain<'a>> {
        Box::new(move |loom_context: &'a LoomContext, ctx: &'a mut ExecutionContext, hooks: &'a HookRegistry| {
            Box::pin(Self::execute_chain_recursive(loom_context, ctx, hooks, chain, next_index))
        })
    }

    // fn register_builtin_interceptors(&mut self) {
    //     // Interceptor globali built-in
    //     self.register_global(Arc::new(SecurityAuditInterceptor::new())).unwrap();
    //     self.register_global(Arc::new(PerformanceMonitorInterceptor::new())).unwrap();
    //     self.register_global(Arc::new(ComplianceInterceptor::new())).unwrap();
    //
    //     // Interceptor di direttive built-in
    //     self.register_directive(Arc::new(TimeoutDirectiveInterceptor::new())).unwrap();
    //     self.register_directive(Arc::new(ParallelDirectiveInterceptor::new())).unwrap();
    //     self.register_directive(Arc::new(IfDirectiveInterceptor::new())).unwrap();
    // }

    /// Diagnostica: lista interceptor attivi per un target
    pub fn list_active_interceptors(&self, target: ExecutionTarget) -> Vec<(String, String, i32)> {
        let context = ExecutionContext {
            variables: HashMap::new(),
            env_vars: std::env::vars().collect(),
            working_dir: None,
            dry_run: false,
            metadata: HashMap::new(),
            parallelization_kind: ParallelizationKind::Sequential,
            previous_result: None,
            target,
        };

        let global = self.global_manager.get_active(&context);
        let mut result = Vec::new();

        for interceptor in global {
            result.push((
                interceptor.name.clone(),
                "global".to_string(),
                interceptor.config.priority,
            ));
        }

        result.sort_by(|a, b| b.2.cmp(&a.2));
        result
    }

    /// Valida che non ci siano conflitti di priorità
    pub fn validate_priority_conflicts(&self) -> Result<(), Vec<String>> {
        // Per ora implementazione semplice
        // In pratica potresti fare check più sofisticati
        Ok(())
    }
}
