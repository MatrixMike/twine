//! Error handling for the Twine Scheme interpreter
//!
//! This module defines the error types used throughout the interpreter,
//! including syntax errors with position information and general parse errors.

use std::fmt;

/// Error types for the Twine interpreter
#[derive(Debug, Clone)]
pub enum Error {
    /// Syntax errors with location information
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },

    /// General parsing errors
    ParseError(String),

    /// Runtime evaluation errors
    RuntimeError(String),

    /// Arity errors for incorrect number of arguments
    ArityError {
        procedure: String,
        expected: usize,
        actual: usize,
    },

    /// Type errors for incorrect argument types
    TypeError {
        procedure: String,
        expected: String,
        actual: String,
        position: Option<usize>,
    },

    /// Environment-related errors
    EnvironmentError {
        kind: EnvironmentErrorKind,
        identifier: String,
        context: Option<String>,
    },
}

/// Specific kinds of environment errors
#[derive(Debug, Clone)]
pub enum EnvironmentErrorKind {
    /// Unbound identifier error
    UnboundIdentifier,
    /// Invalid identifier
    InvalidIdentifier,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SyntaxError {
                message,
                line,
                column,
            } => {
                write!(f, "Syntax error at line {line}, column {column}: {message}")
            }
            Error::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Error::RuntimeError(msg) => write!(f, "Runtime error: {msg}"),
            Error::ArityError {
                procedure,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "{}: expected {} argument{}, got {}",
                    procedure,
                    expected,
                    if *expected == 1 { "" } else { "s" },
                    actual
                )
            }
            Error::TypeError {
                procedure,
                expected,
                actual,
                position,
            } => {
                if let Some(pos) = position {
                    write!(
                        f,
                        "{procedure}: expected {expected} for argument {pos}, got {actual}"
                    )
                } else {
                    write!(f, "{procedure}: expected {expected}, got {actual}")
                }
            }
            Error::EnvironmentError {
                kind,
                identifier,
                context,
            } => {
                let base_msg = match kind {
                    EnvironmentErrorKind::UnboundIdentifier => {
                        format!("Unbound identifier: '{identifier}'")
                    }
                    EnvironmentErrorKind::InvalidIdentifier => {
                        format!("Invalid identifier: '{identifier}'")
                    }
                };

                if let Some(ctx) = context {
                    write!(f, "{base_msg}. {ctx}")
                } else {
                    write!(f, "{base_msg}")
                }
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type alias for Twine operations
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a syntax error with position information
    pub fn syntax_error(message: &str, line: usize, column: usize) -> Self {
        Self::SyntaxError {
            message: message.to_string(),
            line,
            column,
        }
    }

    /// Create a general parse error
    pub fn parse_error(message: &str) -> Self {
        Self::ParseError(message.to_string())
    }

    /// Create a runtime error
    pub fn runtime_error(message: &str) -> Self {
        Self::RuntimeError(message.to_string())
    }

    /// Create an unbound identifier error with optional context
    pub fn unbound_identifier(identifier: &str, context: Option<&str>) -> Self {
        Self::EnvironmentError {
            kind: EnvironmentErrorKind::UnboundIdentifier,
            identifier: identifier.to_string(),
            context: context.map(|c| c.to_string()),
        }
    }

    /// Create an invalid identifier error with optional context
    pub fn invalid_identifier(identifier: &str, context: Option<&str>) -> Self {
        Self::EnvironmentError {
            kind: EnvironmentErrorKind::InvalidIdentifier,
            identifier: identifier.to_string(),
            context: context.map(|c| c.to_string()),
        }
    }

    /// Create an arity error for incorrect number of arguments
    pub fn arity_error(procedure: &str, expected: usize, actual: usize) -> Self {
        Self::ArityError {
            procedure: procedure.to_string(),
            expected,
            actual,
        }
    }

    /// Create a structured type error with procedure and type information
    pub fn type_error(
        procedure: &str,
        expected: &str,
        actual: &str,
        position: Option<usize>,
    ) -> Self {
        Self::TypeError {
            procedure: procedure.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error_creation() {
        let error = Error::SyntaxError {
            message: "unexpected token".to_string(),
            line: 5,
            column: 10,
        };

        assert_eq!(
            error.to_string(),
            "Syntax error at line 5, column 10: unexpected token"
        );
    }

    #[test]
    fn test_parse_error_creation() {
        let error = Error::ParseError("invalid expression".to_string());

        assert_eq!(error.to_string(), "Parse error: invalid expression");
    }

    #[test]
    fn test_result_type_usage() {
        fn parse_number(s: &str) -> Result<i32> {
            s.parse()
                .map_err(|_| Error::ParseError("not a valid number".to_string()))
        }

        assert!(parse_number("42").is_ok());
        assert_eq!(parse_number("42").unwrap(), 42);

        assert!(parse_number("abc").is_err());
        assert!(matches!(
            parse_number("abc").unwrap_err(),
            Error::ParseError(_)
        ));
    }

    #[test]
    fn test_runtime_error() {
        let error = Error::runtime_error("Division by zero");

        assert!(matches!(error, Error::RuntimeError(_)));
        assert_eq!(error.to_string(), "Runtime error: Division by zero");
    }

    #[test]
    fn test_error_debug_formatting() {
        let syntax_error = Error::SyntaxError {
            message: "test error".to_string(),
            line: 1,
            column: 1,
        };

        let debug_output = format!("{syntax_error:?}");
        assert!(debug_output.contains("SyntaxError"));
        assert!(debug_output.contains("test error"));
        assert!(debug_output.contains("line: 1"));
        assert!(debug_output.contains("column: 1"));
    }

    #[test]
    fn test_error_cloning() {
        let original = Error::ParseError("original error".to_string());
        let cloned = original.clone();

        assert_eq!(original.to_string(), cloned.to_string());
    }

    #[test]
    fn test_syntax_error_helper() {
        let error = Error::syntax_error("unexpected token", 5, 10);

        assert!(matches!(error, Error::SyntaxError { .. }));
        assert_eq!(
            error.to_string(),
            "Syntax error at line 5, column 10: unexpected token"
        );
    }

    #[test]
    fn test_parse_error_helper() {
        let error = Error::parse_error("invalid expression");

        assert!(matches!(error, Error::ParseError(_)));
        assert_eq!(error.to_string(), "Parse error: invalid expression");
    }

    #[test]
    fn test_unbound_identifier_error() {
        let error = Error::unbound_identifier("x", None);

        assert!(matches!(
            error,
            Error::EnvironmentError {
                kind: EnvironmentErrorKind::UnboundIdentifier,
                ..
            }
        ));
        assert_eq!(error.to_string(), "Unbound identifier: 'x'");
    }

    #[test]
    fn test_unbound_identifier_error_with_context() {
        let error = Error::unbound_identifier("x", Some("Did you mean one of: 'y', 'z'?"));

        assert_eq!(
            error.to_string(),
            "Unbound identifier: 'x'. Did you mean one of: 'y', 'z'?"
        );
    }

    #[test]
    fn test_invalid_identifier_error() {
        let error = Error::invalid_identifier("123abc", None);

        assert!(matches!(
            error,
            Error::EnvironmentError {
                kind: EnvironmentErrorKind::InvalidIdentifier,
                ..
            }
        ));
        assert_eq!(error.to_string(), "Invalid identifier: '123abc'");
    }

    #[test]
    fn test_invalid_identifier_error_with_context() {
        let error =
            Error::invalid_identifier("123abc", Some("Identifiers cannot start with digits"));

        assert!(matches!(
            error,
            Error::EnvironmentError {
                kind: EnvironmentErrorKind::InvalidIdentifier,
                ..
            }
        ));
        assert_eq!(
            error.to_string(),
            "Invalid identifier: '123abc'. Identifiers cannot start with digits"
        );
    }

    #[test]
    fn test_arity_error() {
        let error = Error::arity_error("car", 1, 0);

        assert!(matches!(error, Error::ArityError { .. }));
        assert_eq!(error.to_string(), "car: expected 1 argument, got 0");

        // Test plural
        let error = Error::arity_error("list", 3, 2);
        assert_eq!(error.to_string(), "list: expected 3 arguments, got 2");
    }

    #[test]
    fn test_type_error() {
        let error = Error::type_error("car", "list", "number", None);

        assert!(matches!(error, Error::TypeError { .. }));
        assert_eq!(error.to_string(), "car: expected list, got number");

        // Test with position
        let error = Error::type_error("cons", "list", "string", Some(2));
        assert_eq!(
            error.to_string(),
            "cons: expected list for argument 2, got string"
        );
    }
}
