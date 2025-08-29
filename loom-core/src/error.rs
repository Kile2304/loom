use derive_more::with_trait::Display;
use crate::types::Position;
use crate::interceptor::scope::ExecutionScope;
use thiserror::Error;

/// Main error type for Loom operations
#[derive(Debug, Clone, Error)]
pub enum LoomError {
    /// Parsing errors
    #[error("Parse error at {position}: {message}")]
    ParseError {
        message: String,
        position: Position,
    },

    /// Semantic validation errors
    #[error("Validation error{}: {message}", position.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default())]
    ValidationError {
        message: String,
        position: Option<Position>,
    },

    /// Runtime execution errors
    #[error("Execution error{}: {message}{}", position.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default(), cause.as_ref().map(|c| format!(" (caused by: {})", c)).unwrap_or_default())]
    ExecutionError {
        message: String,
        position: Option<Position>,
        #[source]
        cause: Option<Box<LoomError>>,
    },

    /// Import/module resolution errors
    #[error("Import error at {position} importing '{import_path}': {message}")]
    ImportError {
        message: String,
        import_path: String,
        position: Position,
    },

    /// Type system errors
    #[error("Type error at {position}: expected {expected}, found {found}")]
    TypeError {
        expected: String,
        found: String,
        position: Position,
    },

    /// Undefined reference errors
    #[error("Undefined {kind} '{name}' at {position}")]
    UndefinedError {
        name: String,
        kind: UndefinedKind,
        position: Position,
    },

    /// I/O and file system errors
    #[error("I/O error{}: {message}", path.as_ref().map(|p| format!(" on '{}'", p)).unwrap_or_default())]
    IoError {
        message: String,
        path: Option<String>,
    },

    /// Configuration errors
    #[error("Configuration error{}: {message}", path.as_ref().map(|p| format!(" in '{}'", p)).unwrap_or_default())]
    ConfigError {
        message: String,
        path: Option<String>,
    },

    /// Plugin system errors
    #[error("Plugin error in '{plugin_name}': {message}")]
    PluginError {
        message: String,
        plugin_name: String,
    },

    #[error(transparent)]
    InterceptorError {
        #[from]
        error: InterceptorError,
        interceptor_stack: Vec<String>,
    },

    /// Errori di conversione tra tipi
    #[error("Conversion error{}: cannot convert '{value}' from {from_type} to {to_type}", position.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default())]
    ConversionError {
        from_type: String,
        to_type: String,
        value: String,
        position: Option<Position>,
    },

    /// Errori di lock/concorrenza
    #[error("Concurrency error on resource '{resource}' during '{operation}': {message}")]
    ConcurrencyError {
        resource: String,
        operation: String,
        message: String,
    },

    /// Errori di valutazione di espressioni
    #[error("Expression error in {expression_type} at {position}: {message}")]
    ExpressionError {
        expression_type: String,
        message: String,
        position: Position,
    },

    /// Errori di funzioni non implementate
    #[error("Feature '{feature}' not implemented in context '{context}'{}", position.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default())]
    NotImplementedError {
        feature: String,
        context: String,
        position: Option<Position>,
    },

    /// Errori di definizione non trovata
    #[error("Definition '{name}' not found at {position}. Available definitions: [{}]", available_definitions.join(", "))]
    DefinitionNotFoundError {
        name: String,
        available_definitions: Vec<String>,
        position: Position,
    },

    /// Errori di parameter mismatch
    #[error("Parameter error in '{definition_name}'{}: {}",
        position.as_ref().map(|p| format!(" at {}", p)).unwrap_or_default(),
        parameter_name.as_ref().map(|p| format!("invalid parameter '{}'", p))
            .unwrap_or_else(|| format!("expected {} parameters, got {}", expected_count, provided_count))
    )]
    ParameterError {
        definition_name: String,
        expected_count: usize,
        provided_count: usize,
        parameter_name: Option<String>,
        position: Option<Position>,
    },

    /// Errori di chain interceptor
    #[error("Interceptor chain error at position {chain_position} in '{interceptor_name}': {cause}")]
    InterceptorChainError {
        interceptor_name: String,
        chain_position: usize,
        #[source]
        cause: Box<LoomError>,
    },
}

#[derive(Debug, Clone, Error)]
pub enum InterceptorError {
    // Directive interceptor errors
    #[error("Interceptor error: While executing the Directive Interceptor '{name}' the following error occured '{message}'")]
    Directive {
        name: String,
        message: String,
    },

