use std::collections::HashMap;
use std::sync::Arc;
use crate::error::{LoomError, LoomResult};
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::global::ActiveGlobalInterceptor;
use crate::interceptor::global::config::GlobalInterceptorConfig;
use crate::interceptor::global::interceptor::GlobalInterceptor;
use crate::interceptor::priority::PriorityRanges;
use crate::loom_error;

/// Manager per interceptor globali
pub struct GlobalInterceptorManager {
    interceptors: HashMap<String, Arc<dyn GlobalInterceptor>>,
    configs: HashMap<String, GlobalInterceptorConfig>,
    user_overrides: HashMap<String, bool>,
}

impl GlobalInterceptorManager {
    pub fn new() -> Self {
        Self {
            interceptors: HashMap::new(),
            configs: HashMap::new(),
            user_overrides: HashMap::new(),
        }
    }

    pub fn register(&mut self, interceptor: Arc<dyn GlobalInterceptor>) -> LoomResult<()> {
        let name = interceptor.name().to_string();
        let config = interceptor.default_config();

        // Valida che la priorità sia nel range corretto per interceptor globali
        self.validate_global_priority(config.priority)?;

        self.interceptors.insert(name.clone(), interceptor);
        self.configs.insert(name, config);
        Ok(())
    }

    pub fn configure(&mut self, name: &str, config: GlobalInterceptorConfig) -> LoomResult<()> {
        if !self.interceptors.contains_key(name) {
            return loom_error!("Global interceptor '{}' not found", name);
        }

        self.validate_global_priority(config.priority)?;
        self.configs.insert(name.to_string(), config);
        Ok(())
    }

    pub fn set_user_override(&mut self, name: &str, enabled: bool) -> LoomResult<()> {
        let config = self.configs.get(name)
            .ok_or_else(|| LoomError::execution(format!("Global interceptor '{}' not found", name)))?;

        if !config.user_overridable {
            return loom_error!("Global interceptor '{}' cannot be overridden", name);
        }

        self.user_overrides.insert(name.to_string(), enabled);
        Ok(())
    }

    /// Ottieni interceptor attivi per un contesto
    pub fn get_active(&self, context: &ExecutionContext) -> Vec<ActiveGlobalInterceptor> {
        let mut active = Vec::new();

        for (name, interceptor) in &self.interceptors {
            let mut config = self.configs.get(name).unwrap().clone();

            // Applica override utente
            if let Some(&user_enabled) = self.user_overrides.get(name) {
                config.enabled = user_enabled;
            }

            // Controlla se dovrebbe attivarsi
            if interceptor.should_activate(context, &config) {
                active.push(ActiveGlobalInterceptor {
                    interceptor: interceptor.clone(),
                    config,
                    name: name.clone(),
                });
            }
        }

        // Ordina per priorità
        active.sort_by(|a, b| b.config.priority.cmp(&a.config.priority));

        active
    }

    fn validate_global_priority(&self, priority: i32) -> LoomResult<()> {
        let valid_ranges = [
            PriorityRanges::CRITICAL_SYSTEM,
            PriorityRanges::GLOBAL_HIGH,
            PriorityRanges::GLOBAL_NORMAL,
            PriorityRanges::GLOBAL_SUPPORT,
            PriorityRanges::MONITORING,
        ];

        let is_valid = valid_ranges.iter().any(|range| range.contains(&priority));

        if !is_valid {
            return loom_error!(
                "Global interceptor priority {} is not in valid range. Use: CRITICAL_SYSTEM (9000-10000), GLOBAL_HIGH (8000-9000), GLOBAL_NORMAL (5000-7000), GLOBAL_SUPPORT (1000-3000), MONITORING (0-500)",
                priority
            );
        }

        Ok(())
    }
}