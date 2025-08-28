use crate::ast::{Definition, DirectiveCall, Expression, Statement};
use crate::context::LoomContext;
use crate::error::{LoomError, LoomResult};
use crate::interceptor::context::ExecutionContext;
use crate::types::DefinitionKind;
use std::sync::Arc;

/// Quando una direttiva viene eseguita nel ciclo di vita
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// ExecutionActivity ottimizzata con Arc per evitare clone pesanti
#[derive(Debug, Clone)]
pub enum ExecutionActivity {
    // Terminale - usa Arc per Statement condiviso
    Command(Arc<Statement>),

    // Block con Arc per evitare clone
    Block(Arc<BlockTarget>),

    // Pipeline ottimizzata
    Pipeline {
        name: Arc<str>,
        directives: Arc<[DirectiveCall]>,
        stages: Arc<[BlockTarget]>,
    },

    Stage(Arc<BlockTarget>),

    // Job ottimizzato
    Job {
        name: Arc<str>,
        directives: Arc<[DirectiveCall]>,
        blocks: Arc<[BlockTarget]>,
    },

    // Schedule ottimizzato
    Schedule {
        name: Arc<str>,
        directives: Arc<[DirectiveCall]>
    },

    // Definition ottimizzata
    Definition {
        name: Arc<str>,
        directives: Arc<[DirectiveCall]>,
        blocks: Arc<[BlockTarget]>,
    },
}

#[derive(Debug, Clone, Copy)]
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

/// Conversion ottimizzata da Definition che evita clone multipli
impl From<&Definition> for ExecutionActivity {
    fn from(value: &Definition) -> Self {
        let name_arc: Arc<str> = value.signature.name.clone().into();
        let directives_arc = value.directives.clone();

        // Pre-converti i block in BlockTarget una volta sola
        let blocks_arc: Arc<[BlockTarget]> = value.body.iter()
            .map(|block| BlockTarget {
                directives: block.directives.clone(),
                commands: block.statements.clone(),
                label: block.label.clone(),
            })
            .collect::<Vec<_>>()
            .into();

        match value.kind {
            DefinitionKind::Recipe => {
                ExecutionActivity::Definition {
                    name: name_arc,
                    directives: directives_arc,
                    blocks: blocks_arc,
                }
            }
            DefinitionKind::Job => {
                ExecutionActivity::Job {
                    name: name_arc,
                    directives: directives_arc,
                    blocks: blocks_arc,
                }
            }
            DefinitionKind::Pipeline => {
                ExecutionActivity::Pipeline {
                    name: name_arc,
                    directives: directives_arc,
                    stages: blocks_arc,
                }
            }
            DefinitionKind::Schedule => {
                ExecutionActivity::Schedule {
                    name: name_arc,
                    directives: directives_arc,
                }
            }
        }
    }
}

impl ExecutionActivity {
    /// Build child activities - DRASTICAMENTE ottimizzato per evitare clone
    pub fn build_child(&self, loom_context: &LoomContext, context: &ExecutionContext) -> LoomResult<Vec<ExecutionActivity>> {
        match self {
            ExecutionActivity::Command(_) => Ok(Vec::new()),

            ExecutionActivity::Block(block) => {
                // Usa iterator e map invece di collect + clone
                let activities: Vec<ExecutionActivity> = block.commands.iter()
                    .map(|stmt| ExecutionActivity::Command(Arc::new(stmt.clone()))) // Solo questo clone è inevitabile per ora
                    .collect();
                Ok(activities)
            }

            ExecutionActivity::Stage(stage) => {
                // Pre-alloca con capacità nota
                let mut activities = Vec::with_capacity(stage.commands.len());

                for statement in stage.commands.iter() {
                    match statement {
                        Statement::Command { parts, .. } => {
                            // Evaluation efficace evitando cloni temporanei
                            let name = parts.iter()
                                .map(|expr| {
                                    expr.evaluate(loom_context, context, Default::default())
                                        .and_then(|val| val.stringify(loom_context, context))
                                })
                                .collect::<LoomResult<Vec<_>>>()?
                                .join("");

                            let job_definition = loom_context.find_definition(&name)
                                .ok_or_else(|| LoomError::definition_resolution(&name, "Cannot find Job"))?;

                            // Usa Arc per evitare clone delle parti pesanti
                            let blocks: Arc<[BlockTarget]> = job_definition.body.iter()
                                .map(|block| BlockTarget {
                                    directives: block.directives.clone(),
                                    commands: block.statements.clone(),
                                    label: block.label.clone(),
                                })
                                .collect::<Vec<_>>()
                                .into();

                            activities.push(ExecutionActivity::Job {
                                name: name.into(),
                                directives: job_definition.directives.clone(),
                                blocks,
                            });
                        }
                        _ => return Err(LoomError::execution("Tipo di statement non previsto per uno stage!"))
                    }
                }

                Ok(activities)
            }

            ExecutionActivity::Pipeline { stages, .. } => {
                // Map diretto senza clone intermedio
                let activities: Vec<ExecutionActivity> = stages.iter()
                    .map(|stage| ExecutionActivity::Stage(Arc::new(stage.clone()))) // Clone minimale
                    .collect();
                Ok(activities)
            }

            ExecutionActivity::Job { blocks, .. } => {
                // Map diretto
                let activities: Vec<ExecutionActivity> = blocks.iter()
                    .map(|block| ExecutionActivity::Block(Arc::new(block.clone()))) // Clone minimale
                    .collect();
                Ok(activities)
            }

            ExecutionActivity::Schedule { .. } => Ok(Vec::new()),

            ExecutionActivity::Definition { blocks, .. } => {
                // Map diretto
                let activities: Vec<ExecutionActivity> = blocks.iter()
                    .map(|block| ExecutionActivity::Block(Arc::new(block.clone()))) // Clone minimale
                    .collect();
                Ok(activities)
            }
        }
    }
}

