use crate::ast::*;
use crate::types::*;
use std::collections::HashMap;
use std::path::PathBuf;

// TODO: In futuro pensasre se integrare il supporto di namespace

// TODO: Rendere il LoomContext più avanzato, in modo che ci sia un oggetto esterno contenente la cache
// Dei file già caricati e che per ogni esecuzione si passi i riferimenti da quell'oggetto
// Per il caching valutare: moka, ttl_cache e lru.

pub type ModuleId = uuid::Uuid;
pub type DefinitionId = uuid::Uuid;
pub type EnumId = uuid::Uuid;

/// Main context holding all parsed workflow information
#[derive(Debug)]
pub struct LoomContext {
    /// Moduli caricati/file
    pub modules: HashMap<ModuleId, Module>,
    /// Alcune definitions hanno uno o n alias, quindi, questa mappa avrà come valore, l'indice per recuperare la definizione
    definitions_ref: HashMap<String, (ModuleId, DefinitionId)>,
    enums_def_ref: HashMap<String, (ModuleId, EnumId)>,
    // No variable ref, perchè, hanno scope "locale" x file.
    // TODO: Momentaneamente pensata come cache, valutare se necessaria!
    /// Import graph for dependency resolution
    pub dependencies: HashMap<PathBuf, Vec<ImportKind>>,
}

#[derive(Debug, PartialEq)]
pub struct Module {
    pub definitions: HashMap<DefinitionId, Definition>,
    pub enums: HashMap<EnumId, EnumDef>,
    pub variables: HashMap<String, LoomValue>,
    pub dependencies: HashMap<PathBuf, Vec<ImportKind>>,
}

#[derive(Debug, PartialEq)]
pub enum ImportKind {
    ImportAll,
    ImportDefinition(String),
}

impl LoomContext {
    pub fn new() -> Self {
        Self {
            definitions_ref: HashMap::new(),
            enums_def_ref: HashMap::new(),
            dependencies: HashMap::new(),
            modules: HashMap::new(),
        }
    }

    // /// Add a parsed workflow file to the context
    // pub fn add_file(&mut self, path: PathBuf, file: WorkflowFile) -> Result<(), String> {
    //     // Store the file
    //     self.files.insert(path.clone(), file);
    // 
    //     // Update import graph
    //     self.update_import_graph(&path)?;
    // 
    //     // Resolve all imports and merge definitions
    //     self.resolve_imports()?;
    // 
    //     Ok(())
    // }
    pub fn call_function(&self, name: &str, args: Vec<LoomValue>) -> Result<LoomValue, String> {
        Ok(LoomValue::Empty)
    }
    

    /// Find a definition by name
    pub fn find_definition(&self, name: &str) -> Option<&Definition> {
        self.definitions_ref.get(name)
            .and_then(|index|
                self.modules.get(&index.0)
                    .and_then(|it| it.definitions.get(&index.1))
            )
    }

    /// Find an enum by name
    pub fn find_enum(&self, name: &str) -> Option<&EnumDef> {
        self.enums_def_ref.get(name)
            .and_then(|index| self.modules.get(&index.0)?.enums.get(&index.1))
    }

    /// Get variable value
    // pub fn get_variable(&self, name: &str) -> Option<&LoomValue> {
    //     self.variables.get(name)
    // }
    pub fn get_variables(&self, name: &str) -> Option<&HashMap<String, LoomValue>> {
        self.definitions_ref.get(name)
            .and_then(|index|
                self.modules.get(&index.0)
                    .and_then(|it| Some(&it.variables))
            )
    }

    // /// Set variable value
    // pub fn set_variable(&mut self, name: String, value: LoomValue) {
    //     self.variables.insert(name, value);
    // }

    // /// Get all definitions of a specific kind
    // pub fn get_definitions_by_kind(&self, kind: DefinitionKind) -> Vec<&Definition> {
    //     self.definitions
    //         .iter()
    //         .filter(|def| def.kind == kind)
    //     .collect()
    // }

    // /// Validate that all referenced definitions exist
    // pub fn validate_references(&self) -> Result<(), Vec<String>> {
    //     let mut errors = Vec::new();
    // 
    //     for definition in &self.definitions {
    //         self.validate_definition_references(&definition.signature.name, definition, &mut errors);
    //     }
    // 
    //     if errors.is_empty() {
    //         Ok(())
    //     } else {
    //         Err(errors)
    //     }
    // }

