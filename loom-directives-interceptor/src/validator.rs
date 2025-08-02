// use super::traits::*;
// use loom_core::{LoomValue, Expression, DirectiveCall, LoomError, LoomResult};
// use std::collections::HashMap;
// use loom_core::ast::{DirectiveCall, Expression};
// use loom_core::definition::directive::DirectiveDefinition;
// use loom_core::definition::{ArgDefinition, ParameterType};
// use loom_core::error::{LoomError, LoomResult};
// use loom_core::interceptor::DirectiveScope;
// use loom_core::registry::directive::DirectiveRegistry;
// use loom_core::types::LoomValue;
//
// /// Validatore per parametri di direttive
// pub struct DirectiveValidator;
//
// impl DirectiveValidator {
//     /// Valida i parametri rispetto alla definizione
//     pub fn validate_parameters(
//         definition: &dyn DirectiveDefinition,
//         call: &DirectiveCall,
//     ) -> LoomResult<HashMap<String, Expression>> {
//         let mut result = HashMap::new();
//         let mut provided_positional = 0;
//         let mut provided_named = std::collections::HashSet::new();
//
//         let params = definition.parameters();
//
//         // Prima passata: raccogli tutti i parametri forniti
//         for arg in &call.args {
//             match arg {
//                 ArgDefinition::Positional(expr) => {
//                     if provided_positional < params.len() {
//                         let param = &params[provided_positional];
//                         result.insert(param.name.clone(), expr.clone());
//                         provided_positional += 1;
//                     } else {
//                         return Err(LoomError::validation_at(
//                             format!("Too many positional arguments for directive '{}'", definition.name()),
//                             call.position.clone(),
//                         ));
//                     }
//                 }
//                 ArgDefinition::Named { name, value } => {
//                     if provided_named.contains(name) {
//                         return Err(LoomError::validation_at(
//                             format!("Duplicate parameter '{}' in directive '{}'", name, definition.name()),
//                             call.position.clone(),
//                         ));
//                     }
//
//                     // Verifica che il parametro esista
//                     if !params.iter().any(|p| p.name == *name) {
//                         return Err(LoomError::validation_at(
//                             format!("Unknown parameter '{}' for directive '{}'", name, definition.name()),
//                             call.position.clone(),
//                         ));
//                     }
//
//                     result.insert(name.clone(), value.clone());
//                     provided_named.insert(name.clone());
//                 }
//             }
//         }
//
//         // Seconda passata: controlla parametri mancanti e applica default
//         for param in &params {
//             if !result.contains_key(&param.name) {
//                 if param.required {
//                     return Err(LoomError::validation_at(
//                         format!("Missing required parameter '{}' for directive '{}'",
//                                 param.name, definition.name()),
//                         call.position.clone(),
//                     ));
//                 } else if let Some(ref default) = param.default_value {
//                     result.insert(param.name.clone(), Expression::Literal(default.clone()));
//                 }
//             }
//         }
//
//         // Validazione dei tipi (basic check)
//         for (name, expr) in &result {
//             let param = params.iter().find(|p| p.name == *name).unwrap();
//             Self::validate_expression_type(expr, &param.param_type, &param.name, call)?;
//         }
//
//         // Validazione customizzata dalla definizione
//         definition.validate_parameters(&call.args)?;
//
//         Ok(result)
//     }
//
//     /// Valida che un'espressione sia compatibile con il tipo atteso
//     fn validate_expression_type(
//         expr: &Expression,
//         expected_type: &ParameterType,
//         param_name: &str,
//         call: &DirectiveCall,
//     ) -> LoomResult<()> {
//         match (expr, expected_type) {
//             // Literal validation
//             (Expression::Literal(LoomValue::String(_)), ParameterType::String) => Ok(()),
//             (Expression::Literal(LoomValue::Number(_)), ParameterType::Number) => Ok(()),
//             (Expression::Literal(LoomValue::Boolean(_)), ParameterType::Boolean) => Ok(()),
//             (Expression::Literal(LoomValue::Array(_)), ParameterType::Array(_)) => Ok(()),
//             // (Expression::Literal(LoomValue::Object(_)), ParameterType::Object) => Ok(()),
//
//             // Expression type - any expression is valid, will be evaluated at runtime
//             // (_, ParameterType::Expression) => Ok(()),
//
//             // Enum validation
//             (Expression::Literal(LoomValue::String(val)), ParameterType::Enum(values)) => {
//                 if values.contains(val) {
//                     Ok(())
//                 } else {
//                     Err(LoomError::validation_at(
//                         format!("Parameter '{}' must be one of: {}, got '{}'",
//                                 param_name, values.join(", "), val),
//                         call.position.clone(),
//                     ))
//                 }
//             }
//
//             // Variables and function calls are OK - will be validated at runtime
//             (Expression::Variable(_), _) => Ok(()),
//             (Expression::FunctionCall { .. }, _) => Ok(()),
//             (Expression::EnvVariable(_), _) => Ok(()),
//
//             // For complex expressions, we can't validate statically
//             (Expression::BinaryOp { .. }, _) => Ok(()),
//             (Expression::UnaryOp { .. }, _) => Ok(()),
//             (Expression::IndexAccess { .. }, _) => Ok(()),
//             (Expression::Interpolation { .. }, _) => Ok(()),
//             (Expression::Array(_), ParameterType::Array(_)) => Ok(()),
//             // (Expression::Object(_), ParameterType::Object) => Ok(()),
//             (Expression::EnumAccess { .. }, _) => Ok(()),
//
//             // Type mismatch
//             (expr, expected) => {
//                 Err(LoomError::validation_at(
//                     format!("Parameter '{}' expects {}, but got incompatible expression",
//                             param_name, Self::type_name(expected)),
//                     call.position.clone(),
//                 ))
//             }
//         }
//     }
//
//     fn type_name(param_type: &ParameterType) -> &str {
//         match param_type {
//             ParameterType::String => "string",
//             ParameterType::Number => "number",
//             ParameterType::Boolean => "boolean",
//             ParameterType::Array(_) => "array",
//             ParameterType::Json => "json",
//             // ParameterType::Expression => "expression",
//             ParameterType::Enum(_) => "enum value",
//         }
//     }
// }
//
// /// Valida compatibilità di scope per una direttiva
// pub fn validate_directive_scope(
//     definition: &dyn DirectiveDefinition,
//     actual_scope: DirectiveScope,
//     call: &DirectiveCall,
// ) -> LoomResult<()> {
//     if definition.scope().contains(&actual_scope) {
//         Ok(())
//     } else {
//         let valid_scopes: Vec<String> = definition
//             .scope()
//             .iter()
//             .map(|s| format!("{:?}", s))
//             .collect();
//
//         Err(LoomError::validation_at(
//             format!(
//                 "Directive '{}' cannot be used in {:?} scope. Valid scopes: {}",
//                 definition.name(),
//                 actual_scope,
//                 valid_scopes.join(", ")
//             ),
//             call.position.clone(),
//         ))
//     }
// }
//
// /// Valida che le direttive non siano in conflitto
// pub fn validate_directive_conflicts(
//     directives: &[DirectiveCall],
//     registry: DirectiveRegistry,
// ) -> LoomResult<()> {
//     let directive_names: Vec<String> = directives
//         .iter()
//         .map(|d| d.name.clone())
//         .collect();
//
//     // let conflicts = registry.find_conflicts(&directive_names);
//     //
//     // if !conflicts.is_empty() {
//     //     let conflict_messages: Vec<String> = conflicts
//     //         .iter()
//     //         .map(|(a, b)| format!("'{}' conflicts with '{}'", a, b))
//     //         .collect();
//     //
//     //     return Err(LoomError::validation(format!(
//     //         "Directive conflicts detected: {}",
//     //         conflict_messages.join(", ")
//     //     )));
//     // }
//
//     Ok(())
// }
//
// /// Valida che direttive non ripetibili non siano duplicate
// pub fn validate_directive_repetition(
//     directives: &[DirectiveCall],
//     registry: DirectiveRegistry,
// ) -> LoomResult<()> {
//     let mut seen = std::collections::HashMap::new();
//
//     for directive in directives {
//         let count = seen.entry(&directive.name).or_insert(0);
//         *count += 1;
//
//         if *count > 1 {
//             if let Some(def) = registry.get_directive(&directive.name) {
//                 if !def.definition.repeatable() {
//                     return Err(LoomError::validation_at(
//                         format!("Directive '{}' cannot be repeated", directive.name),
//                         directive.position.clone(),
//                     ));
//                 }
//             }
//         }
//     }
//
//     Ok(())
// }
//
// // #[cfg(test)]
// // mod tests {
// //     use loom_core::ast::{DirectiveArg, DirectiveCall, Expression};
// //     use loom_core::definition::{ParameterDefinition, ParameterType};
// //     use loom_core::definition::directive::DirectiveDefinition;
// //     use loom_core::error::LoomResult;
// //     use loom_core::interceptor::{DirectiveScope, ExecutionKind, ExecutionPhase};
// //     use loom_core::interceptor::context::DirectiveExecutionContext;
// //     use super::*;
// //     use loom_core::types::{LoomValue, Position};
// //
// //     struct TestDirective;
// //
// //     impl DirectiveDefinition for TestDirective {
// //         fn name(&self) -> &str { "test" }
// //         fn description(&self) -> &str { "Test directive" }
// //         fn scope(&self) -> &[DirectiveScope] { &[DirectiveScope::Definition] }
// //         fn execution_kind(&self) -> ExecutionKind { ExecutionKind::ExecuteDefinition }
// //         fn execution_phase(&self) -> ExecutionPhase { ExecutionPhase::OnStart }
// //
// //         fn parameters(&self) -> &[ParameterDefinition] {
// //             &[
// //                 ParameterDefinition {
// //                     name: "required_param".to_string(),
// //                     param_type: ParameterType::String,
// //                     required: true,
// //                     default_value: None,
// //                     description: "Required parameter".to_string(),
// //                 },
// //                 ParameterDefinition {
// //                     name: "optional_param".to_string(),
// //                     param_type: ParameterType::Number,
// //                     required: false,
// //                     default_value: Some(LoomValue::Number(42)),
// //                     description: "Optional parameter".to_string(),
// //                 },
// //             ]
// //         }
// //
// //         fn parse_args(&self, _call: &DirectiveCall) -> LoomResult<DirectiveExecutionContext> {
// //             todo!()
// //         }
// //     }
// //
// //     #[test]
// //     fn test_parameter_validation() {
// //         let directive = TestDirective;
// //         let call = DirectiveCall {
// //             name: "test".to_string(),
// //             args: vec![
// //                 DirectiveArg::Positional(Expression::Literal(LoomValue::String("hello".to_string()))),
// //             ],
// //             position: Position::default(),
// //         };
// //
// //         let result = DirectiveValidator::validate_parameters(&directive, &call);
// //         assert!(result.is_ok());
// //
// //         let params = result.unwrap();
// //         assert_eq!(params.len(), 2); // required + default
// //     }
// // }