use std::collections::HashMap;
use serde_json::Value;
use crate::ast::Expression;
use crate::context::LoomContext;
use crate::InputArg;
use crate::interceptor::context::ExecutionContext;

#[derive(Debug, Clone, PartialEq)]
pub enum LoomValue {
    Literal(LiteralValue),
    Expression(Box<Expression>),
    Empty,
}

impl LoomValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            LoomValue::Literal(_) => "literal",
            LoomValue::Expression(_) => "expression",
            LoomValue::Empty => "empty",
        }
    }
}

/// Types of executable definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefinitionKind {
    Recipe,
    Job,
    Pipeline,
    Schedule,
    Plugin,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: HashMap<String, String>,
}

/// Variable assignment
#[derive(Debug, Clone, PartialEq)]
pub struct VariableAssignment {
    pub name: String,
    pub value: Expression,
}

/// Parameter definition with unevaluated expressions (for parsing/definition phase)
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterDefinition {
    pub name: String,
    pub param_type: Option<String>,
    pub default_value: Option<Expression>, // Unevaluated expression
    pub required: bool,
}

/// Function signature with parameter definitions
#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub name: String,
    pub parameters: Vec<ParameterDefinition>, // Use ParameterDefinition here
}

#[derive(Debug, Default)]
pub enum ParallelizationKind {
    Parallel {
        max_thread: u8,
    },
    #[default]
    Sequential,
}

/// Position information for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            file: None,
        }
    }
}

impl Signature {

    pub fn args_into_variable(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        args: Vec<InputArg>,
    ) -> Result<Vec<(String, LoomValue)>, String> {
        args.iter()
            .map(|arg|
                 (arg, self.parameters.iter().find(|param| param.name == arg.name))
            ).filter(|(_, p)| p.is_some())
            .map(|(v1, v2)| (v1, v2.unwrap()))
            .map(|(v1, v2)|
                v2.value_from_arg(v1.value.as_ref(), loom_context, context)
                    .map(|it| (v2.name.to_string(), it))
            )
        .collect::<Result<Vec<_>, _>>()
    }

}

impl ParameterDefinition {

    // TODO: Potrebbe essere il caso di convertire queste stringhe in costanti!
    pub fn value_from_arg(
        &self,
        value: Option<&String>,
        loom_context: &LoomContext,
        context: &ExecutionContext,
    ) -> Result<LoomValue, String> {
        match value {
            Some(value) => {
                if let Some(param_type) = &self.param_type {
                    Ok(
                        LoomValue::Literal(
                            match param_type.as_str() {
                                "bool" => {
                                    match value.to_lowercase().as_str() {
                                        "false" => Ok(LiteralValue::Boolean(false)),
                                        "true" => Ok(LiteralValue::Boolean(true)),
                                        other => Err(format!("The parameter '{}' is expected as a boolean, but, '{}' is not a boolean", self.name, other)),
                                    }?
                                }
                                "number" => {
                                    value.parse::<i64>()
                                        .map(|it| LiteralValue::Number(it))
                                        .map_err(|_|
                                            format!(
                                                "Il parametro '{}' è tipizzato come numero ma '{}' non è un numero valido"
                                                , self.name, value
                                            )
                                        )?
                                }
                                "float" => {
                                    value.parse::<f64>()
                                        .map(|it| LiteralValue::Float(it))
                                        .map_err(|_|
                                            format!(
                                                "Il parametro '{}' è tipizzato come float ma '{}' non è un float valido"
                                                , self.name, value
                                            )
                                        )?
                                }
                                "string" => {
                                    LiteralValue::String(value.to_string())
                                }
                                // Enumerator type
                                other => {
                                    let en = loom_context.find_enum(other).unwrap();
                                    en.variants.get(value)
                                        .map(|it| LiteralValue::String(it.to_string()))
                                    .ok_or_else(||
                                        format!(
                                            "Il parametro '{}' è tipizzato come enum e '{}' non è uno dei valori attesi.\nValori attesi: {:?}"
                                            , self.name, value, en.variants.keys()
                                        )
                                    )?
                                }
                            }
                        )
                    )
                } else {
                    Ok(LoomValue::Literal(LiteralValue::String(value.to_string())))
                }
            }
            None => {
                match &self.param_type {
                    None => Ok(LoomValue::Literal(LiteralValue::Boolean(true))),
                    Some(param_type) => {
                        if param_type.as_str() == "bool" {
                            Ok(LoomValue::Literal(LiteralValue::Boolean(true)))
                        } else {
                            self.default_value
                                .as_ref()
                            .ok_or_else(|| format!("No default value for parameter {} and no value provided", self.name))?
                                .evaluate(loom_context, context)
                        }
                    }
                }
            }
        }
    }

