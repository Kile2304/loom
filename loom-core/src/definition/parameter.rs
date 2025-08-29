use std::collections::{HashMap, HashSet};
use crate::ast::Expression;
use crate::definition::{ArgDefinition, ParameterDefinition, ParameterType, ValidationRules};
use crate::error::{LoomError, LoomResult};
use crate::types::{LiteralValue, Position};

/// Tipo di argomenti utilizzati
#[derive(Debug, PartialEq)]
pub enum ArgumentType {
    Positional,
    Named,
}

/// Determina se gli argomenti sono tutti posizionali o tutti named
pub fn determine_argument_type(args: &[ArgDefinition]) -> LoomResult<ArgumentType> {
    if args.is_empty() {
        return Ok(ArgumentType::Positional);
    }

    let has_positional = args.iter().any(|arg| matches!(arg, ArgDefinition::Positional(_)));
    let has_named = args.iter().any(|arg| matches!(arg, ArgDefinition::Named { .. }));

    if has_positional && has_named {
        return Err(LoomError::validation(
            "Cannot mix 'positional' and 'named' arguments in the same directive call"
        ));
    }

    if has_named {
        Ok(ArgumentType::Named)
    } else {
        Ok(ArgumentType::Positional)
    }
}

/// Validazione per argomenti posizionali
pub fn validate_positional_arguments(
    args: &[ArgDefinition],
    parameters: &[ParameterDefinition],
    directive_name: &str,
) -> LoomResult<()> {
    // Verifica che non ci siano parametri che non accettano posizionali
    for param in parameters {
        if param.allow_named_parameter && !can_be_positional(param) {
            return Err(LoomError::parameter_validation(
                directive_name,
                format!("Parameter '{}' can only be used as named parameter", param.name)
            ));
        }
    }

    let required_count = parameters.iter().filter(|p| p.required).count();
    let max_positional_count = if has_varargs_parameter(parameters) {
        // Con varargs, possiamo avere un numero illimitato di argomenti
        usize::MAX
    } else {
        parameters.len()
    };

    // Controllo numero minimo di argomenti
    if args.len() < required_count {
        return Err(LoomError::parameter_mismatch(
            directive_name,
            required_count,
            args.len()
        ));
    }

    // Controllo numero massimo di argomenti (solo se non c'è varargs)
    if max_positional_count != usize::MAX && args.len() > max_positional_count {
        return Err(LoomError::parameter_mismatch(
            directive_name,
            max_positional_count,
            args.len()
        ));
    }

    // Validazione dei tipi per argomenti literali
    validate_literal_argument_types(args, parameters, directive_name)?;

    Ok(())
}

/// Validazione per argomenti named
pub fn validate_named_arguments(
    args: &[ArgDefinition],
    parameters: &[ParameterDefinition],
    directive_name: &str,
) -> LoomResult<()> {
    let param_map: HashMap<&str, &ParameterDefinition> = parameters
        .iter()
        .map(|p| (p.name.as_str(), p))
        .collect();

    let mut provided_params = HashSet::new();

    // Verifica che tutti gli argomenti named siano parametri validi
    for arg in args {
        if let ArgDefinition::Named { name, .. } = arg {
            if !param_map.contains_key(name.as_str()) {
                let available: Vec<&str> = param_map.keys().copied().collect();
                return Err(LoomError::definition_not_found(
                    name.to_string(),
                    available.iter().map(|s| s.to_string()).collect(),
                    Position::default() // Idealmente dovremmo avere la posizione dell'argomento
                ));
            }

            let param = param_map[name.as_str()];
            if !param.allow_named_parameter {
                return Err(LoomError::parameter_validation(
                    directive_name,
                    format!("Parameter '{}' cannot be used as named parameter", name)
                ));
            }

            if param.varargs {
                return Err(LoomError::parameter_validation(
                    directive_name,
                    format!("Varargs parameter '{}' cannot be used as named parameter", name)
                ));
            }

            if !provided_params.insert(name.as_str()) {
                return Err(LoomError::parameter_validation(
                    directive_name,
                    format!("Parameter '{}' specified multiple times", name)
                ));
            }
        }
    }

    // Verifica che tutti i parametri required siano presenti
    for param in parameters {
        if param.required && !provided_params.contains(param.name.as_str()) {
            return Err(LoomError::parameter_validation(
                directive_name,
                format!("Required parameter '{}' is missing", param.name)
            ));
        }
    }

    Ok(())
}

