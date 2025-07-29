//! Integration tests for lambda expressions
//!
//! This file contains integration tests for lambda expression functionality:
//! - Lambda creation (basic, no parameters, multiple parameters)
//! - Lambda environment capture and closures
//! - Lambda with different body types
//! - Lambda error handling
//! - Lambda with special form names as parameters
//! - Lambda expressions with closures
//! - Complex lambda expressions and applications
//! - Error handling for lambda syntax

mod common;

use common::eval_source;
use twine_scheme::runtime::Environment;
use twine_scheme::types::{Symbol, Value};

#[test]
fn test_integration_lambda_creation_basic() {
    let mut env = Environment::new();

    // Test basic lambda creation: (lambda (x) x)
    let identity_lambda = "(lambda (x) x)";
    let result = eval_source(identity_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}

#[test]
fn test_integration_lambda_creation_no_parameters() {
    let mut env = Environment::new();

    // Test lambda with no parameters: (lambda () 42)
    let constant_lambda = "(lambda () 42)";
    let result = eval_source(constant_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(0));
}

#[test]
fn test_integration_lambda_creation_multiple_parameters() {
    let mut env = Environment::new();

    // Test lambda with multiple parameters: (lambda (x y z) (+ x y z))
    let multi_param_lambda = "(lambda (x y z) (+ x y z))";
    let result = eval_source(multi_param_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(3));
}

#[test]
fn test_integration_lambda_environment_capture() {
    let mut env = Environment::new();

    // Define a value in the environment
    let define_expr = "(define outer-value 100)";
    eval_source(define_expr, &mut env).unwrap();

    // Create lambda that references the outer value
    let capturing_lambda = "(lambda (x) (+ x outer-value))";
    let result = eval_source(capturing_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));

    // The lambda should capture the current environment
    // We can't directly test the captured environment, but we know it was created
}

#[test]
fn test_integration_lambda_error_cases() {
    let mut env = Environment::new();

    // Test lambda with wrong arity
    let wrong_arity = "(lambda)";
    let result = eval_source(wrong_arity, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("requires parameter list and body")
    );

    // Test lambda with no body
    let no_body = "(lambda (x))";
    let result = eval_source(no_body, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("requires parameter list and body")
    );

    // Test lambda with invalid parameter list
    let invalid_params = "(lambda x x)";
    let result = eval_source(invalid_params, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("parameter list must be a list")
    );

    // Test lambda with non-symbol parameter
    let non_symbol_param = "(lambda (42) x)";
    let result = eval_source(non_symbol_param, &mut env);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be a symbol"));

    // Test lambda with duplicate parameters
    let duplicate_params = "(lambda (x x) x)";
    let result = eval_source(duplicate_params, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("duplicate parameter")
    );

    // Test lambda with unbound identifier in body (during creation, not application)
    // This should succeed during lambda creation - error happens during application
    let unbound_in_body = "(lambda (x) (+ x undefined-var))";
    let result = eval_source(unbound_in_body, &mut env);
    assert!(result.is_ok()); // Lambda creation should succeed
    assert!(result.unwrap().is_procedure());
}

#[test]
fn test_integration_lambda_with_define() {
    let mut env = Environment::new();

    // Define a lambda procedure
    let define_lambda = "(define square (lambda (x) (* x x)))";
    let result = eval_source(define_lambda, &mut env).unwrap();
    assert_eq!(result, Value::Nil);

    // Verify the lambda was stored correctly
    let square_proc = env.lookup(&Symbol::new("square")).unwrap();
    assert!(square_proc.is_procedure());
    let proc = square_proc.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}

#[test]
fn test_integration_lambda_nested_environments() {
    let mut env = Environment::new();

    // Create nested environments with lambda
    let nested_expr = r#"
        (let ((outer 10))
          (lambda (x)
            (let ((inner 5))
              (+ x outer inner))))
    "#;

    let result = eval_source(nested_expr, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));

    // The lambda should capture both outer and its parameter environment
}

