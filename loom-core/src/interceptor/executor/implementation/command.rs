use std::collections::HashMap;
use std::process::Command;
use crate::ast::Expression;
use crate::context::LoomContext;
use crate::interceptor::context::ExecutionContext;
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::ExecutorInterceptor;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::{InterceptorChain, InterceptorResult};
use crate::interceptor::result::ExecutionResult;
use crate::interceptor_result;
use crate::types::LoomValue;

pub struct CommandExecutorInterceptor(pub Vec<Expression>, pub Option<String>);

#[async_trait::async_trait]
impl ExecutorInterceptor for CommandExecutorInterceptor {
    fn name(&self) -> &str {
        "command"
    }
    fn description(&self) -> &str {
        "Esegue un command"
    }
    fn default_config(&self) -> ExecutorConfig {
        ExecutorConfig::default()
    }
    async fn intercept<'a>(
        &'a self,
        loom_context: &'a LoomContext,
        context: &'a mut ExecutionContext,
        _hook_registry: &'a HookRegistry,
        // TODO: Queste config mi potrebbero servie a qualcosa in questo livello
        _config: &'a ExecutorConfig,
        // TODO: Non dovrebbe esistere un NEXT perchè gli executor sono terminali e contengono altri interceptor
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        // TODO: Aggiungere hooks di "inizio", "fine", "success" e "error" definition
        // Esegue il comando
        self.launch_interceptor(loom_context, context)
    }

}


impl CommandExecutorInterceptor {
    
    fn launch_interceptor(
        &self,
        loom_context: &LoomContext,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionResult, String> {
        let command =
            self.0.iter()
                .map(|it|
                    it.evaluate(loom_context, context)
                        .map(|it|
                            match it {
                                LoomValue::Literal(lit) => lit.stringify(),
                                _ => panic!("Unexpected")
                            }
                        )
                )
                .collect::<Result<Vec<_>, String>>()?
            .join("");
        
        self.execute_command(&command, context)
    }
    
    /// Esegue un comando in modo cross-platform
    fn execute_command(&self, command_string: &str, context: &ExecutionContext) -> Result<ExecutionResult, String> {
        if context.dry_run {
            return Ok(ExecutionResult {
                output: Some(format!("DRY RUN: Would execute: {}", command_string)),
                exit_code: Some(0),
                metadata: HashMap::new(),
            });
        }

        // Parsing del comando per separare comando base e argomenti
        // let parts = self.parse_command(command_string)?;
        // if parts.is_empty() {
        //     return Err("Empty command".to_string());
        // }

        // let (cmd, args) = parts.split_first().unwrap();

        // let start_time = std::time::Instant::now();

        // Costruisce il comando
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd_builder = Command::new("cmd");
            cmd_builder.args(&["/C", command_string]);
            cmd_builder
        } else {
            let mut cmd_builder = Command::new("sh");
            cmd_builder.args(&["-c", command_string]);
            cmd_builder
        };

        // Imposta la working directory se specificata
        if let Some(ref working_dir) = context.working_dir {
            command.current_dir(working_dir);
        }

        // Imposta le variabili d'ambiente
        for (key, value) in &context.env_vars {
            command.env(key, value);
        }

        // Esegue il comando
        match command.output() {
            Ok(output) => {
                // let execution_time = start_time.elapsed();
                // let success = output.status.success();
                let exit_code = output.status.code();

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                // let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let mut metadata = HashMap::new();
                metadata.insert("command".to_string(), command_string.to_string());
                if let Some(code) = exit_code {
                    metadata.insert("exit_code".to_string(), code.to_string());
                }

                Ok(ExecutionResult {
                    output: if stdout.is_empty() { None } else { Some(stdout) },
                    exit_code,
                    metadata,
                })
            }
            Err(e) => {
                // let execution_time = start_time.elapsed();
                let mut metadata = HashMap::new();
                metadata.insert("command".to_string(), command_string.to_string());
                metadata.insert("system_error".to_string(), e.to_string());

                Ok(ExecutionResult {
                    output: None,
                    exit_code: None,
                    metadata,
                })
            }
        }
    }

    // /// Parsing semplice del comando per separare comando e argomenti
    // /// Gestisce le virgolette per argomenti con spazi
    // fn parse_command(&self, command_string: &str) -> Result<Vec<String>, String> {
    //     let mut parts = Vec::new();
    //     let mut current_part = String::new();
    //     let mut in_quotes = false;
    //     let mut chars = command_string.chars().peekable();
    //
    //     while let Some(ch) = chars.next() {
    //         match ch {
    //             '"' if !in_quotes => {
    //                 in_quotes = true;
    //             }
    //             '"' if in_quotes => {
    //                 in_quotes = false;
    //             }
    //             ' ' if !in_quotes => {
    //                 if !current_part.is_empty() {
    //                     parts.push(current_part.trim().to_string());
    //                     current_part.clear();
    //                 }
    //             }
    //             _ => {
    //                 current_part.push(ch);
    //             }
    //         }
    //     }
    //
    //     if in_quotes {
    //         return Err("Unclosed quote in command".to_string());
    //     }
    //
    //     if !current_part.is_empty() {
    //         parts.push(current_part.trim().to_string());
    //     }
    //
    //     Ok(parts)
    // }
}