    // fn update_import_graph(&mut self, file_path: &PathBuf) -> Result<(), String> {
    //     let file = self.files.get(file_path).ok_or("File not found")?;
    // 
    //     let mut dependencies = Vec::new();
    //     for import in &file.imports {
    //         let import_path = self.resolve_import_path(file_path, &import)?;
    //         dependencies.push(import_path);
    //     }
    // 
    //     // self.dependencies.insert(file_path.clone(), dependencies);
    //     // TODO: Sistemare
    //     Ok(())
    // }

    fn resolve_import_path(&self, current_file: &PathBuf, import_path: &str) -> Result<PathBuf, String> {
        // Simple resolution - in practice, this would be more sophisticated
        let current_dir = current_file.parent().unwrap_or(current_file);
        let resolved = current_dir.join(format!("{}.wfc", import_path));
        Ok(resolved)
    }

    // fn resolve_imports(&mut self) -> Result<(), String> {
    //     // Topological sort of files based on import dependencies
    //     self.compute_load_order()?;
    // 
    //     // Clear existing resolved data
    //     self.definitions.clear();
    //     self.enums.clear();
    //     self.variables.clear();
    // 
    //     // Process files in dependency order
    //     for file_path in &self.import_graph.load_order.clone() {
    //         self.process_file_imports(file_path)?;
    //     }
    // 
    //     Ok(())
    // }

    // fn compute_load_order(&mut self) -> Result<(), String> {
    //     // Simple topological sort implementation
    //     // In practice, you'd want a more robust cycle detection
    //     let mut visited = std::collections::HashSet::new();
    //     let mut order = Vec::new();
    // 
    //     for file_path in self.files.keys() {
    //         if !visited.contains(file_path) {
    //             self.dfs_visit(file_path, &mut visited, &mut order)?;
    //         }
    //     }
    // 
    //     order.reverse();
    //     self.import_graph.load_order = order;
    //     Ok(())
    // }

    // fn dfs_visit(
    //     &self,
    //     file_path: &PathBuf,
    //     visited: &mut std::collections::HashSet<PathBuf>,
    //     order: &mut Vec<PathBuf>,
    // ) -> Result<(), String> {
    //     visited.insert(file_path.clone());
    // 
    //     if let Some(deps) = self.import_graph.dependencies.get(file_path) {
    //         for dep in deps {
    //             if !visited.contains(dep) {
    //                 self.dfs_visit(dep, visited, order)?;
    //             }
    //         }
    //     }
    // 
    //     order.push(file_path.clone());
    //     Ok(())
    // }

    // fn process_file_imports(&mut self, file_path: &PathBuf) -> Result<(), String> {
    //     let file = self.files.get(file_path).unwrap().clone();
    // 
    //     // Add enums
    //     for enum_def in file.enums {
    //         self.enums.insert(enum_def.name.clone(), enum_def);
    //     }
    // 
    //     // Process variable assignments
    //     for var_assignment in file.variables {
    //         // Note: In practice, you'd evaluate the expression here
    //         // For now, we'll store as-is and evaluate during execution
    //         self.variables.insert(var_assignment.name.clone(), LoomValue::Empty);
    //     }
    // 
    //     // Add definitions
    //     for definition in file.definitions {
    //         let name = definition.signature.name.clone();
    //         let last_index = self.definitions.len();
    //         self.definitions.push(definition);
    //         self.definitions_ref.insert(name, last_index);
    //     }
    // 
    //     Ok(())
    // }

    fn validate_definition_references(&self, _name: &str, definition: &Definition, errors: &mut Vec<String>) {
        // Validate that all referenced jobs/recipes exist
        self.validate_block_references(&definition.body, errors);
    }

    fn validate_block_references(&self, block: &Block, errors: &mut Vec<String>) {
        for statement in &block.statements {
            match statement {
                Statement::Call { name, .. } => {
                    if !self.definitions_ref.contains_key(name) {
                        errors.push(format!("Undefined reference: {}", name));
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for LoomContext {
    fn default() -> Self {
        Self::new()
    }
}