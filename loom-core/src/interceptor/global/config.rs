use std::collections::HashMap;
use crate::interceptor::global::interceptor::ActivationCondition;

/// Configurazione per interceptor globali
#[derive(Debug, Clone, Default)]
pub struct GlobalInterceptorConfig {
    /// Se l'interceptor è abilitato
    pub enabled: bool,
    /// Priorità (più alta = eseguita prima)
    pub priority: i32,
    /// Condizioni per l'attivazione
    pub conditions: Vec<ActivationCondition>,
    /// Parametri di configurazione
    pub parameters: HashMap<String, serde_json::Value>,
    /// Se può essere disabilitato dall'utente
    pub user_overridable: bool,
}