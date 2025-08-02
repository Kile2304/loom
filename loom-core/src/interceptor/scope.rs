use crate::ast::{Definition, DirectiveCall, Expression};
use crate::interceptor::directive::ActiveDirectiveInterceptor;
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
#[derive(Debug, Default, Clone, PartialEq)]
pub enum ExecutionTarget {
    // TODO: Terminale
    Command(CommandTarget),
    // All single command are considered implicit Block
    // Block like if-else, for...
    Block(BlockTarget),
    // TODO: Si trasforma ed esegue n job 
    Pipeline {
        name: String,
        directives: Vec<DirectiveCall>,
        stages: Vec<StageTarget>,
    },
    Stage(StageTarget),
    // TODO: Job si trasforma in Command
    Job(JobTarget),
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
    #[default] None
}

// impl<'a> From<&'a Definition> for ExecutionTarget {
//     fn from(value: &'a Definition) -> Self {
//         match &value.kind {
//             DefinitionKind::Recipe => {
//                 Self::Definition {
//                     name: value.signature.name.to_string(),
//                     blocks: 
//                         value.body.statements.iter()
//                             .map(|it| it.)
//                 }
//             }
//             DefinitionKind::Job => {
//                 
//             }
//             DefinitionKind::Pipeline => {
//                 
//             }
//             DefinitionKind::Schedule => {
//                 
//             }
//             DefinitionKind::Plugin => {
//                 
//             }
//         }
//     }
// }

impl ExecutionTarget {
    
    pub fn build_child(&self) -> Vec<ExecutionTarget> {
        match self { 
            ExecutionTarget::Command(_)  | ExecutionTarget::None => vec![],
            ExecutionTarget::Block(block) => {
                block.commands.iter()
                    .map(|it| ExecutionTarget::Command(it.clone()))
                .collect::<Vec<_>>()
            }
            ExecutionTarget::Stage(stage) => {
                stage.jobs.iter()
                    // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                    .map(|it| it.clone())
                    .map(ExecutionTarget::Job)
                .collect::<Vec<_>>()
            }
            ExecutionTarget::Pipeline { stages, .. } => {
                stages.iter()
                    // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                    .map(|it| it.clone())
                    .map(ExecutionTarget::Stage)
                .collect::<Vec<_>>()
            }
            ExecutionTarget::Job(job) => {
                job.command.iter()
                    // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                    .map(|it| it.clone())
                    .map(ExecutionTarget::Command)
                .collect::<Vec<_>>()
            }
            ExecutionTarget::Schedule { name, .. } => {
                // TODO: Empty temporaneo, perchè devo definire bene cosa può contenere
                vec![]
            }
            ExecutionTarget::Definition { name, blocks, .. } => {
                blocks.iter()
                    // TODO: Clone temporaneo, vedere se riesce a procedere senza clonare l'oggetto
                    .map(|it| it.clone())
                    .map(|it| ExecutionTarget::Block(BlockTarget { directives: it.directives, commands: it.commands }))
                .collect::<Vec<_>>()
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BlockTarget {
    pub directives: Vec<DirectiveCall>,
    pub commands: Vec<CommandTarget>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CommandTarget {
    pub command: Vec<Expression>,
    pub workind_dir: Option<String>,
    pub directives: Vec<DirectiveCall>
}
#[derive(Debug, Default, Clone, PartialEq)]
pub struct JobTarget {
    pub name: String,
    pub directives: Vec<DirectiveCall>,
    pub command: Vec<CommandTarget>,
}
#[derive(Debug, Default, Clone, PartialEq)]
pub struct StageTarget {
    pub name: String,
    pub directives: Vec<DirectiveCall>,
    pub jobs: Vec<JobTarget>
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