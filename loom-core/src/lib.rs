pub mod types;
pub mod ast;
pub mod context;
pub mod error;
pub mod definition;
pub mod interceptor;
pub mod event;

pub struct InputArg {
    name: String,
    value: Option<String>,
}