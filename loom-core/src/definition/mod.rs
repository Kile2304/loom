use std::sync::Arc;
use crate::ast::Expression;
use crate::types::LoomValue;

pub mod function;
pub mod parameter;
pub mod directive;

/// Regole di validazione per parametri
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationRules {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,   // Regex per stringhe
    pub min_value: Option<f64>,    // Per numeri
    pub max_value: Option<f64>,
}

/// Parametro che una direttiva/funzione/recipe può accettare
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub default_value: Option<LoomValue>,
    pub required: bool,
    pub allow_named_parameter: bool,
    pub varargs: bool, // Accetta argomenti variabili
    pub deprecated: bool,
    pub validation_rules: Option<ValidationRules>,
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
    Enum(Arc<str>), // Per valori predefiniti
    // Solo parametri definition potrebbero essere così!
    Any,
}

#[macro_export]
/// Macro principale universale per creare ParameterDefinition
macro_rules! param {
    // Caso base: solo nome
    ($param_type:expr, $name:expr) => {
        $crate::definition::ParameterDefinition {
            name: $name.to_string(),
            param_type: $param_type,
            description: String::new(),
            default_value: None,
            required: true,
            allow_named_parameter: true,
            varargs: false,
            deprecated: false,
            validation_rules: None,
        }
    };

    // Con argomenti aggiuntivi
    ($param_type:expr, $name:expr, $($key:ident $(=> $value:expr)?),* $(,)?) => {
        {
            let mut param = $crate::definition::ParameterDefinition {
                name: $name.to_string(),
                param_type: $param_type,
                description: String::new(),
                default_value: None,
                required: true,
                allow_named_parameter: true,
                varargs: false,
                deprecated: false,
                validation_rules: None,
            };

            $($crate::param!(@set_field param, $key $(=> $value)?);)*
            param
        }
    };

    // Helper interno per settare i campi
    (@set_field $param:ident, description => $value:expr) => {
        $param.description = $value.to_string();
    };

    (@set_field $param:ident, default => $value:expr) => {
        $param.default_value = Some($value);
        $param.required = false;
    };

    (@set_field $param:ident, required => $value:expr) => {
        $param.required = $value;
    };

    (@set_field $param:ident, allow_named => $value:expr) => {
        $param.allow_named_parameter = $value;
    };

    (@set_field $param:ident, varargs => $value:expr) => {
        $param.varargs = $value;
    };

    (@set_field $param:ident, deprecated => $value:expr) => {
        $param.deprecated = $value;
    };

    (@set_field $param:ident, validation => $value:expr) => {
        $param.validation_rules = Some($value);
    };

    // Flags senza valori
    (@set_field $param:ident, optional) => {
        $param.required = false;
    };

    (@set_field $param:ident, positional_only) => {
        $param.allow_named_parameter = false;
    };

    (@set_field $param:ident, varargs) => {
        $param.varargs = true;
    };

    (@set_field $param:ident, deprecated) => {
        $param.deprecated = true;
    };
}

#[macro_export]
/// Helper per creare ValidationRules
macro_rules! validation {
    ($($field:ident => $value:expr),* $(,)?) => {
        $crate::definition::ValidationRules {
            min_length: None,
            max_length: None,
            pattern: None,
            min_value: None,
            max_value: None,
            $($crate::validation!(@set_field $field => $value),)*
        }
    };

    (@set_field min_length => $value:expr) => { min_length: Some($value) };
    (@set_field max_length => $value:expr) => { max_length: Some($value) };
    (@set_field pattern => $value:expr) => { pattern: Some($value.to_string()) };
    (@set_field min_value => $value:expr) => { min_value: Some($value) };
    (@set_field max_value => $value:expr) => { max_value: Some($value) };
}

// Macro semplificate per tipi specifici
#[macro_export]
macro_rules! string_param {
    ($name:expr, $($key:ident$( => $value:expr)?),* $(,)?) => {
        $crate::param!($crate::definition::ParameterType::String, $name, $($key$( => $value)?),*)
    };
}

#[macro_export]
macro_rules! number_param {
    ($name:expr, $($key:ident$( => $value:expr)?),* $(,)?) => {
        $crate::param!($crate::definition::ParameterType::Number, $name, $($key$( => $value)?),*)
    };
}

#[macro_export]
macro_rules! bool_param {
    ($name:expr, $($key:ident$( => $value:expr)?),* $(,)?) => {
        $crate::param!($crate::definition::ParameterType::Boolean, $name, $($key$( => $value)?),*)
    };
}

#[macro_export]
macro_rules! json_param {
    ($name:expr, $($key:ident$( => $value:expr)?),* $(,)?) => {
        $crate::param!($crate::definition::ParameterType::Json, $name, $($key$( => $value)?),*)
    };
}

#[macro_export]
macro_rules! array_param {
    ($name:expr, $inner_type:expr, $($key:ident$( => $value:expr)?),* $(,)?) => {
        $crate::param!($crate::definition::ParameterType::Array(Box::new($inner_type)), $name, $($key $(=> $value)?),*)
    };
}

#[macro_export]
macro_rules! enum_param {
    ($name:expr, [$($variant:expr),+ $(,)?], $($key:ident$( => $value:expr)?),* $(,)?) => {
        $crate::param!(
            $crate::definition::ParameterType::Enum(vec![$($variant.to_string()),+]),
            $name,
            $($key $(=> $value)?),*
        )
    };
}

#[macro_export]
macro_rules! params {
    ($($param:expr),* $(,)?) => {
        vec![$($param),*]
    };
}