/// BlockTarget ottimizzato con Arc slices
#[derive(Debug, Clone, PartialEq)]
pub struct BlockTarget {
    pub directives: Arc<[DirectiveCall]>,
    pub commands: Arc<[Statement]>,
    pub label: Arc<[Expression]>,
}

impl Default for BlockTarget {
    fn default() -> Self {
        Self {
            directives: Arc::new([]),
            commands: Arc::new([]),
            label: Arc::new([]),
        }
    }
}

/// JobTarget ottimizzato
#[derive(Debug, Clone, PartialEq)]
pub struct JobTarget {
    pub name: Arc<str>,
    pub blocks: Arc<[BlockTarget]>,
}

impl Default for JobTarget {
    fn default() -> Self {
        Self {
            name: Arc::from(""),
            blocks: Arc::new([]),
        }
    }
}

/// Hook system per eventi granulari
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Builder helpers per conversion efficienti
impl BlockTarget {
    pub fn new(
        directives: impl Into<Arc<[DirectiveCall]>>,
        commands: impl Into<Arc<[Statement]>>,
        label: impl Into<Arc<[Expression]>>
    ) -> Self {
        Self {
            directives: directives.into(),
            commands: commands.into(),
            label: label.into(),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn with_commands(commands: impl Into<Arc<[Statement]>>) -> Self {
        Self {
            directives: Arc::new([]),
            commands: commands.into(),
            label: Arc::new([]),
        }
    }
}

impl JobTarget {
    pub fn new(name: impl Into<Arc<str>>, blocks: impl Into<Arc<[BlockTarget]>>) -> Self {
        Self {
            name: name.into(),
            blocks: blocks.into(),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }
}

/// Utility methods per ExecutionActivity
impl ExecutionActivity {
    pub fn name(&self) -> Option<&str> {
        match self {
            ExecutionActivity::Pipeline { name, .. } => Some(name.as_ref()),
            ExecutionActivity::Job { name, .. } => Some(name.as_ref()),
            ExecutionActivity::Schedule { name, .. } => Some(name.as_ref()),
            ExecutionActivity::Definition { name, .. } => Some(name.as_ref()),
            _ => None,
        }
    }

    pub fn directives(&self) -> Option<&[DirectiveCall]> {
        match self {
            ExecutionActivity::Pipeline { directives, .. } => Some(directives.as_ref()),
            ExecutionActivity::Job { directives, .. } => Some(directives.as_ref()),
            ExecutionActivity::Schedule { directives, .. } => Some(directives.as_ref()),
            ExecutionActivity::Definition { directives, .. } => Some(directives.as_ref()),
            ExecutionActivity::Block(block) => Some(block.directives.as_ref()),
            ExecutionActivity::Stage(stage) => Some(stage.directives.as_ref()),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ExecutionActivity::Command(_))
    }

    pub fn children_count(&self) -> usize {
        match self {
            ExecutionActivity::Command(_) => 0,
            ExecutionActivity::Block(block) => block.commands.len(),
            ExecutionActivity::Pipeline { stages, .. } => stages.len(),
            ExecutionActivity::Stage(stage) => stage.commands.len(),
            ExecutionActivity::Job { blocks, .. } => blocks.len(),
            ExecutionActivity::Schedule { .. } => 0,
            ExecutionActivity::Definition { blocks, .. } => blocks.len(),
        }
    }
}