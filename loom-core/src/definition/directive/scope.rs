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
