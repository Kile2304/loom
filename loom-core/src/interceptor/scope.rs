use crate::ast::{Definition, DirectiveCall, Expression, Statement};
use crate::context::LoomContext;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::directive::ActiveDirectiveInterceptor;
use crate::interceptor::executor::interceptor::ExecutorInterceptor;
use crate::interceptor::global::ActiveGlobalInterceptor;
use crate::types::DefinitionKind;

/// Quando una direttiva viene eseguita nel ciclo di vita
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionKind {
    /// Durante il parsing/help - per direttive informative
    Help,
    /// Prima di eseguire una definition (recipe/job/pipeline)
    ExecuteDefinition,
    /// Prima di eseguire un singolo job
    ExecuteJob,
    /// Prima di eseguire un comando shell
    ExecuteCommand,
    /// Durante la valutazione del contesto (variabili, espressioni)
    ContextEvaluation,
    /// Durante la validazione sintattica
    Validation,
}

/// Livello dove può essere applicata una direttiva
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirectiveScope {
    /// A livello di definition (recipe, job, pipeline)
    Definition,
    /// A livello di statement (comando, if, for)
    Statement,
    /// A livello di stage (solo per pipeline)
    Stage,
    /// Globale (file level)
    Global,
    /// Single command level
    Command,
    /// Block of commands (like if-else)
    Block
}

// Potrebbe essere il caso di creare un corrispettivo x Scheduler.
// @Scheduler, @pipeline e @job; sarebero directive a "load time", ovvero, non hanno un corrispondente interceptor
// Ma permettono di distinguere che tipo di ExecutionTarget creare!
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionActivity {
    // TODO: Terminale
    Command(Statement),
    // All single command are considered implicit Block
    // Block like if-else, for...
    Block(BlockTarget),
    // TODO: Si trasforma ed esegue n job 
    Pipeline {
        name: String,
        directives: Vec<DirectiveCall>,
        stages: Vec<BlockTarget>,
    },
    Stage(BlockTarget),
    // TODO: Job si trasforma in Command
    Job {
        name: String,
        directives: Vec<DirectiveCall>,
        // Comandi contenuti nella definition
        blocks: Vec<BlockTarget>,
    },
    // TODO: Si trasforma ed esegue jobs??
    Schedule {
        name: String,
        directives: Vec<DirectiveCall>
        // TODO: Di quali altre informazioni ho bisogno?
    },
    // TODO: Definition si trasforma ed esegue N Command
    Definition {
        name: String,
        directives: Vec<DirectiveCall>,
        // Comandi contenuti nella definition
        blocks: Vec<BlockTarget>,
    },
}
#[derive(Debug, Clone)]
pub enum ExecutionScope {
    Command,
    Block,
    Pipeline,
    Job,
    Stage,
    Schedule,
    Definition,
}
impl From<&Definition> for ExecutionScope {
    fn from(value: &Definition) -> Self {
        match value.kind {
            DefinitionKind::Job         => Self::Definition,
            DefinitionKind::Recipe      => Self::Command,
            DefinitionKind::Schedule    => Self::Block,
            DefinitionKind::Pipeline    => Self::Pipeline,
        }
    }
}

// TODO: Troppi clone, prevedre il passaggio di reference o un Arc

impl From<&Definition> for ExecutionActivity {
    fn from(value: &Definition) -> Self {
        match value.kind {
            DefinitionKind::Recipe => {
                ExecutionActivity::Definition {
                    name: value.signature.name.to_string(),
                    directives: value.directives.clone(),
                    blocks: 
                        value.body.iter()
                            .map(|it| BlockTarget {
                                directives: it.directives.clone(),
                                commands: it.statements.clone(),
                                label: it.label.clone(),
                            })
                        .collect(),
                }
            }
            DefinitionKind::Job => {
                ExecutionActivity::Job {
                    name: value.signature.name.to_string(),
                    directives: value.directives.clone(),
                    blocks: value.body.iter()
                            .map(|it| BlockTarget {
                                directives: it.directives.clone(),
                                commands: it.statements.clone(),
                                label: it.label.clone(),
                            })
                        .collect::<Vec<_>>(),
                }
            }
            DefinitionKind::Pipeline => {
                ExecutionActivity::Pipeline {
                    name: value.signature.name.to_string(),
                    directives: value.directives.clone(),
                    stages: value.body.iter()
                        .map(|it| BlockTarget {
                            directives: it.directives.clone(),
                            commands: it.statements.clone(),
                            label: it.label.clone(),
                        })
                    .collect(),
                }
            }
            DefinitionKind::Schedule => {
                ExecutionActivity::Schedule {
                    name: value.signature.name.to_string(),
                    directives: value.directives.clone(),
                }
            }
        }
    }
}

