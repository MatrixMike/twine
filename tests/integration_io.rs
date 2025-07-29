//! Integration tests for I/O operations
//!
//! This file contains integration tests for I/O functionality:
//! - Display builtin procedure
//! - Newline builtin procedure
//! - I/O in complex expressions and procedures
//! - I/O error handling
//! - Subprocess-based output capture for verification

mod common;

use common::{eval_source, test_io};
use twine_scheme::runtime::Environment;
use twine_scheme::types::Value;

#[test]
fn test_integration_display_builtin() {
    // Test display with string - capture actual stdout
    test_io("(display \"Hello, World!\")", "Hello, World!");

    // Test display with number
    test_io("(display 42)", "42");

    // Test display with boolean
    test_io("(display #t)", "#t");
    test_io("(display #f)", "#f");

    // Test display with symbol
    test_io("(display 'hello)", "hello");

    // Test display with list
    test_io("(display '(1 2 3))", "(1 2 3)");

    // Test display with empty string
    test_io("(display \"\")", "");
}

#[test]
fn test_integration_newline_builtin() {
    // Test newline - capture actual stdout
    test_io("(newline)", "\n");

    // Test multiple newlines
    test_io("(newline) (newline)", "\n\n");
}

#[test]
fn test_integration_display_and_newline_combination() {
    // Test display followed by newline - capture combined output
    test_io("(display \"Hello\") (newline)", "Hello\n");

    // Test multiple display and newline combinations
    test_io(
        "(display \"Line 1\") (newline) (display \"Line 2\") (newline)",
        "Line 1\nLine 2\n",
    );

    // Test mixed display types with newlines
    test_io(
        "(display 42) (newline) (display #t) (newline) (display 'symbol)",
        "42\n#t\nsymbol",
    );

    // Test display without newline at end
    test_io("(display \"No newline\")", "No newline");
}

#[test]
fn test_integration_io_in_expressions() {
    // Test I/O in lambda - capture actual output
    test_io(
        "(define print-hello (lambda () (display \"Hello\") (newline))) (print-hello)",
        "Hello\n",
    );

    // Test I/O in conditional
    test_io(
        "(if #t (display \"True branch\") (display \"False branch\"))",
        "True branch",
    );

    // Test I/O in let expression
    test_io(
        "(let ((msg \"From let\")) (display msg) (newline))",
        "From let\n",
    );

    // Test I/O with arithmetic results
    test_io("(display (+ 10 32)) (newline)", "42\n");

    // Test I/O with list operations
    test_io("(display (car '(first second))) (newline)", "first\n");
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
    assert_eq!(result, Value::number(42.0)); // display returns nil, which is truthy in Scheme

    // Test using I/O in arithmetic context should fail
    let result = eval_source("(+ 1 (display \"test\"))", &mut env);
    assert!(result.is_err()); // Can't add nil to number
}

#[test]
fn test_integration_io_subprocess_capture() {
    // This test demonstrates subprocess-based stdout capture for integration testing
    // It verifies that I/O procedures produce the exact expected output

    test_io(
        r#"(let ((name "World"))
             (display "Hello, ")
             (display name)
             (display "!")
             (newline))"#,
        "Hello, World!\n",
    );

    // Test with more complex expression
    test_io(
        r#"(define print-list
             (lambda (lst)
               (if (null? lst)
                 (newline)
                 (begin
                   (display (car lst))
                   (display " ")
                   (print-list (cdr lst))))))
           (print-list '(1 2 3))"#,
        "1 2 3 \n",
    );
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
    // Test comprehensive I/O functionality with subprocess stdout capture
    // This verifies exact output for all supported value types

    // Test string display
    test_io(
        r#"(display "String: ") (display "hello world") (newline)"#,
        "String: hello world\n",
    );

    // Test number display
    test_io(
        "(display \"Number: \") (display 42.5) (newline)",
        "Number: 42.5\n",
    );

    // Test boolean display
    test_io(
        "(display \"Boolean: \") (display #t) (display \" \") (display #f) (newline)",
        "Boolean: #t #f\n",
    );

    // Test symbol display
    test_io(
        "(display \"Symbol: \") (display 'test-symbol) (newline)",
        "Symbol: test-symbol\n",
    );

    // Test list display
    test_io(
        "(display \"List: \") (display '(1 2 3)) (newline)",
        "List: (1 2 3)\n",
    );

    // Test empty list display
    test_io(
        "(display \"Empty: \") (display '()) (newline)",
        "Empty: ()\n",
    );

    // Test nested list display
    test_io(
        "(display \"Nested: \") (display '((a b) (c d))) (newline)",
        "Nested: ((a b) (c d))\n",
    );

    // Test mixed types in complex expression
    test_io(
        r#"(let ((items '("hello" 42 #t)))
             (display "Items: ")
             (display items)
             (newline))"#,
        "Items: (\"hello\" 42 #t)\n",
    );
}

#[test]
fn test_integration_io_with_procedures() {
    // Test I/O within user-defined procedures
    test_io(
        r#"(define greet
             (lambda (name)
               (display "Hello, ")
               (display name)
               (display "!")
               (newline)))
           (greet "Alice")"#,
        "Hello, Alice!\n",
    );

    // Test I/O in recursive-style procedure
    test_io(
        r#"(define print-numbers
             (lambda (n)
               (if (> n 0)
                 (begin
                   (display n)
                   (display " ")
                   (print-numbers (- n 1)))
                 (newline))))
           (print-numbers 3)"#,
        "3 2 1 \n",
    );

    // Test I/O in conditional procedures
    test_io(
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
        "5 is positive\n-3 is negative\n0 is zero\n",
    );
}

#[test]
fn test_integration_io_with_let_and_define() {
    // Test I/O with let bindings
    test_io(
        r#"(let ((message "Local message")
                  (count 3))
             (display message)
             (display " repeated ")
             (display count)
             (display " times")
             (newline))"#,
        "Local message repeated 3 times\n",
    );

    // Test I/O with define in local scope
    test_io(
        r#"(let ((prefix ">>"))
             (define print-with-prefix
               (lambda (text)
                 (display prefix)
                 (display " ")
                 (display text)
                 (newline)))
             (print-with-prefix "Line 1")
             (print-with-prefix "Line 2"))"#,
        ">> Line 1\n>> Line 2\n",
    );
}
