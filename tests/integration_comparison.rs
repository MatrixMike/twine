//! Integration tests for comparison operations
//!
//! This file contains integration tests for comparison functionality:
//! - Equality and inequality operators (=, <, >, <=, >=)
//! - Comparison with variables and expressions
//! - Comparison edge cases and error handling
//! - Complex comparison expressions

mod common;

use common::eval_source;
use twine_scheme::runtime::Environment;

#[test]
fn test_integration_comparison_operations() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("a", Value::number(5.0));
    env.define_str("b", Value::number(3.0));

    // Equality
    let result = eval_source("(= 5 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(= 5 3)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(= a 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Less than
    let result = eval_source("(< 3 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(< 5 3)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(< b a)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Greater than
    let result = eval_source("(> 5 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(> 3 5)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(> a b)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Less than or equal
    let result = eval_source("(<= 3 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(<= 5 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(<= 5 3)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Greater than or equal
    let result = eval_source("(>= 5 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(>= 5 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(>= 3 5)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());
}

#[test]
fn test_integration_comparison_edge_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test equality with same values
    let result = eval_source("(= 5 5 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(= 5 5 6)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test with zero
    let result = eval_source("(= 0 0)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(< 0 1)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(> 0 -1)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test with negative numbers
    let result = eval_source("(< -5 -3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(> -3 -5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(= -0 0)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test with decimal numbers
    let result = eval_source("(< 1.5 1.6)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(= 2.0 2)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test chained comparisons
    let result = eval_source("(< 1 2 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(< 1 3 2)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(> 5 4 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
}

#[test]
fn test_integration_comparison_with_expressions() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("x", Value::number(10.0));
    env.define_str("y", Value::number(5.0));

    // Comparison with arithmetic expressions
    let result = eval_source("(> (+ x y) 10)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap()); // 15 > 10

    let result = eval_source("(= (* y 2) x)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap()); // 10 = 10

    let result = eval_source("(< (- x y) y)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap()); // 5 < 5 is false

    let result = eval_source("(<= (- x y) y)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap()); // 5 <= 5 is true

    // Complex nested expressions
    let result = eval_source("(> (+ (* x 2) y) (- (* y 4) 3))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap()); // (20 + 5) > (20 - 3) => 25 > 17

    // Comparison with list elements
    let result = eval_source("(< (car '(3 7)) (car (cdr '(3 7))))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap()); // 3 < 7
}

#[test]
fn test_integration_comparison_error_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Type error - comparing number with non-number
    let result = eval_source("(< 5 \"hello\")", &mut env);
    assert!(result.is_err());

    let result = eval_source("(= 42 #t)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(> 10 '(1 2))", &mut env);
    assert!(result.is_err());

    // Too few arguments
    let result = eval_source("(=)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(<)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(= 5)", &mut env);
    assert!(result.is_err());

    // Mixed type comparisons should fail
    let result = eval_source("(< \"a\" \"b\")", &mut env);
    assert!(result.is_err()); // String comparison not supported

    let result = eval_source("(= '() '())", &mut env);
    assert!(result.is_err()); // List comparison not supported for =
}

#[test]
fn test_integration_comparison_with_conditionals() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("threshold", Value::number(50.0));

    // Comparison results used in conditionals
    let result = eval_source("(if (> 75 threshold) \"high\" \"low\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "high");

    let result = eval_source("(if (< 25 threshold) \"below\" \"above\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "below");

    let result = eval_source("(if (= 50 threshold) \"equal\" \"different\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "equal");

    // Nested conditionals with comparisons
    let result = eval_source(
        "(if (> 75 threshold) (if (< 75 100) \"medium-high\" \"very-high\") \"low\")",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "medium-high");

    // Multiple comparisons in conditional
    env.define_str("min", Value::number(10.0));
    env.define_str("max", Value::number(90.0));
    env.define_str("value", Value::number(45.0));

    let result = eval_source(
        "(if (and (>= value min) (<= value max)) \"in-range\" \"out-of-range\")",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "in-range");
}

#[test]
fn test_integration_comparison_precision() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test floating point comparison precision
    let result = eval_source("(= 0.1 0.1)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test very small differences
    let result = eval_source("(< 1.0001 1.0002)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test large numbers
    let result = eval_source("(= 1000000.0 1000000)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test negative zero
    let result = eval_source("(= -0.0 0.0)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
}

#[test]
fn test_integration_comparison_comprehensive() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Set up test data
    env.define_str(
        "numbers",
        Value::list(vec![
            Value::number(5.0),
            Value::number(10.0),
            Value::number(15.0),
        ]),
    );

    // Complex comparison scenarios
    let result = eval_source(
        "(< (car numbers) (car (cdr numbers)) (car (cdr (cdr numbers))))",
        &mut env,
    )
    .unwrap();
    assert!(result.as_boolean().unwrap()); // 5 < 10 < 15

    // Comparison with arithmetic on list elements
    let result = eval_source("(= (* (car numbers) 2) (car (cdr numbers)))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap()); // 5 * 2 = 10

    // Mixed operations
    env.define_str("factor", Value::number(3.0));
    let result = eval_source(
        "(> (+ (car numbers) factor) (- (car (cdr numbers)) 1))",
        &mut env,
    )
    .unwrap();
    assert!(!result.as_boolean().unwrap()); // (5 + 3) > (10 - 1) => 8 > 9 is false
}
