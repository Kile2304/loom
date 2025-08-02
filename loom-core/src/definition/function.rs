use std::collections::HashMap;
use crate::ast::DirectiveCall;
use crate::definition::{ArgDefinition, ParameterDefinition};
use crate::error::LoomResult;
use crate::types::LoomValue;

// TODO: Integrazione ancora da studiare
pub trait FunctionDefinition: Send + Sync {

    /// Nome della direttiva (senza @)
    fn name(&self) -> &str;

    /// Descrizione per l'help
    fn description(&self) -> &str;

    /// Parametri accettati
    fn parameters(&self) -> Vec<ParameterDefinition>;

    /// Validazione customizzata dei parametri
    fn validate_parameters(&self, args: &[ArgDefinition]) -> LoomResult<()> {
        // Default implementation
        Ok(())
    }

    /// Trasforma il DirectiveCall in parametri strutturati per l'executor
    fn parse_args(&self, call: &DirectiveCall) -> LoomResult<HashMap<String, LoomValue>>;

}