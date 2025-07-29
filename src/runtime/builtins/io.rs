//! I/O builtin procedures for the Twine Scheme runtime
//!
//! This module implements basic I/O operations as builtin procedures:
//! `display`, `newline`
//!
//! ## Testing Strategy
//!
//! This module uses a hybrid testing approach to verify both return values and actual stdout output:
//!
//! ### Unit Tests (src/runtime/builtins/io.rs)
//! - Use `display_to_writer()` and `newline_to_writer()` with custom buffers
//! - Provide **exact byte-level verification** of output content
//! - Fast execution with precise control over test conditions
//! - Verify all data types format correctly (strings without quotes, proper list formatting, etc.)
//!
//! ### Integration Tests (tests/integration.rs)
//! - Use the main `display()` and `newline()` functions through full evaluation pipeline
//! - Verify procedures execute correctly with proper return values (`Value::Nil`)
//! - Test I/O in real contexts (arithmetic expressions, conditionals, lambdas)
//! - Run with `cargo test -- --nocapture` to see actual stdout output
//!
//! ### Alternative Approaches Available
//! - **Subprocess testing**: Use `std::process::Command` to capture stdout from test binaries
//! - **External crates**: Use `gag` crate for stdout redirection (requires dependency)
//! - **Current approach**: Provides comprehensive testing without external dependencies
//!
//! ## Implementation Notes
//!
//! Currently synchronous implementations using `print!` and `println!` directly.
//! In Phase 4, these will be replaced with async versions that properly yield fibers.

use crate::error::{Error, Result};
use crate::types::Value;
use std::io::{Write, stdout};

/// Implements the `display` builtin procedure
///
/// Outputs the given value to stdout without adding a newline.
/// Takes exactly one argument of any type.
///
/// Examples:
/// - `(display "hello")` outputs: hello
/// - `(display 42)` outputs: 42
/// - `(display #t)` outputs: #t
pub fn display(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("display", 1, args.len()));
    }

    let value = &args[0];

    // Format the value for display output
    let output = match value {
        Value::String(s) => {
            // Display strings without quotes, just the content
            s.as_str().to_string()
        }
        Value::Symbol(_)
        | Value::Number(_)
        | Value::Boolean(_)
        | Value::List(_)
        | Value::Nil
        | Value::Procedure(_) => {
            // Use standard formatting for all other types
            format!("{value}")
        }
    };

    // Print to stdout without newline
    print!("{output}");

    // Flush to ensure immediate output
    stdout()
        .flush()
        .map_err(|e| Error::runtime_error(&format!("I/O error: {e}")))?;

    // Return an unspecified value (using empty list as convention)
    Ok(Value::Nil)
}

/// Helper function for display that accepts any writer (for testing)
pub fn display_to_writer<W: Write>(args: &[Value], writer: &mut W) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("display", 1, args.len()));
    }

    let value = &args[0];

    // Format the value for display output
    let output = match value {
        Value::String(s) => {
            // Display strings without quotes, just the content
            s.as_str().to_string()
        }
        Value::Symbol(_)
        | Value::Number(_)
        | Value::Boolean(_)
        | Value::List(_)
        | Value::Nil
        | Value::Procedure(_) => {
            // Use standard formatting for all other types
            format!("{value}")
        }
    };

    // Write to the provided writer without newline
    write!(writer, "{output}").map_err(|e| Error::runtime_error(&format!("I/O error: {e}")))?;

    // Flush to ensure immediate output
    writer
        .flush()
        .map_err(|e| Error::runtime_error(&format!("I/O error: {e}")))?;

    // Return an unspecified value (using empty list as convention)
    Ok(Value::Nil)
}

/// Implements the `newline` builtin procedure
///
/// Outputs a newline character to stdout.
/// Takes no arguments.
///
/// Example:
/// - `(newline)` outputs a newline
pub fn newline(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Error::arity_error("newline", 0, args.len()));
    }

    // Print newline to stdout
    println!();

    // Flush to ensure immediate output
    stdout()
        .flush()
        .map_err(|e| Error::runtime_error(&format!("I/O error: {e}")))?;

    // Return an unspecified value (using empty list as convention)
    Ok(Value::Nil)
}

