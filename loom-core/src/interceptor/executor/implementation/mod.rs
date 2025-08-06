use crate::interceptor::InterceptorChain;
use crate::interceptor_result;

pub mod command;
pub mod composable;
pub mod definition;


pub fn empty_execute_intercept_next<'a>() -> Box<InterceptorChain<'a>> {
    Box::new(|_| interceptor_result!(Err("You are trying to call an empty interceptor chain".to_string())) )
}
