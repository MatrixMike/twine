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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SyntaxError {
                message,
                line,
                column,
            } => {
                write!(
                    f,
                    "Syntax error at line {}, column {}: {}",
                    line, column, message
                )
            }
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
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
    fn test_error_debug_formatting() {
        let syntax_error = Error::SyntaxError {
            message: "test error".to_string(),
            line: 1,
            column: 1,
        };

        let debug_output = format!("{:?}", syntax_error);
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
}
