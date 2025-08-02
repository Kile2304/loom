use std::sync::Arc;
use crate::interceptor::global::config::GlobalInterceptorConfig;
use crate::interceptor::global::interceptor::GlobalInterceptor;

pub mod interceptor;
pub mod config;
pub mod manager;

/// Categorie di interceptor globali
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GlobalInterceptorCategory {
    /// Sicurezza e compliance
    Security,
    /// Monitoring e observability
    Monitoring,
    /// Performance e optimization
    Performance,
    /// Development e debugging
    Development,
    /// Business rules
    Business,
    /// Generale
    General,
}
/// Interceptor globale attivo con la sua configurazione
#[derive(Clone)]
pub struct ActiveGlobalInterceptor {
    pub interceptor: Arc<dyn GlobalInterceptor>,
    pub config: GlobalInterceptorConfig,
    pub name: String,
}