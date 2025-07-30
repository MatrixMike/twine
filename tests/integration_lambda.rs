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
    eval_source(&format!("(define make-nested {})", nested_lambda), &mut env).unwrap();
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
        &format!("(define process-data {})", complex_lambda),
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
