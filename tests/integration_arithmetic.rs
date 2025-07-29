//! Integration tests for arithmetic operations
//!
//! This file contains integration tests for arithmetic functionality:
//! - Basic arithmetic operations (+, -, *, /)
//! - Arithmetic with variables and expressions
//! - Arithmetic edge cases and error handling
//! - Complex arithmetic expressions

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
fn test_integration_arithmetic_operations() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("x", Value::number(10.0));
    env.define_str("y", Value::number(3.0));

    // Basic arithmetic
    let result = eval_source("(+ 1 2 3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0);

    let result = eval_source("(- 10 3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 7.0);

    let result = eval_source("(* 4 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0);

    let result = eval_source("(/ 15 3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    // Arithmetic with identifiers
    let result = eval_source("(+ x y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 13.0);

    let result = eval_source("(* x y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Nested arithmetic
    let result = eval_source("(+ (* 2 3) (- 10 5))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 11.0);

    // Multiple arguments
    let result = eval_source("(+ 1 2 3 4 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);

    let result = eval_source("(* 2 3 4)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 24.0);
}

#[test]
fn test_integration_arithmetic_edge_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test arithmetic with zero
    let result = eval_source("(+ 0 5 0)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    let result = eval_source("(* 0 100)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);

    let result = eval_source("(- 10 0)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test negative numbers
    let result = eval_source("(+ -5 10)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    let result = eval_source("(* -2 3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), -6.0);

    let result = eval_source("(- -5 -3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), -2.0);

    // Test decimal numbers
    let result = eval_source("(+ 1.5 2.5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);

    let result = eval_source("(* 0.5 4)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 2.0);

    // Test single argument operations
    let result = eval_source("(+ 42)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    let result = eval_source("(* 7)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 7.0);

    // Test subtraction and division with single argument (negation and reciprocal)
    let result = eval_source("(- 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), -5.0);

    let result = eval_source("(/ 4)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.25);
}

#[test]
fn test_integration_arithmetic_with_list_length() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str(
        "my-list",
        Value::list(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]),
    );

    // Test arithmetic using list length
    let result = eval_source("(+ (length my-list) 2)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0); // 3 + 2

    let result = eval_source("(* (length my-list) 10)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0); // 3 * 10

    // Test with empty list
    env.define_str("empty-list", Value::list(vec![]));
    let result = eval_source("(+ (length empty-list) 1)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0); // 0 + 1
}

#[test]
fn test_integration_complex_list_arithmetic() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test arithmetic on multiple list elements
    let result = eval_source("(+ (car '(10 20)) (car (cdr '(10 20))))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Test complex nested expression with lists and arithmetic
    let result = eval_source("(* (car '(3 4)) (+ 5 (car (cdr '(1 2)))))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 21.0); // 3 * (5 + 2) = 3 * 7 = 21

    // Test arithmetic with nested list operations
    let result = eval_source(
        "(- (+ (car '(15 10)) (car (cdr '(15 10)))) (car '(5 3)))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0); // (15 + 10) - 5 = 25 - 5 = 20
}

#[test]
fn test_integration_arithmetic_error_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Type error - arithmetic with non-number
    let result = eval_source("(+ 1 \"not a number\")", &mut env);
    assert!(result.is_err());

    let result = eval_source("(* 5 #t)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(- 10 '(1 2))", &mut env);
    assert!(result.is_err());

    // Division by zero
    let result = eval_source("(/ 5 0)", &mut env);
    assert!(result.is_err());

    // No arguments (except for + and * which have identity elements)
    let result = eval_source("(-)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(/)", &mut env);
    assert!(result.is_err());

    // But + and * with no arguments should work
    let result = eval_source("(+)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);

    let result = eval_source("(*)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);
}

#[test]
fn test_integration_arithmetic_with_expressions() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("base", Value::number(100.0));
    env.define_str("multiplier", Value::number(2.0));

    // Arithmetic with complex expressions
    let result = eval_source("(+ (* base multiplier) (- 50 10))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 240.0); // (100 * 2) + (50 - 10) = 200 + 40

    // Nested arithmetic with conditionals
    let result = eval_source("(* (if (> base 50) 3 1) base)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 300.0); // 3 * 100

    // Complex expression with multiple operations
    let result = eval_source("(+ (/ (* base 4) 2) (- base 50))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 250.0); // (400 / 2) + (100 - 50) = 200 + 50

    // Arithmetic with quoted lists and car/cdr
    let result = eval_source("(+ (* (car '(5 10)) 2) (car (cdr '(20 30))))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 40.0); // (5 * 2) + 30 = 10 + 30
}

#[test]
fn test_integration_arithmetic_precision() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test floating point precision
    let result = eval_source("(+ 0.1 0.2)", &mut env).unwrap();
    // Note: This might have floating point precision issues
    let value = result.as_number().unwrap();
    assert!((value - 0.3).abs() < 0.0001);

    // Test with larger decimal numbers
    let result = eval_source("(* 3.14159 2)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 6.28318);

    // Test very small numbers
    let result = eval_source("(+ 0.0001 0.0002)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0003);

    // Test very large numbers
    let result = eval_source("(+ 1000000 2000000)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3000000.0);
}