#[test]
fn test_integration_lambda_complex_body() {
    let mut env = Environment::new();

    // Test lambda with complex body expression
    let complex_lambda = r#"
        (lambda (x)
          (if (> x 0)
            (* x 2)
            (- x)))
    "#;

    let result = eval_source(complex_lambda, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}

#[test]
fn test_integration_lambda_with_special_form_names() {
    let mut env = Environment::new();

    // Lambda can use special form names as parameter names
    let special_names_lambda = "(lambda (if define lambda) (+ if define lambda))";
    let result = eval_source(special_names_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(3));

    // Test with other special forms
    let more_special = "(lambda (let quote cons) (* let quote))";
    let result = eval_source(more_special, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(3));
}

#[test]
fn test_integration_lambda_multiple_body_expressions() {
    let mut env = Environment::new();

    // Test lambda with multiple body expressions - should return last one
    let multi_body_lambda = r#"
        (lambda (x)
          (+ x 1)
          (+ x 2)
          (+ x 3))
    "#;

    let result = eval_source(multi_body_lambda, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}

#[test]
fn test_integration_lambda_with_lists() {
    let mut env = Environment::new();

    // Test lambda that works with lists
    let list_lambda = "(lambda (lst) (cons 'new lst))";
    let result = eval_source(list_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));

    // Test lambda with list operations in body
    let list_ops_lambda = "(lambda (x y) (list x y (+ x y)))";
    let result = eval_source(list_ops_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(2));
}

#[test]
fn test_integration_lambda_closure_behavior() {
    let mut env = Environment::new();

    // Test that lambda captures lexical environment
    eval_source("(define captured-value 42)", &mut env).unwrap();

    let closure_lambda = "(lambda (x) (+ x captured-value))";
    let result = eval_source(closure_lambda, &mut env).unwrap();

    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());

    // Test nested closure capture
    let nested_closure = r#"
        (let ((local-var 100))
          (lambda (x)
            (lambda (y)
              (+ x y local-var))))
    "#;

    let result = eval_source(nested_closure, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}

#[test]
fn test_integration_lambda_with_conditionals() {
    let mut env = Environment::new();

    // Test lambda with conditional expressions
    let conditional_lambda = r#"
        (lambda (x y)
          (if (> x y)
            (- x y)
            (+ x y)))
    "#;

    let result = eval_source(conditional_lambda, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(2));

    // Test lambda returning different types based on condition
    let type_conditional_lambda = r#"
        (lambda (flag)
          (if flag
            "true-branch"
            42))
    "#;

    let result = eval_source(type_conditional_lambda, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}

#[test]
fn test_integration_lambda_with_let_binding() {
    let mut env = Environment::new();

    // Test lambda with let expressions in body
    let let_lambda = r#"
        (lambda (x)
          (let ((temp (* x 2)))
            (+ temp 1)))
    "#;

    let result = eval_source(let_lambda, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));

    // Test lambda created within let
    let lambda_in_let = r#"
        (let ((factor 10))
          (lambda (x) (* x factor)))
    "#;

    let result = eval_source(lambda_in_let, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}

#[test]
fn test_integration_lambda_comprehensive() {
    let mut env = Environment::new();

    // Test complex lambda with multiple features
    eval_source("(define global-multiplier 5)", &mut env).unwrap();

    let comprehensive_lambda = r#"
        (lambda (data-list threshold)
          (let ((first (car data-list))
                (rest (cdr data-list)))
            (if (> first threshold)
              (let ((multiplied (* first global-multiplier)))
                (cons multiplied rest))
              (cons (+ first 10) rest))))
    "#;

    let result = eval_source(comprehensive_lambda, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(2));

    // Test lambda that returns another lambda
    let higher_order_lambda = r#"
        (lambda (operation)
          (if (eq? operation 'add)
            (lambda (x y) (+ x y))
            (lambda (x y) (* x y))))
    "#;

    let result = eval_source(higher_order_lambda, &mut env).unwrap();
    assert!(result.is_procedure());
    let proc = result.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));
}
