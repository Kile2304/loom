use crate::types::*;
use std::collections::HashMap;
use crate::context::{LoomContext, Module};
use crate::definition::ArgDefinition;
use crate::error::{LoomError, LoomResult, UndefinedKind};
use crate::interceptor::context::ExecutionContext;

/// A complete definition (recipe, job, pipeline, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub kind: DefinitionKind,
    pub signature: Signature,
    pub body: Vec<Block>,
    pub directives: Vec<DirectiveCall>,
    pub position: Position,
    pub module_index: usize,
}

/// Block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub directives: Vec<DirectiveCall>, // @if, @for, etc.
    pub label: Vec<Expression>, // Optional implicit (vec may be empty)
}

/// Individual statement in a block
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Shell command execution
    Command {
        parts: Vec<Expression>,
        directives: Vec<DirectiveCall>, // Direttive anche sui singoli comandi
    },

    /// Recipe/job call
    Call {
        name: String,
        args: Vec<Expression>,
        directives: Vec<DirectiveCall>, // Direttive anche sulle singole call
    },

}

/// Assignment targets
#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Variable(String),
    IndexAccess {
        object: String,
        index: Expression,
    },
}

/// Expression types
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Literal values
    Literal(LiteralValue),

    /// Variable reference
    Variable(String),

    /// Function call
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },

    /// Array/object index access
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    /// Binary operations
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },

    /// Unary operations
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },

    /// String interpolation
    Interpolation {
        parts: Vec<InterpolationPart>,
    },

    // C'è già IndecxAccess, ha davvero senso?
    /// Enum access (e.g., Environment["production"])
    EnumAccess {
        enum_name: String,
        variant: String,
    },
}


/// Parts of string interpolation
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Text(String),
    Expression(Expression),
}

// TODO: Non ancora integrati, prevedere di integrare in futuro
/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add, Subtract, Multiply, Divide, Modulo,

    // Comparison
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,

    // Logical
    And, Or,

    // String
    Contains, StartsWith, EndsWith,

    // Special
    Is, IsNot, // For "is empty", "is not empty"
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Minus,
}

/// Directive call (e.g., @doc, @parallel, @timeout)
#[derive(Debug, Clone, PartialEq)]
pub struct DirectiveCall {
    pub name: String,
    pub args: Vec<ArgDefinition>,
    pub position: Position,
}

/// Directive argument
#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveArg {
    Positional(Expression),
    Named {
        name: String,
        value: Expression,
    },
}

impl Block {

    pub fn new(statements: Vec<Statement>, directives: Vec<DirectiveCall>, label: Vec<Expression>) -> Self {
        Self {
            statements, directives, label
        }
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }
}
impl Expression {

