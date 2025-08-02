use crate::ast::Expression;
use crate::types::LoomValue;

pub mod directive;
pub mod function;

/// Parametro che una direttiva/funzione/recipe può accettare
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDefinition {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<LoomValue>,
    pub description: String,
}
/// Argomento di una direttiva
#[derive(Debug, Clone, PartialEq)]
pub enum ArgDefinition {
    Positional(Expression),
    Named { name: String, value: Expression },
}

/// Tipi di parametri supportati
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array(Box<ParameterType>),
    Json,
    Enum(Vec<String>), // Per valori predefiniti
}