/// Validazione dei tipi per argomenti literali
pub fn validate_literal_argument_types(
    args: &[ArgDefinition],
    parameters: &[ParameterDefinition],
    directive_name: &str,
) -> LoomResult<()> {
    for (i, arg) in args.iter().enumerate() {
        if let ArgDefinition::Positional(Expression::Literal(literal)) = arg {
            // Per argomenti posizionali, trova il parametro corrispondente
            let param = if i < parameters.len() && !has_varargs_parameter(parameters) {
                &parameters[i]
            } else if has_varargs_parameter(parameters) && i >= parameters.len() - 1 {
                // È un argomento varargs
                parameters.last().unwrap()
            } else {
                continue; // Errore di numero argomenti già gestito
            };

            validate_literal_type(literal, &param.param_type, &param.name, directive_name)?;
        } else if let ArgDefinition::Named { name, value: Expression::Literal(literal) } = arg {
            let param = parameters.iter()
                .find(|p| p.name == name.as_ref())
                .unwrap(); // Già verificato che esista

            validate_literal_type(literal, &param.param_type, &param.name, directive_name)?;
        }
    }

    Ok(())
}

/// Validazione del tipo di un valore letterale
pub fn validate_literal_type(
    literal: &LiteralValue,
    expected_type: &ParameterType,
    param_name: &str,
    directive_name: &str,
) -> LoomResult<()> {
    let is_valid = match (literal, expected_type) {
        (LiteralValue::String(_), ParameterType::String) => true,
        (LiteralValue::Number(_), ParameterType::Number) => true,
        (LiteralValue::Float(_), ParameterType::Number) => true,
        (LiteralValue::Boolean(_), ParameterType::Boolean) => true,
        (LiteralValue::Array(arr), ParameterType::Array(inner_type)) => {
            // Validazione ricorsiva per array
            arr.iter().all(|item| {
                validate_literal_type(item, inner_type, param_name, directive_name).is_ok()
            })
        }
        (_, ParameterType::Json) => true, // JSON può accettare qualsiasi valore
        (LiteralValue::String(s), ParameterType::Enum(variants)) => {
            variants.contains(s)
        }
        _ => false,
    };

    if !is_valid {
        return Err(LoomError::parameter_validation(
            directive_name,
            format!(
                "Parameter '{}' expects type {:?} but got {:?}",
                param_name,
                expected_type,
                literal
            )
        ));
    }

    Ok(())
}

// Verifica se un parametro può essere usato come posizionale
pub fn can_be_positional(param: &ParameterDefinition) -> bool {
    // I parametri varargs e quelli che permettono solo named non possono essere posizionali in contesti misti
    !param.varargs
}

/// Verifica se c'è un parametro varargs
pub fn has_varargs_parameter(parameters: &[ParameterDefinition]) -> bool {
    parameters.iter().any(|p| p.varargs)
}

