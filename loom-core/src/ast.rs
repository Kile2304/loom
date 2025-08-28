use crate::types::*;
use std::collections::HashMap;
use crate::context::{LoomContext, Module};
use crate::definition::ArgDefinition;
use crate::error::{LoomError, LoomResult};
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
    ) -> LoomResult<LoomValue> {
        match self {
            Expression::Literal(lit) => Ok(LoomValue::Literal(lit.clone())),
            Expression::Variable(var_name) => {
                // Look up variable in context
                context.get_variable(var_name)
                    .ok_or_else(|| LoomError::execution(format!("Variable '{}' not found", var_name)))
            }
            Expression::FunctionCall { name, args } => {
                // self.evaluate_function_call(name, args, loom_context, context)
                panic!("Not implemented yet!")
            }
            Expression::BinaryOp { left, operator, right } => {
                Self::evaluate_binary_op(left, operator, right, loom_context, context)
            }
            Expression::EnumAccess { enum_name, variant} => {
                let en = loom_context.find_enum(enum_name.as_str())
                    .ok_or_else(|| LoomError::execution(format!("Enum '{}' not found", enum_name)))?;
                en.variants.get(variant.as_str()).clone()
                    .ok_or_else(|| LoomError::execution(format!("Enum '{}' don't contain variant '{}'", enum_name, variant)))
                    .map(|it| LoomValue::Literal(LiteralValue::String(it.to_string())))
            }
            Expression::Interpolation { parts } => {
                let joined = parts.iter()
                    .map(|it|
                        match it {
                            InterpolationPart::Text(t) => Ok(t.to_string()),
                            InterpolationPart::Expression(e) => {
                                match e.evaluate(loom_context, context) {
                                    Ok(LoomValue::Literal(lit)) => Ok(lit.stringify()),
                                    Err(err) => Err(err),
                                    x => Err(LoomError::execution(format!("Interpolation '{:?}' does not contain a literal value", x))),
                                }
                            },
                        }
                    )
                .collect::<LoomResult<Vec<_>>>()?
                    .join("");

                Ok(LoomValue::Literal(LiteralValue::String(joined)))
            }
            // TODO: Manca IndexAccess
            // Add other expression types as needed
            _ => Err(LoomError::execution("Unsupported expression type")),
        }
    }

    /// Helper to evaluate binary operations
    fn evaluate_binary_op(
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        loom_context: &LoomContext,
        context: &ExecutionContext,
    ) -> LoomResult<LoomValue> {
        let left_val = left.evaluate(loom_context, context)?;
        let right_val = right.evaluate(loom_context, context)?;

        match (&left_val, &right_val) {
            (LoomValue::Literal(left_val), LoomValue::Literal(right_val)) => {
                match operator {
                    BinaryOperator::Add => match (left_val, right_val) {
                        (LiteralValue::Number(a), LiteralValue::Number(b)) => Ok(LoomValue::Literal(LiteralValue::Number(a + b))),
                        (LiteralValue::Float(a), LiteralValue::Float(b)) => Ok(LoomValue::Literal(LiteralValue::Float(a + b))),
                        (LiteralValue::String(a), LiteralValue::String(b)) => Ok(LoomValue::Literal(LiteralValue::String(format!("{a}{b}")))),
                        _ => Err(LoomError::execution("Invalid operands for +")),
                    },
                    BinaryOperator::Subtract => match (left_val, right_val) {
                        (LiteralValue::Number(a), LiteralValue::Number(b)) => Ok(LoomValue::Literal(LiteralValue::Number(a - b))),
                        (LiteralValue::Float(a), LiteralValue::Float(b)) => Ok(LoomValue::Literal(LiteralValue::Float(a - b))),
                        _ => Err(LoomError::execution("Invalid operands for -")),
                    },
                    BinaryOperator::Multiply => match (left_val, right_val) {
                        (LiteralValue::Number(a), LiteralValue::Number(b)) => Ok(LoomValue::Literal(LiteralValue::Number(a * b))),
                        (LiteralValue::Float(a), LiteralValue::Float(b)) => Ok(LoomValue::Literal(LiteralValue::Float(a * b))),
                        _ => Err(LoomError::execution("Invalid operands for *")),
                    },
                    BinaryOperator::Divide => match (left_val, right_val) {
                        (LiteralValue::Number(a), LiteralValue::Number(b)) => {
                            if *b == 0 {
                                Err(LoomError::execution("Division by zero"))
                            } else {
                                Ok(LoomValue::Literal(LiteralValue::Number(a / b)))
                            }
                        },
                        (LiteralValue::Float(a), LiteralValue::Float(b)) => {
                            if *b == 0.0 {
                                Err(LoomError::execution("Division by zero"))
                            } else {
                                Ok(LoomValue::Literal(LiteralValue::Float(a / b)))
                            }
                        },
                        _ => Err(LoomError::execution("Invalid operands for /")),
                    },
                    BinaryOperator::Equal => Ok(LoomValue::Literal(LiteralValue::Boolean(left_val == right_val))),
                    BinaryOperator::NotEqual => Ok(LoomValue::Literal(LiteralValue::Boolean(left_val != right_val))),
                    // Add more operators as needed
                    _ => Err(LoomError::execution(format!("Unknown operator '{:?}'", operator))),
                }
            }
            _ => {
                Err(LoomError::execution(format!("Uno tra {:?} e {:?} non è un Literal", left_val, right_val)))
            }
        }
    }

}