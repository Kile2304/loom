use std::collections::HashMap;
use std::sync::Arc;
use crate::interceptor::directive::interceptor::DirectiveInterceptor;
use crate::types::LoomValue;

pub mod interceptor;
pub mod manager;

/// Interceptor di direttiva attivo con i suoi parametri
#[derive(Clone)]
pub struct ActiveDirectiveInterceptor {
    pub interceptor: Arc<dyn DirectiveInterceptor>,
    pub params: HashMap<String, LoomValue>,
    pub name: String,
    pub priority: i32,
}