use std::collections::HashMap;
use std::sync::Arc;
use crate::ast::DirectiveCall;
use crate::context::LoomContext;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::directive::ActiveDirectiveInterceptor;
use crate::interceptor::directive::interceptor::DirectiveInterceptor;
use crate::interceptor::priority::PriorityRanges;

// Manager per interceptor di direttive
pub struct DirectiveInterceptorManager {
    interceptors: HashMap<String, Arc<dyn DirectiveInterceptor>>,
}

impl DirectiveInterceptorManager {
    pub fn new() -> Self {
        Self {
            interceptors: HashMap::new(),
        }
    }

    pub fn register(&mut self, interceptor: Arc<dyn DirectiveInterceptor>) -> Result<(), String> {
        let name = interceptor.directive_name().to_string();
        let priority = interceptor.priority();

        // Valida che la priorità sia nel range corretto per direttive
        self.validate_directive_priority(priority)?;

        self.interceptors.insert(name, interceptor);
        Ok(())
    }

    /// Costruisce interceptor attivi da DirectiveCall
    pub fn build_active(
        &self,
        loom_context: &LoomContext,
        context: &ExecutionContext,
        directives: &[DirectiveCall]
    ) -> Result<Vec<ActiveDirectiveInterceptor>, String> {
        let mut active = Vec::new();

        for directive in directives {
            let interceptor = self.interceptors.get(&directive.name)
                .ok_or_else(|| format!("Unknown directive: {}", directive.name))?;

            let params = interceptor.parse_parameters(loom_context, context, directive)?;

            active.push(ActiveDirectiveInterceptor {
                interceptor: interceptor.clone(),
                params,
                name: directive.name.clone(),
                priority: interceptor.priority(),
            });
        }

        // Ordina per priorità
        active.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(active)
    }

    fn validate_directive_priority(&self, priority: i32) -> Result<(), String> {
        let valid_ranges = [
            PriorityRanges::DIRECTIVE_HIGH,
            PriorityRanges::DIRECTIVE_NORMAL,
            PriorityRanges::DIRECTIVE_SUPPORT,
        ];

        let is_valid = valid_ranges.iter().any(|range| range.contains(&priority));

        if !is_valid {
            return Err(format!(
                "Directive interceptor priority {} is not in valid range. Use: DIRECTIVE_HIGH (7000-8000), DIRECTIVE_NORMAL (3000-5000), DIRECTIVE_SUPPORT (500-1000)",
                priority
            ));
        }

        Ok(())
    }
}