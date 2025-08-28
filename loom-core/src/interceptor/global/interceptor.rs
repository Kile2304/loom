use std::sync::Arc;
use crate::interceptor::context::{ExecutionContext, InterceptorContext};
use crate::interceptor::global::config::GlobalInterceptorConfig;
use crate::interceptor::global::GlobalInterceptorCategory;
use crate::interceptor::{InterceptorChain, InterceptorResult};
use crate::interceptor::result::ExecutionResult;
use crate::interceptor::scope::{ExecutionActivity, ExecutionScope};

#[async_trait::async_trait]
/// Trait per interceptor globali
pub trait GlobalInterceptor: Send + Sync {
    /// Nome dell'interceptor
    fn name(&self) -> &str;

    /// Descrizione per debug/help
    fn description(&self) -> &str;

    /// Configurazione di default
    fn default_config(&self) -> GlobalInterceptorConfig;

    /// Controlla se dovrebbe attivarsi per questo contesto
    fn should_activate(&self, context: &ExecutionContext, config: &GlobalInterceptorConfig) -> bool {
        if !config.enabled {
            return false;
        }

        // Valuta condizioni di attivazione
        for condition in &config.conditions {
            if !self.evaluate_condition(condition, context) {
                return false;
            }
        }

        true
    }

    /// Intercetta l'esecuzione (stesso pattern degli interceptor normali)
    async fn intercept(
        &self,
        context: InterceptorContext<'_>,
        config: &GlobalInterceptorConfig,
        next: Box<InterceptorChain<'_>>,
    ) -> InterceptorResult;

    /// Valuta una condizione di attivazione
    fn evaluate_condition(&self, condition: &ActivationCondition, context: &ExecutionContext) -> bool {
        match condition {
            ActivationCondition::TargetType(types) => {
                let target_type = match &context.scope {
                    ExecutionScope::Command => "command",
                    ExecutionScope::Pipeline => "pipeline",
                    ExecutionScope::Job => "job",
                    // ExecutionTarget::Definition { kind, .. } => match kind {
                    //     DefinitionKind::Recipe => "recipe",
                    //     DefinitionKind::Job => "job",
                    //     DefinitionKind::Pipeline => "pipeline",
                    //     DefinitionKind::Schedule => "schedule",
                    //     _ => "other",
                    // },
                    _ => "other",
                };
                types.contains(&target_type.to_string())
            }
            ActivationCondition::Environment(envs) => {
                const DEVELOPMENT: &str = "development";
                let current_env = context.env_vars.get("LOOM_ENV").map(|it| it.as_str())
                    .or_else(|| context.env_vars.get("ENVIRONMENT").map(|it| it.as_str()))
                .unwrap_or(DEVELOPMENT)
                    .to_string();
                envs.contains(&current_env)
            }
            ActivationCondition::CommandPattern(regex) => {
                // if let ExecutionActivity::Command (c) = &context.target {
                //     // let cmd_str = c.command.join(" ");
                //     // regex.is_match(&cmd_str)
                //     // TODO: Sistemare
                //     false
                // } else {
                //     false
                // }
                // TODO: Rivalutare
                false
            }
            ActivationCondition::Workspace(workspaces) => {
                let current_workspace = context.working_dir
                    .as_ref()
                    .and_then(|wd| std::path::Path::new(wd).file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");
                workspaces.contains(&current_workspace.to_string())
            }
            ActivationCondition::TimeWindow { start, end } => {
                // Implementazione semplificata - in pratica useresti chrono
                let now = chrono::Local::now().time();
                let start_time =
                    chrono::NaiveTime::parse_from_str(start, "%H:%M")
                        .unwrap_or(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let end_time =
                    chrono::NaiveTime::parse_from_str(end, "%H:%M")
                        .unwrap_or(chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap());
                now >= start_time && now <= end_time
            }
            ActivationCondition::Custom(expr) => {
                // Placeholder per valutazione di espressioni custom
                // In pratica implementeresti un expression evaluator
                true
            }
        }
    }

    fn need_chain(&self) -> bool;

    /// Categoria dell'interceptor (per organizing/UI)
    fn category(&self) -> GlobalInterceptorCategory {
        GlobalInterceptorCategory::General
    }
}

/// Condizioni di attivazione per interceptor globali
#[derive(Debug, Clone)]
pub enum ActivationCondition {
    /// Solo per determinati tipi di target
    TargetType(Vec<String>), // ["command", "pipeline", "job"]
    /// Solo per determinati environment
    Environment(Vec<String>), // ["production", "staging"]
    /// Solo se contiene certi pattern nel comando
    CommandPattern(regex::Regex),
    /// Solo per determinati workspace/progetti
    Workspace(Vec<String>),
    /// Solo durante certi orari
    TimeWindow { start: String, end: String }, // "09:00-17:00"
    /// Custom condition (espressione)
    Custom(String),
}