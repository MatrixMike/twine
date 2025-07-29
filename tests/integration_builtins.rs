//! Integration tests for builtin procedures
//!
//! This file contains integration tests for builtin procedure functionality:
//! - Builtin procedure lookup and availability
//! - Builtin procedure behavior and correctness
//! - Builtin shadowing by user definitions
//! - Builtins in nested environments
//! - Error handling for builtin procedures
//! - Error handling for builtin operations

mod common;

use common::eval_source;
use twine_scheme::runtime::Environment;
use twine_scheme::types::Value;

#[test]
fn test_integration_builtin_procedure_lookup() {
    let mut env = Environment::new();

    // Test that builtin procedures are automatically available
    let result = eval_source("(+ 1 2 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0));

    let result = eval_source("(* 4 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(20.0));

    let result = eval_source("(- 10 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0));

    let result = eval_source("(/ 15 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0));

    // Test comparison operators
    let result = eval_source("(= 5 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(< 3 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(> 7 2)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(<= 4 4)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(>= 6 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test list operations
    let result = eval_source("(list 1 2 3)", &mut env).unwrap();
    assert!(result.is_list());

    let result = eval_source("(car '(a b c))", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "a");

    let result = eval_source("(cdr '(a b c))", &mut env).unwrap();
    assert!(result.is_list());

    let result = eval_source("(cons 'x '(y z))", &mut env).unwrap();
    assert!(result.is_list());

    let result = eval_source("(null? '())", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(length '(a b c d))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);

    // Test type predicates
    let result = eval_source("(number? 42)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(string? \"hello\")", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(boolean? #t)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(symbol? 'foo)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(list? '(1 2 3))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(procedure? +)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test I/O procedures (these return nil but shouldn't error)
    let result = eval_source("(display \"test\")", &mut env).unwrap();
    assert_eq!(result, Value::Nil);

    let result = eval_source("(newline)", &mut env).unwrap();
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_integration_builtin_arithmetic_comprehensive() {
    let mut env = Environment::new();

    // Test arithmetic with various argument counts
    let result = eval_source("(+)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);

    let result = eval_source("(+ 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    let result = eval_source("(+ 1 2 3 4 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);

    let result = eval_source("(*)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    let result = eval_source("(* 7)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 7.0);

    let result = eval_source("(* 2 3 4 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 120.0);

    // Test subtraction and division
    let result = eval_source("(- 10)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), -10.0);

    let result = eval_source("(- 10 3 2)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    let result = eval_source("(/ 8)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.125);

    let result = eval_source("(/ 24 2 3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);
}

#[test]
fn test_integration_builtin_comparison_comprehensive() {
    let mut env = Environment::new();

    // Test equality with multiple arguments
    let result = eval_source("(= 5 5 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(= 5 5 6)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test ordering with multiple arguments
    let result = eval_source("(< 1 2 3 4)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(< 1 3 2 4)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(> 10 8 6 4)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(<= 1 2 2 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(>= 5 5 4 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
}

#[test]
fn test_integration_builtin_list_operations_comprehensive() {
    let mut env = Environment::new();

    // Test list construction
    let result = eval_source("(list)", &mut env).unwrap();
    assert!(result.is_list());
    assert_eq!(result.as_list().unwrap().len(), 0);

    let result = eval_source("(list 'a 'b 'c)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "a");

    // Test cons operations
    let result = eval_source("(cons 1 '())", &mut env).unwrap();
    assert!(result.is_list());
    assert_eq!(result.as_list().unwrap().len(), 1);

    let result = eval_source("(cons 'a (cons 'b '()))", &mut env).unwrap();
    assert!(result.is_list());
    assert_eq!(result.as_list().unwrap().len(), 2);

    // Test car and cdr
    let result = eval_source("(car (cons 'first '(second)))", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "first");

    let result = eval_source("(cdr (cons 'first '(second)))", &mut env).unwrap();
    assert!(result.is_list());
    assert_eq!(result.as_list().unwrap().len(), 1);

    // Test null?
    let result = eval_source("(null? (cdr (cons 'only '())))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(null? (cons 'not-null '()))", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test length
    let result = eval_source("(length (list 1 2 3 4 5))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    let result = eval_source("(length '())", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);
}

#[test]
fn test_integration_builtin_type_predicates() {
    let mut env = Environment::new();

    // Test number?
    let result = eval_source("(number? 42)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(number? 3.14)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(number? \"not-a-number\")", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test string?
    let result = eval_source("(string? \"hello\")", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(string? \"\")", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(string? 42)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test boolean?
    let result = eval_source("(boolean? #t)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(boolean? #f)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(boolean? 'not-boolean)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test symbol?
    let result = eval_source("(symbol? 'foo)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(symbol? 'bar-baz)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(symbol? \"not-symbol\")", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test list?
    let result = eval_source("(list? '())", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(list? '(a b c))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(list? (list 1 2 3))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(list? 42)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test procedure?
    let result = eval_source("(procedure? +)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(procedure? car)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    eval_source("(define my-lambda (lambda (x) x))", &mut env).unwrap();
    let result = eval_source("(procedure? my-lambda)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(procedure? 42)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());
}

#[test]
fn test_integration_builtin_shadowing() {
    let mut env = Environment::new();

    // Test that builtins work initially
    let result = eval_source("(+ 2 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0));

    // Shadow the builtin with a user definition
    eval_source("(define + 42)", &mut env).unwrap();

    // Now + should refer to the user-defined value, not the builtin
    let result = eval_source("+", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));

    // Arithmetic should no longer work with +
    let result = eval_source("(+ 2 3)", &mut env);
    assert!(result.is_err()); // Should fail because + is now a number, not a procedure

    // Test shadowing with a procedure
    eval_source("(define * (lambda (x y) (- x y)))", &mut env).unwrap();
    let result = eval_source("(* 10 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0)); // Should subtract, not multiply

    // Test that other builtins still work
    let result = eval_source("(- 10 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0));

    let result = eval_source("(/ 15 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0));
}

#[test]
fn test_integration_builtin_in_nested_environments() {
    let mut env = Environment::new();

    // Test builtins work in nested let expressions
    let result = eval_source("(let ((x 5)) (+ x 3))", &mut env).unwrap();
    assert_eq!(result, Value::number(8.0));

    // Test builtins work in lambda closures
    eval_source("(define add-ten (lambda (x) (+ x 10)))", &mut env).unwrap();
    let result = eval_source("(add-ten 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0));

    // Test builtin shadowing in nested scope
    let result = eval_source("(let ((+ (lambda (x y) (* x y)))) (+ 3 4))", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0)); // Should multiply in let scope

    // Outside let, + should still be addition
    let result = eval_source("(+ 3 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0));

    // Test nested lambda with builtin capture
    eval_source(
        "(define make-multiplier (lambda (factor) (lambda (x) (* x factor))))",
        &mut env,
    )
    .unwrap();
    eval_source("(define times-five (make-multiplier 5))", &mut env).unwrap();
    let result = eval_source("(times-five 7)", &mut env).unwrap();
    assert_eq!(result, Value::number(35.0));
}

#[test]
fn test_integration_builtin_error_handling() {
    let mut env = Environment::new();

    // Test arithmetic errors
    let result = eval_source("(+ 1 \"not-number\")", &mut env);
    assert!(result.is_err());

    let result = eval_source("(/ 5 0)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(-)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(/)", &mut env);
    assert!(result.is_err());

    // Test comparison errors
    let result = eval_source("(< 5 \"not-number\")", &mut env);
    assert!(result.is_err());

    let result = eval_source("(=)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(= 5)", &mut env);
    assert!(result.is_err());

    // Test list operation errors
    let result = eval_source("(car '())", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cdr '())", &mut env);
    assert!(result.is_err());

    let result = eval_source("(car 42)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(length \"not-list\")", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cons)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cons 1)", &mut env);
    assert!(result.is_err());

    // Test type predicate errors (wrong arity)
    let result = eval_source("(number?)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(number? 1 2)", &mut env);
    assert!(result.is_err());

    // Test I/O errors
    let result = eval_source("(display)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(display 1 2)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(newline 1)", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_builtin_comprehensive_interaction() {
    let mut env = Environment::new();

    // Test complex interaction between multiple builtins
    let result = eval_source("(+ (* 2 3) (- 10 5) (/ 12 4))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 14.0); // 6 + 5 + 3

    // Test builtins with list operations
    let result = eval_source("(+ (length '(a b c)) (car '(5 10 15)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 8.0); // 3 + 5

    // Test nested builtin calls with type predicates
    let result = eval_source(
        "(if (and (number? 42) (list? '(a b))) (+ 1 2) (* 3 4))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);

    // Test builtin composition with user-defined procedures
    eval_source("(define square (lambda (x) (* x x)))", &mut env).unwrap();
    let result = eval_source("(+ (square 3) (square 4))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 25.0); // 9 + 16

    // Test complex list manipulation
    let result = eval_source("(cons (+ 1 2) (cons (* 3 4) '()))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 3.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 12.0);

    // Test conditional with comparison and list predicates
    let result = eval_source(
        r#"(if (and (> (length '(a b c d)) 2) (null? '()))
             "both-true"
             "something-false")"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "both-true");
}
