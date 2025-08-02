pub mod interceptor;
mod config;
pub mod implementation;

use std::sync::Arc;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::interceptor::ExecutorInterceptor;

/// Interceptor globale attivo con la sua configurazione
#[derive(Clone)]
pub struct ActiveExecutorInterceptor {
    pub interceptor: Arc<dyn ExecutorInterceptor>,
    pub config: ExecutorConfig,
    pub name: String,
}

impl ActiveExecutorInterceptor {
    pub fn new(executor: Arc<dyn ExecutorInterceptor>) -> Self {
        Self {
            name: executor.name().to_string(),
            config: ExecutorConfig::default(),
            interceptor: executor,
        }
    }
}