    // Global interceptor errors
    #[error("Interceptor error: While executing the Global Interceptor '{name}' the following error occured '{message}'")]
    Global {
        name: String,
        message: String,
    },

    // Execution interceptor errors with scope
    #[error("Interceptor error: While executing the Execution Interceptor (scope) '{scope:?}' the following error occured '{message}'")]
    Execution {
        scope: ExecutionScope,
        message: String,
    },

    // Command execution errors
    #[error("Command execution error{}: '{}' - {}",
        exit_code.map(|c| format!(" (exit code {})", c)).unwrap_or_default(),
        command,
        message
    )]
    CommandExecution {
        command: String,
        message: String,
        exit_code: Option<i32>,
    },

    // Definition resolution errors
    #[error("Definition resolution error: '{name}' - {message}")]
    DefinitionResolution {
        name: String,
        message: String,
    },

    // Parameter validation errors
    #[error("Parameter validation error: '{name}' - {message}")]
    ParameterValidation {
        name: String,
        message: String,
    },

    // Chain execution errors
    #[error("Chain execution error: {message}")]
    ChainExecution {
        message: String,
    },

    // Context access errors
    #[error("Context access error: {message}")]
    ContextAccess {
        message: String,
    },

    // Pipeline execution errors
    #[error("Pipeline execution error in pipeline '{name}'{}: {message}",
        stage.as_ref().map(|s| format!(" at stage '{}'", s)).unwrap_or_default()
    )]
    PipelineExecution {
        name: String,
        stage: Option<String>,
        message: String,
    },

    // Job execution errors
    #[error("Job execution error in job '{name}': {message}")]
    JobExecution {
        name: String,
        message: String,
    }
}

/// Types of undefined references
#[derive(Debug, Clone, PartialEq, Display)]
pub enum UndefinedKind {
    #[display("recipe")]
    Recipe,
    #[display("job")]
    Job,
    #[display("pipeline")]
    Pipeline,
    #[display("variable")]
    Variable,
    #[display("function")]
    Function,
    #[display("enum")]
    Enum,
    #[display("enum variant")]
    EnumVariant,
    #[display("import")]
    Import,
}

/// Result type alias for Loom operations
pub type LoomResult<T> = Result<T, LoomError>;

impl LoomError {
    /// Create a parse error
    pub fn parse(message: impl Into<String>, position: Position) -> Self {
        Self::ParseError {
            message: message.into(),
            position,
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            position: None,
        }
    }

    /// Create a validation error with position
    pub fn validation_at(message: impl Into<String>, position: Position) -> Self {
        Self::ValidationError {
            message: message.into(),
            position: Some(position),
        }
    }

    /// Create an execution error
    pub fn execution(message: impl Into<String>) -> Self {
        Self::ExecutionError {
            message: message.into(),
            position: None,
            cause: None,
        }
    }

    /// Create an execution error with cause
    pub fn execution_with_cause(message: impl Into<String>, cause: LoomError) -> Self {
        Self::ExecutionError {
            message: message.into(),
            position: None,
            cause: Some(Box::new(cause)),
        }
    }

    /// Create a type error
    pub fn type_error(expected: impl Into<String>, found: impl Into<String>, position: Position) -> Self {
        Self::TypeError {
            expected: expected.into(),
            found: found.into(),
            position,
        }
    }

    /// Create an undefined reference error
    pub fn undefined(name: impl Into<String>, kind: UndefinedKind, position: Position) -> Self {
        Self::UndefinedError {
            name: name.into(),
            kind,
            position,
        }
    }