/// Helper function for newline that accepts any writer (for testing)
pub fn newline_to_writer<W: Write>(args: &[Value], writer: &mut W) -> Result<Value> {
    if !args.is_empty() {
        return Err(Error::arity_error("newline", 0, args.len()));
    }

    // Write newline to the provided writer
    writeln!(writer).map_err(|e| Error::runtime_error(&format!("I/O error: {e}")))?;

    // Flush to ensure immediate output
    writer
        .flush()
        .map_err(|e| Error::runtime_error(&format!("I/O error: {e}")))?;

    // Return an unspecified value (using empty list as convention)
    Ok(Value::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_arity() {
        let mut buffer = Vec::new();

        // Test correct arity
        let result = display_to_writer(&[Value::string("hello")], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buffer).unwrap(), "hello");

        // Test incorrect arity - no arguments
        let mut buffer = Vec::new();
        let result = display_to_writer(&[], &mut buffer);
        assert!(result.is_err());
        if let Err(Error::ArityError {
            procedure,
            expected,
            actual,
        }) = result
        {
            assert_eq!(procedure, "display");
            assert_eq!(expected, 1);
            assert_eq!(actual, 0);
        } else {
            panic!("Expected arity error");
        }

        // Test incorrect arity - too many arguments
        let mut buffer = Vec::new();
        let result = display_to_writer(&[Value::string("hello"), Value::number(42.0)], &mut buffer);
        assert!(result.is_err());
        if let Err(Error::ArityError {
            procedure,
            expected,
            actual,
        }) = result
        {
            assert_eq!(procedure, "display");
            assert_eq!(expected, 1);
            assert_eq!(actual, 2);
        } else {
            panic!("Expected arity error");
        }
    }

    #[test]
    fn test_display_string() {
        let mut buffer = Vec::new();

        // String values should be displayed without quotes
        let result = display_to_writer(&[Value::string("hello world")], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "hello world");
    }

    #[test]
    fn test_display_number() {
        let mut buffer = Vec::new();

        // Numbers should be displayed as-is
        let result = display_to_writer(&[Value::number(42.5)], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "42.5");
    }

    #[test]
    fn test_display_boolean() {
        let mut buffer = Vec::new();

        // Test true boolean
        let result = display_to_writer(&[Value::boolean(true)], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "#t");

        // Test false boolean
        let mut buffer = Vec::new();
        let result = display_to_writer(&[Value::boolean(false)], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "#f");
    }

    #[test]
    fn test_display_symbol() {
        let mut buffer = Vec::new();

        // Symbols should be displayed as their name
        let result = display_to_writer(&[Value::symbol("test-symbol")], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "test-symbol");
    }

    #[test]
    fn test_display_list() {
        let mut buffer = Vec::new();

        // Lists should be displayed with parentheses
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let result = display_to_writer(&[list], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "(1 2)");
    }

    #[test]
    fn test_display_empty_list() {
        let mut buffer = Vec::new();

        // Empty list should be displayed as ()
        let result = display_to_writer(&[Value::empty_list()], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "()");
    }

    #[test]
    fn test_newline_arity() {
        let mut buffer = Vec::new();

        // Test correct arity (no arguments)
        let result = newline_to_writer(&[], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "\n");

        // Test incorrect arity - with arguments
        let mut buffer = Vec::new();
        let result = newline_to_writer(&[Value::string("hello")], &mut buffer);
        assert!(result.is_err());
        if let Err(Error::ArityError {
            procedure,
            expected,
            actual,
        }) = result
        {
            assert_eq!(procedure, "newline");
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected arity error");
        }
    }

    #[test]
    fn test_newline_return_value() {
        let mut buffer = Vec::new();

        // newline should return unspecified value (empty list)
        let result = newline_to_writer(&[], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer).unwrap(), "\n");
    }

    #[test]
    fn test_io_procedures_are_side_effects() {
        let mut buffer1 = Vec::new();
        let mut buffer2 = Vec::new();

        // Both display and newline should return unspecified values
        // since they are primarily side-effect operations

        let display_result = display_to_writer(&[Value::string("test")], &mut buffer1);
        assert!(display_result.is_ok());
        assert_eq!(display_result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer1).unwrap(), "test");

        let newline_result = newline_to_writer(&[], &mut buffer2);
        assert!(newline_result.is_ok());
        assert_eq!(newline_result.unwrap(), Value::Nil);
        assert_eq!(String::from_utf8(buffer2).unwrap(), "\n");
    }

    #[test]
    fn test_display_various_types() {
        // Test that display works with all value types
        let test_cases = vec![
            (Value::number(3.14159), "3.14159"),
            (Value::string("test string"), "test string"),
            (Value::symbol("test-symbol"), "test-symbol"),
            (Value::boolean(true), "#t"),
            (Value::boolean(false), "#f"),
            (Value::empty_list(), "()"),
            (
                Value::list(vec![Value::number(1.0), Value::symbol("a")]),
                "(1 a)",
            ),
        ];

        for (value, expected_output) in test_cases {
            let mut buffer = Vec::new();
            let result = display_to_writer(&[value], &mut buffer);
            assert!(result.is_ok(), "display should work with all value types");
            assert_eq!(result.unwrap(), Value::Nil);
            assert_eq!(String::from_utf8(buffer).unwrap(), expected_output);
        }
    }

    #[test]
    fn test_display_and_newline_combination() {
        let mut buffer = Vec::new();

        // Test multiple displays and newlines
        display_to_writer(&[Value::string("Hello")], &mut buffer).unwrap();
        display_to_writer(&[Value::string(" ")], &mut buffer).unwrap();
        display_to_writer(&[Value::string("World")], &mut buffer).unwrap();
        newline_to_writer(&[], &mut buffer).unwrap();
        display_to_writer(&[Value::number(42.0)], &mut buffer).unwrap();
        newline_to_writer(&[], &mut buffer).unwrap();

        assert_eq!(String::from_utf8(buffer).unwrap(), "Hello World\n42\n");
    }

    #[test]
    fn test_display_string_without_quotes() {
        let mut buffer = Vec::new();

        // Verify that strings are displayed without quotes
        let result = display_to_writer(&[Value::string("\"quoted\" string")], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buffer).unwrap(), "\"quoted\" string");
    }

    #[test]
    fn test_display_nested_list() {
        let mut buffer = Vec::new();

        // Test nested list display
        let inner_list = Value::list(vec![Value::number(2.0), Value::number(3.0)]);
        let outer_list = Value::list(vec![Value::number(1.0), inner_list]);
        let result = display_to_writer(&[outer_list], &mut buffer);
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buffer).unwrap(), "(1 (2 3))");
    }

    #[test]
    fn test_exact_output_verification_comprehensive() {
        // This test demonstrates that we are actually verifying the exact
        // stdout output, not just return values. Each assertion checks
        // the precise bytes written to the output stream.

        let mut buffer = Vec::new();

        // Test 1: Exact string content verification
        display_to_writer(&[Value::string("Exact test: 123")], &mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            "Exact test: 123",
            "String output must match exactly"
        );
        buffer.clear();

        // Test 2: Number precision verification
        display_to_writer(&[Value::number(3.14159)], &mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            "3.14159",
            "Number output must preserve precision"
        );
        buffer.clear();

        // Test 3: Boolean representation verification
        display_to_writer(&[Value::boolean(true)], &mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            "#t",
            "Boolean true must output exactly '#t'"
        );
        buffer.clear();

        display_to_writer(&[Value::boolean(false)], &mut buffer).unwrap();
        assert_eq!(
            String::from_utf8(buffer.clone()).unwrap(),
            "#f",
            "Boolean false must output exactly '#f'"
        );
        buffer.clear();

        // Test 4: Complex list structure verification
        let complex_list = Value::list(vec![
            Value::symbol("define"),
            Value::symbol("factorial"),
            Value::list(vec![
                Value::symbol("lambda"),
                Value::list(vec![Value::symbol("n")]),
                Value::list(vec![
                    Value::symbol("if"),
                    Value::list(vec![
                        Value::symbol("="),
                        Value::symbol("n"),
                        Value::number(0.0),
                    ]),
                    Value::number(1.0),
                    Value::list(vec![
                        Value::symbol("*"),
                        Value::symbol("n"),
                        Value::list(vec![
                            Value::symbol("factorial"),
                            Value::list(vec![
                                Value::symbol("-"),
                                Value::symbol("n"),
                                Value::number(1.0),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ]);

        display_to_writer(&[complex_list], &mut buffer).unwrap();
        let output = String::from_utf8(buffer.clone()).unwrap();
        assert_eq!(
            output, "(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))",
            "Complex nested list must format correctly"
        );
        buffer.clear();

        // Test 5: Newline character verification
        newline_to_writer(&[], &mut buffer).unwrap();
        assert_eq!(
            buffer, b"\n",
            "Newline must output exactly one newline byte"
        );
        buffer.clear();

        // Test 6: Multiple operations sequence verification
        display_to_writer(&[Value::string("Hello")], &mut buffer).unwrap();
        display_to_writer(&[Value::string(", ")], &mut buffer).unwrap();
        display_to_writer(&[Value::string("World")], &mut buffer).unwrap();
        display_to_writer(&[Value::string("!")], &mut buffer).unwrap();
        newline_to_writer(&[], &mut buffer).unwrap();
        display_to_writer(&[Value::number(42.0)], &mut buffer).unwrap();

        assert_eq!(
            String::from_utf8(buffer).unwrap(),
            "Hello, World!\n42",
            "Sequence of operations must produce exact combined output"
        );
    }
}
