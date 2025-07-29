//! Integration tests for I/O operations
//!
//! This file contains integration tests for I/O functionality:
//! - Display builtin procedure
//! - Newline builtin procedure
//! - I/O in complex expressions and procedures
//! - I/O error handling
//! - Subprocess-based output capture for verification

use twine_scheme::runtime::Environment;
use twine_scheme::types::Value;
use twine_scheme::{Error, Result};

// Helper function for end-to-end evaluation testing
fn eval_source(
    source: &str,
    env: &mut twine_scheme::runtime::Environment,
) -> Result<twine_scheme::types::Value> {
    use twine_scheme::parser::Parser;
    use twine_scheme::runtime::eval::eval;

    let mut parser = Parser::new(source.to_string())?;
    let expr = parser.parse_expression()?.expr;
    eval(expr, env)
}

#[test]
fn test_integration_display_builtin() {
    use std::process::Command;

    // Test display with string - capture actual stdout
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display \"Hello, World!\")"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Hello, World!");

    // Test display with number
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display 42)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42");

    // Test display with boolean
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display #t)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "#t");

    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display #f)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "#f");

    // Test display with symbol
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display 'hello)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "hello");

    // Test display with list
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display '(1 2 3))"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "(1 2 3)");

    // Test display with empty string
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display \"\")"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
}

#[test]
fn test_integration_newline_builtin() {
    use std::process::Command;

    // Test newline - capture actual stdout
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(newline)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "\n");

    // Test multiple newlines
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(newline) (newline)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "\n\n");
}

#[test]
fn test_integration_display_and_newline_combination() {
    use std::process::Command;

    // Test display followed by newline - capture combined output
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display \"Hello\") (newline)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Hello\n");

    // Test multiple display and newline combinations
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display \"Line 1\") (newline) (display \"Line 2\") (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Line 1\nLine 2\n"
    );

    // Test mixed display types with newlines
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display 42) (display \" is the answer\") (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "42 is the answer\n"
    );

    // Test display without newline at end
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display \"No newline\")"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "No newline");
}

#[test]
fn test_integration_io_in_expressions() {
    use std::process::Command;

    // Test I/O in lambda - capture actual output
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(define print-hello (lambda () (display \"Hello\") (newline))) (print-hello)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Hello\n");

    // Test I/O in conditional
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(if #t (display \"True branch\") (display \"False branch\"))",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "True branch");

    // Test I/O in let expression
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(let ((msg \"Let message\")) (display msg) (newline))",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Let message\n");

    // Test I/O with arithmetic results
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display (+ 10 32)) (newline)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");

    // Test I/O with list operations
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display (car '(first second))) (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "first\n");
}

#[test]
fn test_integration_io_return_values() {
    let mut env = Environment::new();

    // Test that display returns nil
    let result = eval_source("(display \"test\")", &mut env).unwrap();
    assert_eq!(result, Value::Nil);

    // Test that newline returns nil
    let result = eval_source("(newline)", &mut env).unwrap();
    assert_eq!(result, Value::Nil);

    // Test I/O in expressions where return value matters
    let result = eval_source("(if (display \"side effect\") 42 99)", &mut env).unwrap();
    assert_eq!(result, Value::number(99.0)); // display returns nil, which is falsy

    // Test using I/O in arithmetic context should fail
    let result = eval_source("(+ 1 (display \"test\"))", &mut env);
    assert!(result.is_err()); // Can't add nil to number
}

#[test]
fn test_integration_io_subprocess_capture() {
    use std::process::Command;

    // This test demonstrates subprocess-based stdout capture for integration testing
    // It verifies that I/O procedures produce the exact expected output

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(let ((name "World"))
                 (display "Hello, ")
                 (display name)
                 (display "!")
                 (newline))"#,
        ])
        .output()
        .expect("Failed to execute test binary");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Hello, World!\n");

    // Test with more complex expression
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(define print-list
                 (lambda (lst)
                   (if (null? lst)
                     (newline)
                     (begin
                       (display (car lst))
                       (display " ")
                       (print-list (cdr lst))))))
               (print-list '(1 2 3))"#,
        ])
        .output()
        .expect("Failed to execute test binary");

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "1 2 3 \n");
}

#[test]
fn test_integration_io_error_handling() {
    use std::process::Command;

    // Test that I/O errors are properly propagated through subprocess
    // When the test binary encounters an error, it should exit with non-zero status

    // Test display with invalid arity
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(!output.status.success()); // Should fail

    // Test display with too many arguments
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display \"a\" \"b\")"])
        .output()
        .expect("Failed to execute test binary");
    assert!(!output.status.success()); // Should fail

    // Test newline with arguments
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(newline \"invalid\")"])
        .output()
        .expect("Failed to execute test binary");
    assert!(!output.status.success()); // Should fail

    // Test I/O with undefined identifier
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display undefined-var)"])
        .output()
        .expect("Failed to execute test binary");
    assert!(!output.status.success()); // Should fail

    // Test I/O with evaluation error
    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", "(display (+ 1 \"not-number\"))"])
        .output()
        .expect("Failed to execute test binary");
    assert!(!output.status.success()); // Should fail
}

