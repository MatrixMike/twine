//! Shared helper functions for integration tests
//!
//! This module provides common utilities used across multiple integration test files:
//! - `eval_source()`: Helper for end-to-end evaluation testing
//! - `test_io()`: Helper for subprocess-based I/O output verification

use std::process::Command;
use std::sync::Once;
use twine_scheme::Result;
use twine_scheme::runtime::Environment;
use twine_scheme::types::Value;

static ENSURE_BINARY_BUILT: Once = Once::new();

/// Helper function for end-to-end evaluation testing.
///
/// This function provides a complete pipeline from source code string to evaluated result,
/// combining lexing, parsing, and evaluation in a single call. Used extensively across
/// integration tests to verify the complete interpreter functionality.
///
/// # Parameters
/// - `source`: Scheme source code as a string
/// - `env`: Mutable reference to the evaluation environment
///
/// # Returns
/// - `Result<Value>`: The evaluated result or an error
///
/// # Example
/// ```
/// use twine_scheme::runtime::Environment;
/// use crate::common::eval_source;
///
/// let mut env = Environment::new();
/// let result = eval_source("(+ 1 2)", &mut env).unwrap();
/// assert_eq!(result.as_number().unwrap(), 3.0);
/// ```
pub fn eval_source(source: &str, env: &mut Environment) -> Result<Value> {
    use twine_scheme::parser::Parser;
    use twine_scheme::runtime::eval::eval;

    let mut parser = Parser::new(source.to_string())?;
    let expr = parser.parse_expression()?.expr;
    eval(expr, env)
}

/// Helper function for subprocess-based I/O output verification.
///
/// This function executes Scheme source code in a subprocess using the `test_io` binary
/// and captures the stdout output for verification. This is essential for testing I/O
/// procedures like `display` and `newline` that produce side effects.
///
/// # Parameters
/// - `source`: Scheme source code to execute
/// - `expected_output`: Expected stdout output string
///
/// # Panics
/// Panics if:
/// - The subprocess execution fails
/// - The exit status is non-zero
/// - The actual output doesn't match the expected output
///
/// # Example
/// ```
/// use crate::common::test_io;
///
/// test_io(r#"(display "Hello, World!")"#, "Hello, World!");
/// test_io(r#"(newline)"#, "\n");
/// ```
#[allow(dead_code)]
pub fn test_io(source: &str, expected_output: &str) {
    // Ensure the test_io binary is built once before any tests run
    ENSURE_BINARY_BUILT.call_once(|| {
        let build_result = Command::new("cargo")
            .args(&["build", "--bin", "test_io"])
            .output()
            .expect("Failed to build test_io binary");

        if !build_result.status.success() {
            panic!(
                "Failed to build test_io binary: {}",
                String::from_utf8_lossy(&build_result.stderr)
            );
        }
    });

    // Run the pre-built binary directly to avoid cargo run race conditions
    let binary_path = if cfg!(windows) {
        "./target/debug/test_io.exe"
    } else {
        "./target/debug/test_io"
    };

    let output = Command::new(binary_path)
        .arg(source)
        .output()
        .expect("Failed to execute test_io binary");

    assert!(
        output.status.success(),
        "test_io binary exited with non-zero status: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual_output =
        String::from_utf8(output.stdout).expect("Failed to convert stdout to UTF-8");

    assert_eq!(
        actual_output, expected_output,
        "Output mismatch for source: {}\nExpected: {:?}\nActual: {:?}",
        source, expected_output, actual_output
    );
}