    /// Create an I/O error
    pub fn io(message: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
            path: None,
        }
    }

    /// Create an I/O error with path
    pub fn io_with_path(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
            path: Some(path.into()),
        }
    }

    /// Create an interceptor error with directive scope
    pub fn directive_interceptor(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::Directive {
                name: name.into(),
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error with global scope
    pub fn global_interceptor(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::Global {
                name: name.into(),
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error with execution scope
    pub fn execution_interceptor(scope: ExecutionScope, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::Execution {
                scope,
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for command execution
    pub fn command_execution(command: impl Into<String>, message: impl Into<String>, exit_code: Option<i32>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::CommandExecution {
                command: command.into(),
                message: message.into(),
                exit_code,
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for definition resolution
    pub fn definition_resolution(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::DefinitionResolution {
                name: name.into(),
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for parameter validation
    pub fn parameter_validation(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::ParameterValidation {
                name: name.into(),
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for chain execution
    pub fn chain_execution(message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::ChainExecution {
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for context access
    pub fn context_access(message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::ContextAccess {
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for pipeline execution
    pub fn pipeline_execution(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::PipelineExecution {
                name: name.into(),
                stage: None,
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for pipeline execution with stage
    pub fn pipeline_stage_execution(name: impl Into<String>, stage: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::PipelineExecution {
                name: name.into(),
                stage: Some(stage.into()),
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create an interceptor error for job execution
    pub fn job_execution(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InterceptorError {
            error: InterceptorError::JobExecution {
                name: name.into(),
                message: message.into(),
            },
            interceptor_stack: Vec::new(),
        }
    }

    /// Create a conversion error
    pub fn conversion(
        from_type: impl Into<String>,
        to_type: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::ConversionError {
            from_type: from_type.into(),
            to_type: to_type.into(),
            value: value.into(),
            position: None,
        }
    }

    /// Create a conversion error with position
    pub fn conversion_at(
        from_type: impl Into<String>,
        to_type: impl Into<String>,
        value: impl Into<String>,
        position: Position,
    ) -> Self {
        Self::ConversionError {
            from_type: from_type.into(),
            to_type: to_type.into(),
            value: value.into(),
            position: Some(position),
        }
    }

    /// Create a concurrency error
    pub fn concurrency(
        resource: impl Into<String>,
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ConcurrencyError {
            resource: resource.into(),
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create an expression error
    pub fn expression(
        expression_type: impl Into<String>,
        message: impl Into<String>,
        position: Position,
    ) -> Self {
        Self::ExpressionError {
            expression_type: expression_type.into(),
            message: message.into(),
            position,
        }
    }

    /// Create a not implemented error
    pub fn not_implemented(
        feature: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::NotImplementedError {
            feature: feature.into(),
            context: context.into(),
            position: None,
        }
    }

    /// Create a definition not found error
    pub fn definition_not_found(
        name: impl Into<String>,
        available: Vec<String>,
        position: Position,
    ) -> Self {
        Self::DefinitionNotFoundError {
            name: name.into(),
            available_definitions: available,
            position,
        }
    }

    /// Create a parameter error
    pub fn parameter_mismatch(
        definition_name: impl Into<String>,
        expected: usize,
        provided: usize,
    ) -> Self {
        Self::ParameterError {
            definition_name: definition_name.into(),
            expected_count: expected,
            provided_count: provided,
            parameter_name: None,
            position: None,
        }
    }

    /// Create an interceptor chain error
    pub fn interceptor_chain(
        interceptor_name: impl Into<String>,
        chain_position: usize,
        cause: LoomError,
    ) -> Self {
        Self::InterceptorChainError {
            interceptor_name: interceptor_name.into(),
            chain_position,
            cause: Box::new(cause),
        }
    }

    /// Get the error position if available
    pub fn position(&self) -> Option<&Position> {
        match self {
            Self::ParseError { position, .. } => Some(position),
            Self::ValidationError { position, .. } => position.as_ref(),
            Self::ExecutionError { position, .. } => position.as_ref(),
            Self::ImportError { position, .. } => Some(position),
            Self::TypeError { position, .. } => Some(position),
            Self::UndefinedError { position, .. } => Some(position),
            _ => None,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            _ => ErrorSeverity::Error
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        match &mut self {
            Self::ExecutionError { message, .. } => {
                *message = format!("{}: {}", context.into(), message);
            }
            _ => {}
        }
        self
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

// Automatic conversions
impl From<std::io::Error> for LoomError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

impl From<serde_json::Error> for LoomError {
    fn from(error: serde_json::Error) -> Self {
        Self::validation(format!("JSON error: {}", error))
    }
}

impl From<String> for LoomError {
    fn from(error: String) -> Self {
        Self::execution(error)
    }
}

impl<'a> From<&'a str> for LoomError {
    fn from(error: &'a str) -> Self {
        Self::execution(error)
    }
}

// Macro for creating execution errors
#[macro_export]
macro_rules! loom_error {
    ($msg:expr) => {
        Err(crate::error::LoomError::execution($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        Err(crate::error::LoomError::execution(format!($fmt, $($arg)*)))
    };
}