use std::collections::HashMap;
use std::ops::Deref;
use std::process::Command;
use std::sync::Arc;
use crate::ast::Expression;
use crate::context::LoomContext;
use crate::error::{LoomError, LoomResult};
use crate::interceptor::context::{ExecutionContext, InterceptorContext};
use crate::interceptor::executor::config::ExecutorConfig;
use crate::interceptor::executor::ExecutorInterceptor;
use crate::interceptor::hook::registry::HookRegistry;
use crate::interceptor::{InterceptorChain, InterceptorResult};
use crate::interceptor::result::ExecutionResult;
use crate::interceptor_result;
use crate::loom_error;
use crate::types::LoomValue;

pub struct CommandExecutorInterceptor(pub Arc<[Expression]>);

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
        context: InterceptorContext<'a>,
        // TODO: Queste config mi potrebbero servie a qualcosa in questo livello
        _config: &ExecutorConfig,
        // TODO: Non dovrebbe esistere un NEXT perchè gli executor sono terminali e contengono altri interceptor
        _next: Box<InterceptorChain<'a>>,
    ) -> InterceptorResult {
        // TODO: Aggiungere hooks di "inizio", "fine", "success" e "error" definition
        // Esegue il comando
        self.launch_interceptor(context)
    }

    fn need_chain(&self) -> bool {
        false
    }

}


impl CommandExecutorInterceptor {
    
    fn launch_interceptor(
        &self,
        context: InterceptorContext<'_>,
    ) -> LoomResult<ExecutionResult> {
        let command =
            self.0.iter()
                .map(|it|
                    it.evaluate(
                        context.loom_context,
                        context.execution_context.read().map_err(|_| LoomError::execution("Error while trying to read"))?.deref(),
                        None
                    ).map(|it|
                            match it {
                                LoomValue::Literal(lit) => lit.stringify(),
                                _ => panic!("Unexpected")
                            }
                        )
                )
                .collect::<Result<Vec<_>, LoomError>>()?
            .join("");
        
        self.execute_command(&command, context.execution_context.read().map_err(|_| LoomError::execution("Error while trying to read"))?.deref())
    }
    
    /// Esegue un comando in modo cross-platform
    fn execute_command(&self, command_string: &str, context: &ExecutionContext) -> LoomResult<ExecutionResult> {
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
        //     return loom_error!("Empty command");
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
    // fn parse_command(&self, command_string: &str) -> LoomResult<Vec<String>> {
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
    //         return loom_error!("Unclosed quote in command");
    //     }
    //
    //     if !current_part.is_empty() {
    //         parts.push(current_part.trim().to_string());
    //     }
    //
    //     Ok(parts)
    // }
}