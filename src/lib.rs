//! Twine Scheme Interpreter
//!
//! A minimalist Scheme interpreter written in Rust that implements a functional
//! subset of R7RS-small Scheme with fiber-based concurrency and strict immutability.

use thiserror::Error;

/// Error types for the Twine interpreter
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Syntax errors with location information
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },

    /// General parsing errors
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Result type alias for Twine operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

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
    fn test_deps_directory_structure() {
        use std::path::Path;

        // Verify deps directory structure exists
        assert!(Path::new("deps").exists(), "deps/ directory should exist");
        assert!(
            Path::new("deps/vendor").exists(),
            "deps/vendor/ directory should exist"
        );
        assert!(
            Path::new("deps/docs").exists(),
            "deps/docs/ directory should exist"
        );
        assert!(
            Path::new("deps/registry").exists(),
            "deps/registry/ directory should exist"
        );
    }

    #[test]
    fn test_gitignore_excludes_deps() {
        use std::fs;

        // Read .gitignore and verify it contains deps/
        let gitignore_content =
            fs::read_to_string(".gitignore").expect(".gitignore file should exist");

        assert!(
            gitignore_content.contains("/deps"),
            ".gitignore should exclude /deps directory"
        );
    }

    #[test]
    fn test_vendored_dependencies_exist() {
        use std::path::Path;

        // Verify that vendored dependencies exist
        assert!(
            Path::new("deps/vendor").exists(),
            "deps/vendor/ should exist"
        );

        // Check for at least one vendored dependency (thiserror)
        let vendor_dir = std::fs::read_dir("deps/vendor");
        if let Ok(entries) = vendor_dir {
            let has_dependencies = entries.count() > 0;
            assert!(
                has_dependencies,
                "deps/vendor/ should contain vendored dependencies"
            );
        }
    }

    #[test]
    fn test_generated_documentation_exists() {
        use std::path::Path;

        // Verify that generated documentation exists
        assert!(Path::new("deps/docs").exists(), "deps/docs/ should exist");

        // Check for generated documentation files
        let docs_dir = std::fs::read_dir("deps/docs");
        if let Ok(entries) = docs_dir {
            let has_docs = entries.count() > 0;
            assert!(
                has_docs,
                "deps/docs/ should contain generated documentation"
            );
        }
    }
}
