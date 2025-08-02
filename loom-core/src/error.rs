use crate::types::Position;
use std::fmt;

/// Main error type for Loom operations
#[derive(Debug, Clone)]
pub enum LoomError {
    /// Parsing errors
    ParseError {
        message: String,
        position: Position,
    },

    /// Semantic validation errors
    ValidationError {
        message: String,
        position: Option<Position>,
    },

    /// Runtime execution errors
    ExecutionError {
        message: String,
        position: Option<Position>,
        cause: Option<Box<LoomError>>,
    },

    /// Import/module resolution errors
    ImportError {
        message: String,
        import_path: String,
        position: Position,
    },

    /// Type system errors
    TypeError {
        expected: String,
        found: String,
        position: Position,
    },

    /// Undefined reference errors
    UndefinedError {
        name: String,
        kind: UndefinedKind,
        position: Position,
    },

    /// I/O and file system errors
    IoError {
        message: String,
        path: Option<String>,
    },

    /// Configuration errors
    ConfigError {
        message: String,
        path: Option<String>,
    },

    /// Plugin system errors
    PluginError {
        message: String,
        plugin_name: String,
    },

    /// System/external command errors
    SystemError {
        message: String,
        exit_code: Option<i32>,
        command: Option<String>,
    },
}

/// Types of undefined references
#[derive(Debug, Clone, PartialEq)]
pub enum UndefinedKind {
    Recipe,
    Job,
    Pipeline,
    Variable,
    Function,
    Enum,
    EnumVariant,
    Import,
}

/// Result type alias for Loom operations
pub type LoomResult<T> = Result<T, LoomError>;

/// Multiple errors collected during validation
#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<LoomError>,
}

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

    /// Create a system error
    pub fn system(message: impl Into<String>) -> Self {
        Self::SystemError {
            message: message.into(),
            exit_code: None,
            command: None,
        }
    }

    /// Create a system error with exit code
    pub fn system_with_exit(
        message: impl Into<String>,
        exit_code: i32,
        command: impl Into<String>
    ) -> Self {
        Self::SystemError {
            message: message.into(),
            exit_code: Some(exit_code),
            command: Some(command.into()),
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
            Self::ParseError { .. } => ErrorSeverity::Error,
            Self::ValidationError { .. } => ErrorSeverity::Error,
            Self::ExecutionError { .. } => ErrorSeverity::Error,
            Self::ImportError { .. } => ErrorSeverity::Error,
            Self::TypeError { .. } => ErrorSeverity::Error,
            Self::UndefinedError { .. } => ErrorSeverity::Error,
            Self::IoError { .. } => ErrorSeverity::Error,
            Self::ConfigError { .. } => ErrorSeverity::Warning,
            Self::PluginError { .. } => ErrorSeverity::Warning,
            Self::SystemError { .. } => ErrorSeverity::Error,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for LoomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError { message, position } => {
                write!(f, "Parse error at {}:{}: {}", position.line, position.column, message)
            }
            Self::ValidationError { message, position } => {
                if let Some(pos) = position {
                    write!(f, "Validation error at {}:{}: {}", pos.line, pos.column, message)
                } else {
                    write!(f, "Validation error: {}", message)
                }
            }
            Self::ExecutionError { message, position, cause } => {
                if let Some(pos) = position {
                    write!(f, "Execution error at {}:{}: {}", pos.line, pos.column, message)?;
                } else {
                    write!(f, "Execution error: {}", message)?;
                }
                if let Some(cause) = cause {
                    write!(f, " (caused by: {})", cause)?;
                }
                Ok(())
            }
            Self::ImportError { message, import_path, position } => {
                write!(f, "Import error at {}:{} importing '{}': {}",
                       position.line, position.column, import_path, message)
            }
            Self::TypeError { expected, found, position } => {
                write!(f, "Type error at {}:{}: expected {}, found {}",
                       position.line, position.column, expected, found)
            }
            Self::UndefinedError { name, kind, position } => {
                write!(f, "Undefined {} '{}' at {}:{}",
                       kind, name, position.line, position.column)
            }
            Self::IoError { message, path } => {
                if let Some(path) = path {
                    write!(f, "I/O error on '{}': {}", path, message)
                } else {
                    write!(f, "I/O error: {}", message)
                }
            }
            Self::ConfigError { message, path } => {
                if let Some(path) = path {
                    write!(f, "Configuration error in '{}': {}", path, message)
                } else {
                    write!(f, "Configuration error: {}", message)
                }
            }
            Self::PluginError { message, plugin_name } => {
                write!(f, "Plugin error in '{}': {}", plugin_name, message)
            }
            Self::SystemError { message, exit_code, command } => {
                if let (Some(code), Some(cmd)) = (exit_code, command) {
                    write!(f, "System error (exit code {}): {} - {}", code, cmd, message)
                } else {
                    write!(f, "System error: {}", message)
                }
            }
        }
    }
}

impl fmt::Display for UndefinedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recipe => write!(f, "recipe"),
            Self::Job => write!(f, "job"),
            Self::Pipeline => write!(f, "pipeline"),
            Self::Variable => write!(f, "variable"),
            Self::Function => write!(f, "function"),
            Self::Enum => write!(f, "enum"),
            Self::EnumVariant => write!(f, "enum variant"),
            Self::Import => write!(f, "import"),
        }
    }
}

impl std::error::Error for LoomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ExecutionError { cause: Some(cause), .. } => Some(cause.as_ref()),
            _ => None,
        }
    }
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn add(&mut self, error: LoomError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.errors.len() == 1 {
            write!(f, "{}", self.errors[0])
        } else {
            write!(f, "Multiple validation errors:")?;
            for (i, error) in self.errors.iter().enumerate() {
                write!(f, "\n  {}: {}", i + 1, error)?;
            }
            Ok(())
        }
    }
}

impl std::error::Error for ValidationErrors {}

// Conversion from std::io::Error
impl From<std::io::Error> for LoomError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for LoomError {
    fn from(error: serde_json::Error) -> Self {
        Self::validation(format!("JSON error: {}", error))
    }
}