use crate::ast::Expression;

pub mod types;
pub mod ast;
pub mod context;
pub mod error;
pub mod definition;
pub mod interceptor;
pub mod event;

#[derive(Clone)]
pub struct InputArg {
    name: String,
    value: Option<Expression>,
}