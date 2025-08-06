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
use twine_scheme::Error;
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
            .contains("expected 2 arguments, got 0")
    );

    // Test lambda with no body
    let no_body = "(lambda (x))";
    let result = eval_source(no_body, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("expected 2 arguments, got 1")
    );

    // Test lambda with invalid parameter list
    let invalid_params = "(lambda x x)";
    let result = eval_source(invalid_params, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("parameters must be enclosed in parentheses")
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

#[test]
fn test_integration_lambda_application_basic() {
    let mut env = Environment::new();

    // Define a simple lambda and call it
    let lambda_def = "(define add1 (lambda (x) (+ x 1)))";
    eval_source(lambda_def, &mut env).unwrap();

    // Call the lambda
    let call_result = eval_source("(add1 5)", &mut env).unwrap();
    assert_eq!(call_result, Value::number(6.0));

    // Test identity lambda
    eval_source("(define identity (lambda (x) x))", &mut env).unwrap();
    let result = eval_source("(identity 42)", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));

    let result = eval_source("(identity \"hello\")", &mut env).unwrap();
    assert_eq!(result, Value::string("hello"));
}

#[test]
fn test_integration_lambda_application_multiple_parameters() {
    let mut env = Environment::new();

    // Define lambda with multiple parameters
    let lambda_def = "(define add3 (lambda (x y z) (+ x y z)))";
    eval_source(lambda_def, &mut env).unwrap();

    // Call with multiple arguments
    let result = eval_source("(add3 1 2 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0));

    // Test with different values
    let result = eval_source("(add3 10 20 30)", &mut env).unwrap();
    assert_eq!(result, Value::number(60.0));

    // Test lambda with mixed operations
    eval_source(
        "(define complex-op (lambda (a b c) (* (+ a b) c)))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(complex-op 2 3 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(20.0)); // (2 + 3) * 4 = 20
}

#[test]
fn test_integration_lambda_application_no_parameters() {
    let mut env = Environment::new();

    // Define lambda with no parameters
    let lambda_def = "(define get42 (lambda () 42))";
    eval_source(lambda_def, &mut env).unwrap();

    // Call with no arguments
    let result = eval_source("(get42)", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));

    // Test constant string lambda
    eval_source(
        "(define get-greeting (lambda () \"Hello, World!\"))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(get-greeting)", &mut env).unwrap();
    assert_eq!(result, Value::string("Hello, World!"));
}

#[test]
fn test_integration_lambda_application_closure() {
    let mut env = Environment::new();

    // Define outer variable
    eval_source("(define x 10)", &mut env).unwrap();

    // Define lambda that captures x
    let lambda_def = "(define addx (lambda (y) (+ x y)))";
    eval_source(lambda_def, &mut env).unwrap();

    // Call lambda - should use captured x
    let result = eval_source("(addx 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0));

    // Change x and test again - with lexical scoping, lambda should still use captured x (10)
    eval_source("(define x 20)", &mut env).unwrap();
    let result = eval_source("(addx 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0)); // Should use original captured x (10)

    // Test closure with let
    let closure_expr = r#"
        (let ((multiplier 3))
          (define multiply-by-3 (lambda (n) (* n multiplier)))
          (multiply-by-3 7))
    "#;
    let result = eval_source(closure_expr, &mut env).unwrap();
    assert_eq!(result, Value::number(21.0));
}

#[test]
fn test_integration_lambda_application_nested_calls() {
    let mut env = Environment::new();

    // Define multiple lambdas
    eval_source("(define double (lambda (x) (* x 2)))", &mut env).unwrap();
    eval_source("(define add1 (lambda (x) (+ x 1)))", &mut env).unwrap();

    // Nested function calls
    let result = eval_source("(double (add1 3))", &mut env).unwrap();
    assert_eq!(result, Value::number(8.0)); // double(add1(3)) = double(4) = 8

    // Chain multiple operations
    eval_source("(define subtract5 (lambda (x) (- x 5)))", &mut env).unwrap();
    let result = eval_source("(subtract5 (double (add1 10)))", &mut env).unwrap();
    assert_eq!(result, Value::number(17.0)); // subtract5(double(add1(10))) = subtract5(double(11)) = subtract5(22) = 17
}

#[test]
fn test_integration_lambda_application_with_lists() {
    let mut env = Environment::new();

    // Define lambda that works with lists
    let lambda_def = "(define first-plus-second (lambda (lst) (+ (car lst) (car (cdr lst)))))";
    eval_source(lambda_def, &mut env).unwrap();

    // Call with list argument
    let result = eval_source("(first-plus-second (list 3 7 9))", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0));

    // Test lambda that constructs lists
    eval_source("(define make-pair (lambda (x y) (list x y)))", &mut env).unwrap();
    let result = eval_source("(make-pair 'a 'b)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "b");

    // Test lambda with cons operations
    eval_source("(define prepend (lambda (x lst) (cons x lst)))", &mut env).unwrap();
    let result = eval_source("(prepend 'first '(second third))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "first");
}

#[test]
fn test_integration_lambda_application_complex_expression() {
    let mut env = Environment::new();

    // Call lambda directly without defining it first
    let direct_call = "((lambda (x y) (+ (* x x) (* y y))) 3 4)";
    let result = eval_source(direct_call, &mut env).unwrap();
    assert_eq!(result, Value::number(25.0)); // 3² + 4² = 9 + 16 = 25

    // Complex expression with conditional
    let conditional_call = "((lambda (x) (if (> x 0) (* x 2) (- x))) -5)";
    let result = eval_source(conditional_call, &mut env).unwrap();
    assert_eq!(result, Value::number(5.0)); // -(-5) = 5

    // Lambda with let binding
    let let_call = "((lambda (x) (let ((doubled (* x 2))) (+ doubled 1))) 10)";
    let result = eval_source(let_call, &mut env).unwrap();
    assert_eq!(result, Value::number(21.0)); // (10 * 2) + 1 = 21
}

#[test]
fn test_integration_lambda_application_arity_errors() {
    let mut env = Environment::new();

    // Define lambda expecting 2 parameters
    eval_source("(define add2 (lambda (x y) (+ x y)))", &mut env).unwrap();

    // Test too few arguments
    let result = eval_source("(add2 5)", &mut env);
    assert!(result.is_err());
    if let Err(Error::ArityError {
        expected, actual, ..
    }) = result
    {
        assert_eq!(expected, 2);
        assert_eq!(actual, 1);
    } else {
        panic!("Expected ArityError for too few arguments");
    }

    // Test too many arguments
    let result = eval_source("(add2 1 2 3)", &mut env);
    assert!(result.is_err());
    if let Err(Error::ArityError {
        expected, actual, ..
    }) = result
    {
        assert_eq!(expected, 2);
        assert_eq!(actual, 3);
    } else {
        panic!("Expected ArityError for too many arguments");
    }

    // Test zero-arity lambda with arguments
    eval_source("(define get-constant (lambda () 42))", &mut env).unwrap();
    let result = eval_source("(get-constant 1)", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_lambda_application_error_cases() {
    let mut env = Environment::new();

    // Test calling non-procedure
    eval_source("(define x 42)", &mut env).unwrap();
    let result = eval_source("(x 1 2 3)", &mut env);
    assert!(result.is_err());
    if let Err(Error::RuntimeError(msg)) = result {
        assert!(msg.contains("is not a procedure, got number"));
    } else {
        panic!("Expected RuntimeError for calling non-procedure");
    }

    // Test lambda with unbound identifier in body
    eval_source(
        "(define bad-lambda (lambda (x) (+ x undefined-var)))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(bad-lambda 5)", &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unbound identifier")
    );

    // Test lambda with type error in body
    eval_source(
        "(define type-error-lambda (lambda (x) (+ x \"not-number\")))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(type-error-lambda 5)", &mut env);
    assert!(result.is_err());

    // Test lambda application with expression that fails evaluation
    let result = eval_source("((lambda (x) x) (+ 1 \"not-number\"))", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_lambda_application_parameter_shadowing() {
    let mut env = Environment::new();

    // Define outer variable
    eval_source("(define x 100)", &mut env).unwrap();

    // Define lambda that shadows x
    let lambda_def = "(define test-shadow (lambda (x) (+ x 1)))";
    eval_source(lambda_def, &mut env).unwrap();

    // Call lambda - parameter x should shadow outer x
    let result = eval_source("(test-shadow 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0));

    // Outer x should still be accessible outside lambda
    let result = eval_source("x", &mut env).unwrap();
    assert_eq!(result, Value::number(100.0));

    // Test nested shadowing
    let nested_lambda = r#"
        (lambda (x)
          (let ((x 999))
            (lambda (y) (+ x y))))
    "#;
    eval_source(&format!("(define make-nested {nested_lambda})"), &mut env).unwrap();
    let result = eval_source("((make-nested 1) 2)", &mut env).unwrap();
    assert_eq!(result, Value::number(1001.0)); // 999 + 2, using let-bound x
}

#[test]
fn test_integration_lambda_application_recursive_pattern() {
    let mut env = Environment::new();

    // Define a simple recursive-style pattern (actual recursion needs more setup)
    eval_source("(define make-counter (lambda (n) (lambda () n)))", &mut env).unwrap();

    // Create a counter
    eval_source("(define counter5 (make-counter 5))", &mut env).unwrap();

    // Call the counter
    let result = eval_source("(counter5)", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0));

    // Create another counter
    eval_source("(define counter10 (make-counter 10))", &mut env).unwrap();
    let result = eval_source("(counter10)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0));

    // Test higher-order lambda pattern
    eval_source(
        "(define make-adder (lambda (n) (lambda (x) (+ n x))))",
        &mut env,
    )
    .unwrap();
    eval_source("(define add5 (make-adder 5))", &mut env).unwrap();
    let result = eval_source("(add5 10)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0));
}

#[test]
fn test_integration_lambda_application_comprehensive() {
    let mut env = Environment::new();

    // Complex test combining multiple features
    eval_source("(define x 10)", &mut env).unwrap();
    eval_source(
        "(define make-adder (lambda (n) (lambda (m) (+ n m x))))",
        &mut env,
    )
    .unwrap();
    eval_source("(define add5 (make-adder 5))", &mut env).unwrap();

    let result = eval_source("(add5 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(18.0)); // 5 + 3 + 10

    // Test lambda with complex conditional and list operations
    let complex_lambda = r#"
        (lambda (data)
          (if (null? data)
            '()
            (let ((first (car data))
                  (rest (cdr data)))
              (if (> first 10)
                (cons (* first 2) rest)
                (cons (+ first 5) rest)))))
    "#;

    eval_source(
        &format!("(define process-data {complex_lambda})"),
        &mut env,
    )
    .unwrap();

    // Test with data where first element > 10
    let result = eval_source("(process-data '(15 20 30))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 30.0); // 15 * 2

    // Test with data where first element <= 10
    let result = eval_source("(process-data '(5 20 30))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 10.0); // 5 + 5

    // Test with empty list
    let result = eval_source("(process-data '())", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 0);
}

#[test]
fn test_integration_lambda_application_with_conditionals() {
    let mut env = Environment::new();

    // Define conditional lambda
    eval_source("(define max2 (lambda (a b) (if (> a b) a b)))", &mut env).unwrap();

    let result = eval_source("(max2 10 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0));

    let result = eval_source("(max2 3 8)", &mut env).unwrap();
    assert_eq!(result, Value::number(8.0));

    // Test lambda that returns different types
    eval_source(
        "(define classify (lambda (x) (if (> x 0) 'positive (if (< x 0) 'negative 'zero))))",
        &mut env,
    )
    .unwrap();

    let result = eval_source("(classify 5)", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "positive");

    let result = eval_source("(classify -3)", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "negative");

    let result = eval_source("(classify 0)", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "zero");
}

#[test]
fn test_integration_lambda_application_multiple_body_expressions() {
    let mut env = Environment::new();

    // Test lambda with multiple body expressions - should return last one
    eval_source(
        r#"(define test-multi
             (lambda (x)
               (+ 0 0)
               (+ x 1)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(test-multi 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0)); // Should return result of (+ x 1)

    // Test with side effects (using define in body)
    eval_source(
        r#"(define test-side-effects
             (lambda (x)
               (define temp (* x 2))
               (define result (+ temp 1))
               result))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(test-side-effects 10)", &mut env).unwrap();
    assert_eq!(result, Value::number(21.0)); // (10 * 2) + 1

    // The local definitions should not be accessible outside
    let result = eval_source("temp", &mut env);
    assert!(result.is_err());

    let result = eval_source("result", &mut env);
    assert!(result.is_err());
}

// ================================================================================================
// RECURSION TESTS - Function System Component
// ================================================================================================

#[test]
fn test_integration_lambda_recursion_simple_factorial() {
    let mut env = Environment::new();

    // Define recursive factorial using define + lambda
    eval_source(
        r#"(define factorial
             (lambda (n)
               (if (= n 0)
                   1
                   (* n (factorial (- n 1))))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(factorial 0)", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0));

    let result = eval_source("(factorial 1)", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0));

    let result = eval_source("(factorial 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(120.0)); // 5! = 120

    let result = eval_source("(factorial 6)", &mut env).unwrap();
    assert_eq!(result, Value::number(720.0)); // 6! = 720
}

#[test]
fn test_integration_lambda_recursion_fibonacci() {
    let mut env = Environment::new();

    // Define recursive fibonacci using define + lambda
    eval_source(
        r#"(define fib
             (lambda (n)
               (if (< n 2)
                   n
                   (+ (fib (- n 1)) (fib (- n 2))))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(fib 0)", &mut env).unwrap();
    assert_eq!(result, Value::number(0.0));

    let result = eval_source("(fib 1)", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0));

    let result = eval_source("(fib 2)", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0));

    let result = eval_source("(fib 7)", &mut env).unwrap();
    assert_eq!(result, Value::number(13.0)); // fib(7) = 13

    let result = eval_source("(fib 10)", &mut env).unwrap();
    assert_eq!(result, Value::number(55.0)); // fib(10) = 55
}

#[test]
fn test_integration_lambda_recursion_countdown() {
    let mut env = Environment::new();

    // Define recursive countdown function
    eval_source(
        r#"(define countdown
             (lambda (n)
               (if (= n 0)
                   'done
                   (countdown (- n 1)))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(countdown 0)", &mut env).unwrap();
    assert_eq!(result, Value::symbol("done"));

    let result = eval_source("(countdown 1)", &mut env).unwrap();
    assert_eq!(result, Value::symbol("done"));

    let result = eval_source("(countdown 5)", &mut env).unwrap();
    assert_eq!(result, Value::symbol("done"));
}

#[test]
fn test_integration_lambda_recursion_list_length() {
    let mut env = Environment::new();

    // Define recursive list length function
    eval_source(
        r#"(define list-length
             (lambda (lst)
               (if (null? lst)
                   0
                   (+ 1 (list-length (cdr lst))))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(list-length '())", &mut env).unwrap();
    assert_eq!(result, Value::number(0.0));

    let result = eval_source("(list-length '(1))", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0));

    let result = eval_source("(list-length '(1 2 3 4 5))", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0));

    let result = eval_source("(list-length '(a b c))", &mut env).unwrap();
    assert_eq!(result, Value::number(3.0));
}

#[test]
fn test_integration_lambda_recursion_sum_list() {
    let mut env = Environment::new();

    // Define recursive sum function
    eval_source(
        r#"(define sum-list
             (lambda (lst)
               (if (null? lst)
                   0
                   (+ (car lst) (sum-list (cdr lst))))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(sum-list '())", &mut env).unwrap();
    assert_eq!(result, Value::number(0.0));

    let result = eval_source("(sum-list '(1))", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0));

    let result = eval_source("(sum-list '(1 2 3 4))", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0));

    let result = eval_source("(sum-list '(5 10 15))", &mut env).unwrap();
    assert_eq!(result, Value::number(30.0));
}

#[test]
fn test_integration_lambda_recursion_mutual() {
    let mut env = Environment::new();

    // Define mutually recursive even/odd functions using letrec
    eval_source(
        r#"(define test-mutual
             (letrec ((even? (lambda (n)
                               (if (= n 0)
                                   #t
                                   (odd? (- n 1)))))
                      (odd? (lambda (n)
                              (if (= n 0)
                                  #f
                                  (even? (- n 1))))))
               (lambda (n is-even?)
                 (if is-even?
                     (even? n)
                     (odd? n)))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(test-mutual 0 #t)", &mut env).unwrap();
    assert_eq!(result, Value::boolean(true)); // 0 is even

    let result = eval_source("(test-mutual 0 #f)", &mut env).unwrap();
    assert_eq!(result, Value::boolean(false)); // 0 is not odd

    let result = eval_source("(test-mutual 4 #t)", &mut env).unwrap();
    assert_eq!(result, Value::boolean(true)); // 4 is even

    let result = eval_source("(test-mutual 5 #f)", &mut env).unwrap();
    assert_eq!(result, Value::boolean(true)); // 5 is odd

    let result = eval_source("(test-mutual 7 #t)", &mut env).unwrap();
    assert_eq!(result, Value::boolean(false)); // 7 is not even
}

#[test]
fn test_integration_lambda_recursion_with_accumulator() {
    let mut env = Environment::new();

    // Define tail-recursive factorial with accumulator
    eval_source(
        r#"(define factorial-tail
             (lambda (n)
               (define factorial-helper
                 (lambda (n acc)
                   (if (= n 0)
                       acc
                       (factorial-helper (- n 1) (* n acc)))))
               (factorial-helper n 1)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(factorial-tail 0)", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0));

    let result = eval_source("(factorial-tail 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(120.0));

    let result = eval_source("(factorial-tail 6)", &mut env).unwrap();
    assert_eq!(result, Value::number(720.0));
}

// ================================================================================================
// TAIL CALL OPTIMIZATION TESTS - Function System Component
// ================================================================================================

#[test]
fn test_integration_lambda_tail_calls_basic() {
    let mut env = Environment::new();

    // Test basic tail call between different functions
    eval_source(r#"(define add-one (lambda (x) (+ x 1)))"#, &mut env).unwrap();

    eval_source(
        r#"(define tail-call-add-one
             (lambda (x)
               (add-one x)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(tail-call-add-one 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0));

    // Test tail call with arithmetic operation
    eval_source(r#"(define double (lambda (x) (* x 2)))"#, &mut env).unwrap();

    eval_source(
        r#"(define tail-call-double
             (lambda (x)
               (double (+ x 1))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(tail-call-double 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0)); // double(4 + 1) = double(5) = 10
}

#[test]
fn test_integration_lambda_tail_calls_conditional() {
    let mut env = Environment::new();

    eval_source(r#"(define add-one (lambda (x) (+ x 1)))"#, &mut env).unwrap();
    eval_source(r#"(define double (lambda (x) (* x 2)))"#, &mut env).unwrap();

    // Test tail call in conditional branches
    eval_source(
        r#"(define conditional-tail-call
             (lambda (x)
               (if (> x 10)
                 (add-one x)
                 (double x))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(conditional-tail-call 15)", &mut env).unwrap();
    assert_eq!(result, Value::number(16.0)); // add-one(15) = 16

    let result = eval_source("(conditional-tail-call 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0)); // double(5) = 10

    // Test nested conditionals with tail calls
    eval_source(
        r#"(define nested-conditional-tail
             (lambda (x)
               (if (< x 0)
                 (if (< x -10)
                   (add-one (- x))
                   (double (- x)))
                 (if (> x 10)
                   (double x)
                   (add-one x)))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(nested-conditional-tail -15)", &mut env).unwrap();
    assert_eq!(result, Value::number(16.0)); // add-one(-(-15)) = add-one(15) = 16

    let result = eval_source("(nested-conditional-tail -5)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0)); // double(-(-5)) = double(5) = 10

    let result = eval_source("(nested-conditional-tail 15)", &mut env).unwrap();
    assert_eq!(result, Value::number(30.0)); // double(15) = 30

    let result = eval_source("(nested-conditional-tail 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0)); // add-one(5) = 6
}

#[test]
fn test_integration_lambda_tail_calls_with_let() {
    let mut env = Environment::new();

    eval_source(r#"(define triple (lambda (x) (* x 3)))"#, &mut env).unwrap();

    // Test tail call in let expression
    eval_source(
        r#"(define let-tail-call
             (lambda (x)
               (let ((temp (* x 2)))
                 (triple temp))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(let-tail-call 7)", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0)); // triple(7 * 2) = triple(14) = 42

    // Test tail call in nested let
    eval_source(
        r#"(define nested-let-tail
             (lambda (x)
               (let ((a (+ x 1)))
                 (let ((b (* a 2)))
                   (triple b)))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(nested-let-tail 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(30.0)); // triple(2 * (4 + 1)) = triple(10) = 30
}

#[test]
fn test_integration_lambda_tail_calls_recursive() {
    let mut env = Environment::new();

    // Test tail-recursive countdown (proper tail recursion)
    eval_source(
        r#"(define tail-countdown
             (lambda (n)
               (if (= n 0)
                   'done
                   (tail-countdown (- n 1)))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(tail-countdown 5)", &mut env).unwrap();
    assert_eq!(result, Value::symbol("done"));

    let result = eval_source("(tail-countdown 100)", &mut env).unwrap();
    assert_eq!(result, Value::symbol("done"));

    // Test tail-recursive sum with accumulator
    eval_source(
        r#"(define tail-sum
             (lambda (n acc)
               (if (= n 0)
                   acc
                   (tail-sum (- n 1) (+ acc n)))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(tail-sum 5 0)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0)); // 1+2+3+4+5 = 15

    let result = eval_source("(tail-sum 10 0)", &mut env).unwrap();
    assert_eq!(result, Value::number(55.0)); // sum of 1 to 10 = 55
}

#[test]
fn test_integration_lambda_tail_calls_multiple_expressions() {
    let mut env = Environment::new();

    eval_source(r#"(define helper (lambda (x) (+ x 1)))"#, &mut env).unwrap();

    // Test that only the last expression in lambda body is tail-call optimized
    eval_source(
        r#"(define test-tco-multi
             (lambda (x)
               (+ x 0)
               (+ x 0)
               (helper x)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(test-tco-multi 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0)); // helper(5) = 6

    // Test tail call in conditional within multiple expressions
    eval_source(
        r#"(define complex-tco
             (lambda (x)
               (* x 0)
               (if (> x 5)
                 (helper (* x 2))
                 (* x 3))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(complex-tco 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(9.0)); // 3 * 3 = 9

    let result = eval_source("(complex-tco 7)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0)); // helper(7 * 2) = helper(14) = 15
}

#[test]
fn test_integration_lambda_tail_calls_procedure_syntax() {
    let mut env = Environment::new();

    eval_source(r#"(define add-two (lambda (x) (+ x 2)))"#, &mut env).unwrap();

    // Test tail call with define procedure syntax
    eval_source(
        r#"(define (procedure-tail-call x)
             (if (< x 0)
               (add-two (- x))
               (add-two x)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(procedure-tail-call -3)", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0)); // add-two(-(-3)) = add-two(3) = 5

    let result = eval_source("(procedure-tail-call 6)", &mut env).unwrap();
    assert_eq!(result, Value::number(8.0)); // add-two(6) = 8

    // Test tail call in define procedure with multiple body expressions
    eval_source(
        r#"(define (procedure-multi-tco x)
             (+ x 0)
             (- x 0)
             (if (> x 0)
               (add-two x)
               (add-two (- x))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(procedure-multi-tco 10)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0)); // add-two(10) = 12

    let result = eval_source("(procedure-multi-tco -8)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0)); // add-two(-(-8)) = add-two(8) = 10
}

// ================================================================================================
// COMPREHENSIVE FUNCTION SYSTEM INTEGRATION TESTS
// ================================================================================================

#[test]
fn test_integration_lambda_comprehensive_function_pipeline() {
    let mut env = Environment::new();

    // Test complete function system: creation, application, recursion, and tail calls
    eval_source(
        r#"(define create-sum-processor
             (lambda ()
               (lambda (lst)
                 (define sum-helper
                   (lambda (items acc)
                     (if (null? items)
                         acc
                         (sum-helper
                           (cdr items)
                           (+ (car items) acc)))))
                 (sum-helper lst 0))))"#,
        &mut env,
    )
    .unwrap();

    eval_source(
        r#"(define create-product-processor
             (lambda ()
               (lambda (lst)
                 (if (null? lst)
                     1
                     (let ()
                       (define product-helper
                         (lambda (items acc)
                           (if (null? items)
                               acc
                               (product-helper
                                 (cdr items)
                                 (* (car items) acc)))))
                       (product-helper lst 1))))))"#,
        &mut env,
    )
    .unwrap();

    // Create processors
    eval_source("(define sum-processor (create-sum-processor))", &mut env).unwrap();
    eval_source(
        "(define product-processor (create-product-processor))",
        &mut env,
    )
    .unwrap();

    // Test the processors
    let result = eval_source("(sum-processor '(1 2 3 4 5))", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0)); // 1+2+3+4+5 = 15

    let result = eval_source("(product-processor '(1 2 3 4))", &mut env).unwrap();
    assert_eq!(result, Value::number(24.0)); // 1*2*3*4 = 24

    let result = eval_source("(sum-processor '())", &mut env).unwrap();
    assert_eq!(result, Value::number(0.0));

    let result = eval_source("(product-processor '())", &mut env).unwrap();
    assert_eq!(result, Value::number(1.0)); // Empty product should be 1 (multiplicative identity)
}

#[test]
fn test_integration_lambda_higher_order_recursive_functions() {
    let mut env = Environment::new();

    // Define map function using recursion and higher-order capabilities
    eval_source(
        r#"(define map-func
             (lambda (f lst)
               (if (null? lst)
                   '()
                   (cons (f (car lst))
                         (map-func f (cdr lst))))))"#,
        &mut env,
    )
    .unwrap();

    // Define some functions to map
    eval_source("(define square (lambda (x) (* x x)))", &mut env).unwrap();
    eval_source("(define increment (lambda (x) (+ x 1)))", &mut env).unwrap();

    // Test mapping square function
    let result = eval_source("(map-func square '(1 2 3 4))", &mut env).unwrap();
    let expected = Value::list(vec![
        Value::number(1.0),
        Value::number(4.0),
        Value::number(9.0),
        Value::number(16.0),
    ]);
    assert_eq!(result, expected);

    // Test mapping increment function
    let result = eval_source("(map-func increment '(10 20 30))", &mut env).unwrap();
    let expected = Value::list(vec![
        Value::number(11.0),
        Value::number(21.0),
        Value::number(31.0),
    ]);
    assert_eq!(result, expected);

    // Test with empty list
    let result = eval_source("(map-func square '())", &mut env).unwrap();
    assert_eq!(result, Value::empty_list());
}

#[test]
fn test_integration_lambda_function_composition_and_currying() {
    let mut env = Environment::new();

    // Define function composition
    eval_source(
        r#"(define compose
             (lambda (f g)
               (lambda (x)
                 (f (g x)))))"#,
        &mut env,
    )
    .unwrap();

    // Define some basic functions
    eval_source("(define add-one (lambda (x) (+ x 1)))", &mut env).unwrap();
    eval_source("(define double (lambda (x) (* x 2)))", &mut env).unwrap();
    eval_source("(define square (lambda (x) (* x x)))", &mut env).unwrap();

    // Create composed functions
    eval_source(
        "(define add-one-then-double (compose double add-one))",
        &mut env,
    )
    .unwrap();
    eval_source(
        "(define double-then-square (compose square double))",
        &mut env,
    )
    .unwrap();

    // Test compositions
    let result = eval_source("(add-one-then-double 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0)); // double(add-one(5)) = double(6) = 12

    let result = eval_source("(double-then-square 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(36.0)); // square(double(3)) = square(6) = 36

    // Define currying function
    eval_source(
        r#"(define curry-add
             (lambda (x)
               (lambda (y)
                 (+ x y))))"#,
        &mut env,
    )
    .unwrap();

    // Test curried function
    eval_source("(define add-five (curry-add 5))", &mut env).unwrap();
    let result = eval_source("(add-five 10)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0));

    eval_source("(define add-hundred (curry-add 100))", &mut env).unwrap();
    let result = eval_source("(add-hundred 23)", &mut env).unwrap();
    assert_eq!(result, Value::number(123.0));
}
