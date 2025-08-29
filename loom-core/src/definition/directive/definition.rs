use crate::definition::{ArgDefinition, ParameterDefinition};
use crate::definition::directive::scope::DirectiveScope;
use crate::definition::parameter::{determine_argument_type, validate_named_arguments, validate_positional_arguments, ArgumentType};
use crate::error::LoomResult;

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
        let parameters = self.parameters();

        // Validazione della conformità dei parametri della direttiva
        // TODO: Spostare su registry
        // validate_parameter_definitions(&parameters)?;

        // Determina il tipo di argomenti (tutti posizionali o tutti named)
        let arg_type = determine_argument_type(args)?;

        match arg_type {
            ArgumentType::Positional => {
                validate_positional_arguments(args, &parameters, self.name())?;
            }
            ArgumentType::Named => {
                validate_named_arguments(args, &parameters, self.name())?;
            }
        }

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

}