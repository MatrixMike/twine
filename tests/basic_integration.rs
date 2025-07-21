//! Basic integration tests for Twine Scheme Interpreter
//!
//! This file contains minimal integration tests to verify that the test framework
//! is properly set up and can discover and run tests successfully.

use twine_scheme::{Error, Result};

#[test]
fn test_framework_setup() {
    // Simple test to verify the test framework is working
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_error_integration() {
    // Test that we can use the Error types from the main library
    let error = Error::ParseError("test error".to_string());
    assert_eq!(error.to_string(), "Parse error: test error");
}

#[test]
fn test_result_type_integration() {
    // Test that we can use the Result type from the main library
    fn dummy_function() -> Result<i32> {
        Ok(42)
    }

    let result = dummy_function();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_syntax_error_integration() {
    // Test creating and displaying syntax errors
    let error = Error::SyntaxError {
        message: "unexpected character".to_string(),
        line: 3,
        column: 7,
    };

    let error_string = error.to_string();
    assert!(error_string.contains("Syntax error"));
    assert!(error_string.contains("line 3"));
    assert!(error_string.contains("column 7"));
    assert!(error_string.contains("unexpected character"));
}
