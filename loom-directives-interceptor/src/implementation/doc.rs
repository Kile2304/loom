// use loom_core::definition::directive::DirectiveExecutor;
// use loom_core::error::LoomResult;
// use loom_core::interceptor::context::{DirectiveExecutionContext, ExecutionEnvironment};
// use loom_core::interceptor::result::DirectiveResult;
// use loom_core::types::LoomValue;
// 
// /// Executor per @doc - non fa nulla a runtime, solo documentazione
// pub struct DocExecutor;
// 
// impl DirectiveExecutor for DocExecutor {
//     fn execute(
//         &self,
//         context: &DirectiveExecutionContext,
//         _execution_context: &mut ExecutionEnvironment,
//     ) -> LoomResult<DirectiveResult> {
//         // La documentazione è solo per help/metadata, non fa nulla a runtime
//         let doc_text = context.parameters
//             .get("text")
//             .and_then(|v| if let LoomValue::String(s) = v { Some(s.clone()) } else { None })
//         .unwrap_or_default();
// 
//         Ok(DirectiveResult::Success {
//             output: Some(format!("Documentation: {}", doc_text)),
//             modified_env: None,
//         })
//     }
// }