/// Validazione della conformità delle definizioni dei parametri
/// Questo metodo NON va aggiunto al trait DirectiveDefinition
pub fn validate_parameter_definitions(parameters: &[ParameterDefinition]) -> LoomResult<()> {
    if parameters.is_empty() {
        return Ok(());
    }

    // 1. Verifica unicità dei nomi
    let mut names = HashSet::new();
    for param in parameters {
        if !names.insert(&param.name) {
            return Err(LoomError::validation(
                format!("Duplicate parameter name: '{}'", param.name)
            ));
        }
    }

    // 2. Solo l'ultimo parametro può essere varargs
    let varargs_positions: Vec<usize> = parameters
        .iter()
        .enumerate()
        .filter_map(|(i, p)| if p.varargs { Some(i) } else { None })
        .collect();

    if varargs_positions.len() > 1 {
        return Err(LoomError::validation(
            "Only one parameter can be marked as varargs"
        ));
    }

    if let Some(&varargs_pos) = varargs_positions.first() {
        if varargs_pos != parameters.len() - 1 {
            return Err(LoomError::validation(
                "Varargs parameter must be the last parameter"
            ));
        }

        // Varargs non può avere default value (non ha senso)
        let varargs_param = &parameters[varargs_pos];
        if varargs_param.default_value.is_some() {
            return Err(LoomError::validation(
                format!("Varargs parameter '{}' cannot have a default value", varargs_param.name)
            ));
        }

        // Varargs non può essere required (è implicito che può essere vuoto)
        if varargs_param.required {
            return Err(LoomError::validation(
                format!("Varargs parameter '{}' cannot be marked as required", varargs_param.name)
            ));
        }
    }

    // 3. Per parametri posizionali: required deve venire prima di optional
    // (solo se non tutti sono named-only)
    let has_positional_params = parameters.iter().any(|p| can_be_positional(p));

    if has_positional_params {
        let mut found_optional = false;
        for param in parameters {
            if !can_be_positional(param) {
                continue; // Skip named-only parameters
            }

            if param.varargs {
                break; // Varargs è sempre l'ultimo, quindi ok
            }

            if !param.required {
                found_optional = true;
            } else if found_optional {
                return Err(LoomError::validation(
                    format!(
                        "Required parameter '{}' cannot come after optional parameters in positional context",
                        param.name
                    )
                ));
            }
        }
    }

    // 4. Validazione delle regole di validazione (se presenti)
    for param in parameters {
        if let Some(rules) = &param.validation_rules {
            validate_validation_rules(rules, &param.name, &param.param_type)?;
        }
    }

    Ok(())
}

/// Validazione delle regole di validazione
pub fn validate_validation_rules(
    rules: &ValidationRules,
    param_name: &str,
    param_type: &ParameterType,
) -> LoomResult<()> {
    // min_length e max_length solo per String e Array
    if rules.min_length.is_some() || rules.max_length.is_some() {
        match param_type {
            ParameterType::String | ParameterType::Array(_) => {}
            _ => {
                return Err(LoomError::validation(
                    format!(
                        "Parameter '{}': min_length/max_length can only be used with String or Array types",
                        param_name
                    )
                ));
            }
        }
    }

    // min_value e max_value solo per Number
    if rules.min_value.is_some() || rules.max_value.is_some() {
        if !matches!(param_type, ParameterType::Number) {
            return Err(LoomError::validation(
                format!(
                    "Parameter '{}': min_value/max_value can only be used with Number type",
                    param_name
                )
            ));
        }
    }

    // pattern solo per String
    if rules.pattern.is_some() {
        if !matches!(param_type, ParameterType::String) {
            return Err(LoomError::validation(
                format!(
                    "Parameter '{}': pattern validation can only be used with String type",
                    param_name
                )
            ));
        }
    }

    // Validazione coerenza min/max
    if let (Some(min), Some(max)) = (rules.min_length, rules.max_length) {
        if min > max {
            return Err(LoomError::validation(
                format!(
                    "Parameter '{}': min_length ({}) cannot be greater than max_length ({})",
                    param_name, min, max
                )
            ));
        }
    }

    if let (Some(min), Some(max)) = (rules.min_value, rules.max_value) {
        if min > max {
            return Err(LoomError::validation(
                format!(
                    "Parameter '{}': min_value ({}) cannot be greater than max_value ({})",
                    param_name, min, max
                )
            ));
        }
    }

    Ok(())
}