    /// Helper method to evaluate an expression into a LoomValue
    pub fn evaluate(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        position: Option<Position>,
    ) -> LoomResult<LoomValue> {
        match self {
            Expression::Literal(lit) => Ok(LoomValue::Literal(lit.clone())),

            Expression::Variable(var_name) => {
                context.get_variable(var_name)
                    .ok_or_else(|| {
                        if let Some(pos) = position {
                            LoomError::undefined(
                                var_name,
                                UndefinedKind::Variable,
                                pos
                            )
                        } else {
                            LoomError::execution(format!("Variable '{}' not found", var_name))
                        }
                    })
            }

            Expression::FunctionCall { name, args } => {
                // ✅ Invece di panic!, usa errore appropriato
                Err(LoomError::not_implemented(
                    "function calls",
                    format!("Function '{}' with {} arguments", name, args.len())
                ))
            }

            Expression::IndexAccess { object, index } => {
                let obj_value = object.evaluate(loom_context, context, position.clone())?;
                let index_value = index.evaluate(loom_context, context, position.clone())?;

                match (&obj_value, &index_value) {
                    (LoomValue::Literal(LiteralValue::Array(arr)),
                        LoomValue::Literal(LiteralValue::Number(idx))) => {
                        let idx = *idx as usize;
                        arr.get(idx)
                            .cloned()
                            .map(LoomValue::Literal)
                            .ok_or_else(|| {
                                LoomError::execution(format!(
                                    "Array index {} out of bounds (length: {})",
                                    idx, arr.len()
                                ))
                            })
                    }
                    _ => Err(LoomError::expression(
                        "index_access",
                        format!("Cannot index {:?} with {:?}", obj_value.type_name(), index_value.type_name()),
                        position.unwrap_or_default()
                    ))
                }
            }

            Expression::BinaryOp { left, operator, right } => {
                Self::evaluate_binary_op(left, operator, right, loom_context, context, position)
            }

            Expression::UnaryOp { operator, operand } => {
                let value = operand.evaluate(loom_context, context, position.clone())?;
                match (operator, &value) {
                    (UnaryOperator::Not, LoomValue::Literal(LiteralValue::Boolean(b))) => {
                        Ok(LoomValue::Literal(LiteralValue::Boolean(!b)))
                    }
                    (UnaryOperator::Minus, LoomValue::Literal(LiteralValue::Number(n))) => {
                        Ok(LoomValue::Literal(LiteralValue::Number(-n)))
                    }
                    (UnaryOperator::Minus, LoomValue::Literal(LiteralValue::Float(f))) => {
                        Ok(LoomValue::Literal(LiteralValue::Float(-f)))
                    }
                    _ => Err(LoomError::expression(
                        "unary_operation",
                        format!("Cannot apply {:?} to {:?}", operator, value.type_name()),
                        position.unwrap_or_default()
                    ))
                }
            }

            Expression::EnumAccess { enum_name, variant } => {
                let en = loom_context.find_enum(enum_name.as_str())
                    .ok_or_else(|| {
                        if let Some(pos) = &position {
                            LoomError::undefined(enum_name, UndefinedKind::Enum, pos.clone())
                        } else {
                            LoomError::execution(format!("Enum '{}' not found", enum_name))
                        }
                    })?;

                en.variants.get(variant.as_str())
                    .cloned()
                    .map(|value| LoomValue::Literal(LiteralValue::String(value)))
                    .ok_or_else(|| {
                        if let Some(pos) = position {
                            LoomError::undefined(
                                format!("{}::{}", enum_name, variant),
                                UndefinedKind::EnumVariant,
                                pos
                            )
                        } else {
                            LoomError::execution(format!(
                                "Enum '{}' doesn't contain variant '{}'. Available: [{}]",
                                enum_name,
                                variant,
                                en.variants.keys().map(|it| it.to_string()).collect::<Vec<_>>().join(", ")
                            ))
                        }
                    })
            }

            Expression::Interpolation { parts } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        InterpolationPart::Text(t) => result.push_str(t),
                        InterpolationPart::Expression(expr) => {
                            let value = expr.evaluate(loom_context, context, position.clone())?;
                            let string_value = value.stringify(loom_context, context)
                                .map_err(|e| LoomError::expression(
                                    "string_interpolation",
                                    format!("Failed to stringify expression in interpolation: {}", e),
                                    position.clone().unwrap_or_default()
                                ))?;
                            result.push_str(&string_value);
                        }
                    }
                }
                Ok(LoomValue::Literal(LiteralValue::String(result)))
            }
        }
    }

    /// Helper to evaluate binary operations with better error handling
    fn evaluate_binary_op(
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        position: Option<Position>,
    ) -> LoomResult<LoomValue> {
        let left_val = left.evaluate(loom_context, context, position.clone())?;
        let right_val = right.evaluate(loom_context, context, position.clone())?;

        match (&left_val, &right_val) {
            (LoomValue::Literal(left_val), LoomValue::Literal(right_val)) => {
                Self::evaluate_literal_binary_op(left_val, operator, right_val, position)
            }
            _ => Err(LoomError::expression(
                "binary_operation",
                format!(
                    "Binary operation {:?} not supported between {} and {}",
                    operator, left_val.type_name(), right_val.type_name()
                ),
                position.unwrap_or_default()
            ))
        }
    }

    fn evaluate_literal_binary_op(
        left: &LiteralValue,
        operator: &BinaryOperator,
        right: &LiteralValue,
        position: Option<Position>,
    ) -> LoomResult<LoomValue> {
        use BinaryOperator::*;
        use LiteralValue::*;

        let pos = position.unwrap_or_default();

        match (left, operator, right) {
            // Arithmetic operations
            (Number(a), Add, Number(b)) => Ok(LoomValue::Literal(Number(a + b))),
            (Float(a), Add, Float(b)) => Ok(LoomValue::Literal(Float(a + b))),
            (Number(a), Add, Float(b)) => Ok(LoomValue::Literal(Float(*a as f64 + b))),
            (Float(a), Add, Number(b)) => Ok(LoomValue::Literal(Float(a + *b as f64))),
            (String(a), Add, String(b)) => Ok(LoomValue::Literal(String(format!("{}{}", a, b)))),

            (Number(a), Subtract, Number(b)) => Ok(LoomValue::Literal(Number(a - b))),
            (Float(a), Subtract, Float(b)) => Ok(LoomValue::Literal(Float(a - b))),
            (Number(a), Subtract, Float(b)) => Ok(LoomValue::Literal(Float(*a as f64 - b))),
            (Float(a), Subtract, Number(b)) => Ok(LoomValue::Literal(Float(a - *b as f64))),

            (Number(a), Multiply, Number(b)) => Ok(LoomValue::Literal(Number(a * b))),
            (Float(a), Multiply, Float(b)) => Ok(LoomValue::Literal(Float(a * b))),
            (Number(a), Multiply, Float(b)) => Ok(LoomValue::Literal(Float(*a as f64 * b))),
            (Float(a), Multiply, Number(b)) => Ok(LoomValue::Literal(Float(a * *b as f64))),

            (Number(a), Divide, Number(b)) => {
                if *b == 0 {
                    Err(LoomError::expression("division", "Division by zero", pos))
                } else {
                    Ok(LoomValue::Literal(Number(a / b)))
                }
            }
            (Float(a), Divide, Float(b)) => {
                if *b == 0.0 {
                    Err(LoomError::expression("division", "Division by zero", pos))
                } else {
                    Ok(LoomValue::Literal(Float(a / b)))
                }
            }

            // Comparison operations
            (a, Equal, b) => Ok(LoomValue::Literal(Boolean(a == b))),
            (a, NotEqual, b) => Ok(LoomValue::Literal(Boolean(a != b))),

            (Number(a), Less, Number(b)) => Ok(LoomValue::Literal(Boolean(a < b))),
            (Float(a), Less, Float(b)) => Ok(LoomValue::Literal(Boolean(a < b))),
            (String(a), Less, String(b)) => Ok(LoomValue::Literal(Boolean(a < b))),

            // String operations
            (String(s), Contains, String(sub)) => {
                Ok(LoomValue::Literal(Boolean(s.contains(sub))))
            }
            (String(s), StartsWith, String(prefix)) => {
                Ok(LoomValue::Literal(Boolean(s.starts_with(prefix))))
            }
            (String(s), EndsWith, String(suffix)) => {
                Ok(LoomValue::Literal(Boolean(s.ends_with(suffix))))
            }

            // Boolean operations
            (Boolean(a), And, Boolean(b)) => Ok(LoomValue::Literal(Boolean(*a && *b))),
            (Boolean(a), Or, Boolean(b)) => Ok(LoomValue::Literal(Boolean(*a || *b))),

            // Invalid operations
            _ => Err(LoomError::expression(
                "binary_operation",
                format!(
                    "Operator {:?} not supported between {:?} and {:?}",
                    operator,
                    std::mem::discriminant(left),
                    std::mem::discriminant(right)
                ),
                pos
            ))
        }
    }

}