impl ExecutionActivity {
    
    pub fn build_child(&self, loom_context: &LoomContext, context: &ExecutionContext) -> Result<Vec<ExecutionActivity>, String> {
        Ok(
            match self {
                ExecutionActivity::Command(_) => vec![],
                ExecutionActivity::Block(block) => {
                    block.commands.iter()
                        .map(|it| ExecutionActivity::Command(it.clone()))
                    .collect::<Vec<_>>()
                }
                ExecutionActivity::Stage(stage) => {
                    stage.commands.iter()
                        // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                        .map(|it| it.clone())
                        .map(|it| {
                            match it {
                                // Directives not supported yet for job in stage
                                Statement::Command { parts, .. } => {
                                    let name = 
                                        parts.into_iter()
                                            .map(|it|
                                                it.evaluate(loom_context, context)
                                                    .and_then(|it| it.stringify(loom_context, context))
                                            )
                                        .collect::<Result<Vec<_>, _>>()
                                            .map(|it| it.join(""))?;
                                    let job_definition = 
                                        loom_context.find_definition(&name)
                                            .ok_or_else(||format!("Cannot find Job: '{name}'"))?;
                                    Ok(
                                        ExecutionActivity::Job {
                                            name: name,
                                            directives: job_definition.directives.clone(),
                                            blocks: 
                                                job_definition.body
                                                    .iter()
                                                        .map(|it| BlockTarget {
                                                            directives: it.directives.clone(),
                                                            commands: it.statements.clone(),
                                                            label: it.label.clone(),
                                                        })
                                                .collect(),
                                        }
                                    )
                                }
                                // Non dovrebbe MAI capitare perchè controllato durante il parsing, ma, meglio controllarlo comunque...
                                _ => Err(format!("Tipo di statement non previsto per uno stage!"))
                            }
                        })
                    .collect::<Result<Vec<_>, _>>()?
                }
                ExecutionActivity::Pipeline { stages, .. } => {
                    stages.iter()
                        // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                        .map(|it| it.clone())
                        .map(ExecutionActivity::Stage)
                    .collect::<Vec<_>>()
                }
                ExecutionActivity::Job { name, blocks, directives } => {
                    blocks.iter()
                        // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                        .map(|it| it.clone())
                        .map(ExecutionActivity::Block)
                    .collect::<Vec<_>>()
                }
                ExecutionActivity::Schedule { name, .. } => {
                    // TODO: Empty temporaneo, perchè devo definire bene cosa può contenere
                    vec![]
                }
                ExecutionActivity::Definition { name, blocks, .. } => {
                    blocks.iter()
                        // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                        .map(|it| it.clone())
                        .map(|it| ExecutionActivity::Block(BlockTarget { directives: it.directives, commands: it.commands, label: it.label }))
                    .collect::<Vec<_>>()
                }
            }
        )
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BlockTarget {
    pub directives: Vec<DirectiveCall>,
    pub commands: Vec<Statement>,
    pub label: Vec<Expression>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct JobTarget {
    pub name: String,
    pub blocks: Vec<BlockTarget>,
}

// TODO: Capire bene se ho bisogno di altri Hooks

/// Hook system per eventi granulari
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionHook {
    PreParse,
    PostParse,
    PreValidation,
    PostValidation,
    PreExecution,
    PostExecution,
    PreCommand,
    PostCommand,
    OnError,
    OnSuccess,
    Cleanup,
}