#[test]
fn test_integration_io_comprehensive_output() {
    use std::process::Command;

    // Test comprehensive I/O functionality with subprocess stdout capture
    // This verifies exact output for all supported value types

    // Test string display
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(display "String: ") (display "hello world") (newline)"#,
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "String: hello world\n"
    );

    // Test number display
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display \"Number: \") (display 42.5) (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Number: 42.5\n");

    // Test boolean display
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display \"Boolean: \") (display #t) (display \" \") (display #f) (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Boolean: #t #f\n"
    );

    // Test symbol display
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display \"Symbol: \") (display 'test-symbol) (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Symbol: test-symbol\n"
    );

    // Test list display
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display \"List: \") (display '(1 2 3)) (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "List: (1 2 3)\n");

    // Test empty list display
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display \"Empty: \") (display '()) (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Empty: ()\n");

    // Test nested list display
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            "(display \"Nested: \") (display '((a b) (c d))) (newline)",
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Nested: ((a b) (c d))\n"
    );

    // Test mixed types in complex expression
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(let ((items '("hello" 42 #t)))
                 (display "Items: ")
                 (display items)
                 (newline))"#,
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Items: (\"hello\" 42 #t)\n"
    );
}

#[test]
fn test_integration_io_with_procedures() {
    use std::process::Command;

    // Test I/O within user-defined procedures
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(define greet
                 (lambda (name)
                   (display "Hello, ")
                   (display name)
                   (display "!")
                   (newline)))
               (greet "Alice")"#,
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Hello, Alice!\n");

    // Test I/O in recursive-style procedure
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(define print-numbers
                 (lambda (n)
                   (if (> n 0)
                     (begin
                       (display n)
                       (display " ")
                       (print-numbers (- n 1)))
                     (newline))))
               (print-numbers 3)"#,
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "3 2 1 \n");

    // Test I/O in conditional procedures
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(define describe-number
                 (lambda (n)
                   (display n)
                   (display " is ")
                   (if (> n 0)
                     (display "positive")
                     (if (< n 0)
                       (display "negative")
                       (display "zero")))
                   (newline)))
               (describe-number 5)
               (describe-number -3)
               (describe-number 0)"#,
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "5 is positive\n-3 is negative\n0 is zero\n"
    );
}

#[test]
fn test_integration_io_with_let_and_define() {
    use std::process::Command;

    // Test I/O with let bindings
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(let ((message "Local message")
                      (count 3))
                 (display message)
                 (display " repeated ")
                 (display count)
                 (display " times")
                 (newline))"#,
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Local message repeated 3 times\n"
    );

    // Test I/O with define in local scope
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "test_io",
            r#"(let ((prefix ">>"))
                 (define print-with-prefix
                   (lambda (text)
                     (display prefix)
                     (display " ")
                     (display text)
                     (newline)))
                 (print-with-prefix "Line 1")
                 (print-with-prefix "Line 2"))"#,
        ])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        ">> Line 1\n>> Line 2\n"
    );
}
