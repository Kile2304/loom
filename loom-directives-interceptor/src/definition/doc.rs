// use std::collections::HashMap;
// use loom_core::ast::{DirectiveCall, Expression};
// use loom_core::definition::directive::DirectiveDefinition;
// use loom_core::definition::{ParameterDefinition, ParameterType};
// use loom_core::error::LoomResult;
// use loom_core::interceptor::{DirectiveScope, ExecutionKind, ExecutionPhase};
// use loom_core::interceptor::context::DirectiveExecutionContext;
// use crate::validator::DirectiveValidator;
// 
// /// Direttiva @doc per documentazione
// pub struct DocDirective;
// 
// 
// 
// impl DirectiveDefinition for DocDirective {
// 
//     fn name(&self) -> &str {
//         "doc"
//     }
// 
//     fn description(&self) -> &str {
//         "Adds documentation to a definition"
//     }
// 
//     fn scope(&self) -> &[DirectiveScope] {
//         &[DirectiveScope::Definition, DirectiveScope::Statement]
//     }
// 
//     fn execution_kind(&self) -> ExecutionKind {
//         ExecutionKind::Help // Solo per documentazione, non esegue nulla
//     }
// 
//     fn execution_phase(&self) -> ExecutionPhase {
//         ExecutionPhase::OnStart
//     }
// 
//     fn parameters(&self) -> Vec<ParameterDefinition> {
//         vec![
//             ParameterDefinition {
//                 name: "text".to_string(),
//                 param_type: ParameterType::String,
//                 required: true,
//                 default_value: None,
//                 description: "Documentation text".to_string(),
//             }
//         ]
//     }
// 
//     fn parse_args(&self, call: &DirectiveCall) -> LoomResult<DirectiveExecutionContext> {
//         let validated = DirectiveValidator::validate_parameters(self, call)?;
// 
//         let mut parameters = HashMap::new();
//         if let Some(Expression::Literal(value)) = validated.get("text") {
//             parameters.insert("text".to_string(), value.clone());
//         }
// 
//         Ok(DirectiveExecutionContext {
//             directive_name: self.name().to_string(),
//             parameters,
//             position: call.position.clone(),
//             scope: DirectiveScope::Definition,
//         })
//     }
// }