use std::fmt;

/// Result type alias for configuration operations
pub type ParseResult<T> = Result<T, ConfigError>;

/// Errors that can occur during configuration parsing and management
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Parse error from pest
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    /// Invalid value for the expected type
    TypeError {
        key: String,
        expected: String,
        found: String,
    },

    /// Variable not found
    VariableNotFound { name: String },

    /// Circular variable dependency
    CircularDependency { chain: Vec<String> },

    /// Expression evaluation error
    ExpressionError { expression: String, reason: String },

    /// Invalid color format
    InvalidColor { value: String, reason: String },

    /// Invalid number format
    InvalidNumber { value: String, reason: String },

    /// Configuration key not found
    KeyNotFound { key: String },

    /// Special category not found
    CategoryNotFound {
        category: String,
        key: Option<String>,
    },

    /// Handler error
    HandlerError { handler: String, message: String },

    /// File I/O error
    IoError { path: String, message: String },

    /// Custom error with message
    Custom { message: String },

    /// Multiple errors collected together
    Multiple { errors: Vec<ConfigError> },
}

impl ConfigError {
    /// Create a parse error
    pub fn parse(line: usize, column: usize, message: impl Into<String>) -> Self {
        ConfigError::ParseError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a type error
    pub fn type_error(
        key: impl Into<String>,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        ConfigError::TypeError {
            key: key.into(),
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Create a variable not found error
    pub fn variable_not_found(name: impl Into<String>) -> Self {
        ConfigError::VariableNotFound { name: name.into() }
    }

    /// Create a circular dependency error
    pub fn circular_dependency(chain: Vec<String>) -> Self {
        ConfigError::CircularDependency { chain }
    }

    /// Create an expression error
    pub fn expression(expression: impl Into<String>, reason: impl Into<String>) -> Self {
        ConfigError::ExpressionError {
            expression: expression.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid color error
    pub fn invalid_color(value: impl Into<String>, reason: impl Into<String>) -> Self {
        ConfigError::InvalidColor {
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid number error
    pub fn invalid_number(value: impl Into<String>, reason: impl Into<String>) -> Self {
        ConfigError::InvalidNumber {
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create a key not found error
    pub fn key_not_found(key: impl Into<String>) -> Self {
        ConfigError::KeyNotFound { key: key.into() }
    }

    /// Create a category not found error
    pub fn category_not_found(category: impl Into<String>, key: Option<String>) -> Self {
        ConfigError::CategoryNotFound {
            category: category.into(),
            key,
        }
    }

    /// Create a handler error
    pub fn handler(handler: impl Into<String>, message: impl Into<String>) -> Self {
        ConfigError::HandlerError {
            handler: handler.into(),
            message: message.into(),
        }
    }

    /// Create an I/O error
    pub fn io(path: impl Into<String>, message: impl Into<String>) -> Self {
        ConfigError::IoError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a custom error
    pub fn custom(message: impl Into<String>) -> Self {
        ConfigError::Custom {
            message: message.into(),
        }
    }

    /// Combine multiple errors
    pub fn multiple(errors: Vec<ConfigError>) -> Self {
        ConfigError::Multiple { errors }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ParseError {
                line,
                column,
                message,
            } => {
                write!(
                    f,
                    "Parse error at line {}, column {}: {}",
                    line, column, message
                )
            }
            ConfigError::TypeError {
                key,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Type error for '{}': expected {}, found {}",
                    key, expected, found
                )
            }
            ConfigError::VariableNotFound { name } => {
                write!(f, "Variable '{}' not found", name)
            }
            ConfigError::CircularDependency { chain } => {
                write!(f, "Circular dependency detected: {}", chain.join(" -> "))
            }
            ConfigError::ExpressionError { expression, reason } => {
                write!(f, "Expression error in '{}': {}", expression, reason)
            }
            ConfigError::InvalidColor { value, reason } => {
                write!(f, "Invalid color '{}': {}", value, reason)
            }
            ConfigError::InvalidNumber { value, reason } => {
                write!(f, "Invalid number '{}': {}", value, reason)
            }
            ConfigError::KeyNotFound { key } => {
                write!(f, "Configuration key '{}' not found", key)
            }
            ConfigError::CategoryNotFound { category, key } => {
                if let Some(k) = key {
                    write!(f, "Special category '{}[{}]' not found", category, k)
                } else {
                    write!(f, "Special category '{}' not found", category)
                }
            }
            ConfigError::HandlerError { handler, message } => {
                write!(f, "Handler '{}' error: {}", handler, message)
            }
            ConfigError::IoError { path, message } => {
                write!(f, "I/O error for '{}': {}", path, message)
            }
            ConfigError::Custom { message } => {
                write!(f, "{}", message)
            }
            ConfigError::Multiple { errors } => {
                writeln!(f, "Multiple errors occurred:")?;
                for (i, err) in errors.iter().enumerate() {
                    writeln!(f, "  {}. {}", i + 1, err)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Custom {
            message: err.to_string(),
        }
    }
}

impl<R: pest::RuleType> From<pest::error::Error<R>> for ConfigError {
    fn from(err: pest::error::Error<R>) -> Self {
        let (line, column) = match err.line_col {
            pest::error::LineColLocation::Pos((line, col)) => (line, col),
            pest::error::LineColLocation::Span((line, col), _) => (line, col),
        };

        ConfigError::ParseError {
            line,
            column,
            message: err.variant.to_string(),
        }
    }
}