    /// Evaluates the parameter definition and returns (param_name, Option<LoomValue>)
    /// Returns None when:
    /// - No default value is provided and parameter is not required
    /// - Expression evaluation fails
    pub fn evaluate(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
    ) -> (String, Option<LoomValue>) {
        let value = match &self.default_value {
            Some(expr) => {
                // Try to evaluate the expression
                match expr.evaluate(loom_context, context) {
                    Ok(loom_value) => Some(loom_value),
                    Err(_) => {
                        // Log the error if needed, but return None
                        // You might want to log this somewhere
                        None
                    }
                }
            }
            None => {
                // No default value provided
                if self.required {
                    // For required parameters without defaults, you might want to handle this differently
                    // For now, returning None - you could also return Some(LoomValue::Empty)
                    None
                } else {
                    // Optional parameter without default
                    None
                }
            }
        };

        (self.name.clone(), value)
    }

    /// Helper to evaluate function calls
    fn evaluate_function_call(
        &self,
        name: &str,
        args: &[Expression],
        loom_context: &LoomContext,
        context: &ExecutionContext,
    ) -> Result<LoomValue, String> {
        // Evaluate all arguments first
        let mut evaluated_args = Vec::new();
        for arg in args {
            evaluated_args.push(arg.evaluate(loom_context, context)?);
        }

        // TODO: Prendere da modulo esterno...

        // Call the function with evaluated arguments
        match name {
            "env" => {
                // Example: env("VAR_NAME") - get environment variable
                if evaluated_args.len() != 1 {
                    return Err("env() requires exactly one argument".to_string());
                }
                if let LoomValue::Literal(LiteralValue::String(var_name)) = &evaluated_args[0] {
                    match std::env::var(var_name) {
                        Ok(value) => Ok(LoomValue::Literal(LiteralValue::String(value))),
                        Err(_) => Ok(LoomValue::Empty),
                    }
                } else {
                    Err("env() argument must be a string".to_string())
                }
            }
            "concat" => {
                // Example: concat("a", "b") - concatenate strings
                let mut result = String::new();
                for arg in evaluated_args {
                    match arg {
                        LoomValue::Literal(LiteralValue::String(s)) => result.push_str(&s),
                        other => result.push_str(&format!("{:?}", other)), // Convert to string representation
                    }
                }
                Ok(LoomValue::Literal(LiteralValue::String(result)))
            }
            "default" => {
                // Example: default(var, "fallback") - return first non-empty value
                for arg in evaluated_args {
                    match &arg {
                        LoomValue::Empty => continue,
                        LoomValue::Literal(LiteralValue::String(s)) if s.is_empty() => continue,
                        _ => return Ok(arg),
                    }
                }
                Ok(LoomValue::Empty)
            }
            // Add more built-in functions as needed
            _ => {
                // Try to call user-defined function from context
                loom_context.call_function(name, evaluated_args)
                    // .or_else(|| context.call_function(name, evaluated_args))
                    // .ok_or_else(|| format!("Unknown function '{}'", name))
            }
        }
    }

}

// Esempio di utilizzo con il nuovo metodo evaluate
impl Signature {
    /// Evaluate all parameter definitions with provided arguments
    pub fn evaluate_with_args(
        &self,
        args: &HashMap<String, LoomValue>,
        loom_context: &LoomContext,
        context: &ExecutionContext,
    ) -> HashMap<String, LoomValue> {
        let mut result = HashMap::new();

        for param_def in &self.parameters {
            let (param_name, default_value) = param_def.evaluate(loom_context, context);

            let final_value = if let Some(provided_value) = args.get(&param_name) {
                // Use provided argument
                provided_value.clone()
            } else if let Some(default_val) = default_value {
                // Use evaluated default value
                default_val
            } else {
                // No value provided and no default - use Empty or skip
                LoomValue::Empty
            };

            result.insert(param_name, final_value);
        }

        result
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<LiteralValue>),
    Json(Value),
}

impl LiteralValue {

    pub fn stringify(&self) -> String {
        match self {
            LiteralValue::String(v) => v.to_string(),
            LiteralValue::Number(v) => v.to_string(),
            LiteralValue::Float(v) => v.to_string(),
            LiteralValue::Boolean(v) => v.to_string(),
            LiteralValue::Array(v) =>
                format!("[{}]", v.iter().map(|it| it.stringify()).collect::<Vec<_>>().join(", ")),
            LiteralValue::Json(v) => v.to_string(),
        }
    }

}