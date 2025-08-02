use std::collections::HashMap;
use crate::ast::DirectiveCall;
use crate::definition::{ArgDefinition, ParameterDefinition};
use crate::error::LoomResult;
use crate::interceptor::scope::DirectiveScope;
use crate::types::LoomValue;

/// Definizione di una direttiva (per il parser)
pub trait DirectiveDefinition: Send + Sync {
    /// Nome della direttiva (senza @)
    fn name(&self) -> &str;

    /// Descrizione per l'help
    fn description(&self) -> &str;

    /// Dove può essere usata
    fn scope(&self) -> &[DirectiveScope];

    /// Parametri accettati
    fn parameters(&self) -> Vec<ParameterDefinition>;

    /// Validazione customizzata dei parametri
    fn validate_parameters(&self, args: &[ArgDefinition]) -> LoomResult<()> {
        // Default implementation
        Ok(())
    }

    /// Se la direttiva può essere ripetuta sullo stesso elemento
    fn repeatable(&self) -> bool {
        false
    }

    /// Direttive incompatibili
    fn conflicts_with(&self) -> &[&str] {
        &[]
    }

    /// Trasforma il DirectiveCall in parametri strutturati per l'executor
    fn parse_args(&self, call: &DirectiveCall) -> LoomResult<HashMap<String, LoomValue>>;
}