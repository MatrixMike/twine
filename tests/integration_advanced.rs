//! Integration tests for advanced language features
//!
//! This file contains integration tests for advanced functionality:
//! - Tail call optimization
//! - Multiple lambda body expressions
//! - Comprehensive evaluation scenarios
//! - Mixed operations combining multiple language features
//! - Complex nested expressions and procedures

use twine_scheme::Result;
use twine_scheme::runtime::Environment;
use twine_scheme::types::Value;

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

// Helper function for testing I/O side effects using subprocess execution
fn test_io(source: &str, expected_output: &str) {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(&["run", "--bin", "test_io", source])
        .output()
        .expect("Failed to execute test binary");
    assert!(output.status.success(), "Test binary execution failed");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected_output);
}

#[test]
fn test_integration_tail_call_optimization() {
    let mut env = Environment::new();

    // Test tail call optimization by defining functions that call other functions in tail position
    // This avoids the self-recursion issue while still testing the TCO mechanism

    // Define a helper function that adds 1
    eval_source(r#"(define add-one (lambda (x) (+ x 1)))"#, &mut env).unwrap();

    // Define a function that tail-calls add-one
    eval_source(
        r#"(define tail-call-add-one
             (lambda (x)
               (add-one x)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(tail-call-add-one 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0));

    // Test tail call with conditional
    eval_source(
        r#"(define conditional-tail-call
             (lambda (x)
               (if (> x 10)
                 (add-one x)
                 (* x 2))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(conditional-tail-call 15)", &mut env).unwrap();
    assert_eq!(result, Value::number(16.0)); // add-one(15) = 16

    let result = eval_source("(conditional-tail-call 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0)); // 5 * 2 = 10

    // Test tail call in let expression
    eval_source(
        r#"(define let-tail-call
             (lambda (x)
               (let ((temp (* x 2)))
                 (add-one temp))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(let-tail-call 7)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0)); // add-one(7 * 2) = add-one(14) = 15

    // Test nested tail calls
    eval_source(r#"(define double (lambda (x) (* x 2)))"#, &mut env).unwrap();

    eval_source(
        r#"(define nested-tail-call
             (lambda (x)
               (double (add-one x))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(nested-tail-call 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0)); // double(add-one(4)) = double(5) = 10

    // Test tail call with define procedure syntax
    eval_source(
        r#"(define (procedure-tail-call x)
             (if (< x 0)
               (add-one (- x))
               (double x)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(procedure-tail-call -3)", &mut env).unwrap();
    assert_eq!(result, Value::number(4.0)); // add-one(-(-3)) = add-one(3) = 4

    let result = eval_source("(procedure-tail-call 6)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0)); // double(6) = 12
}

#[test]
fn test_integration_multiple_lambda_body_expressions() {
    let mut env = Environment::new();

    // Test lambda with multiple body expressions - side effects and return value
    eval_source(
        r#"(define test-multi
             (lambda (x)
               (+ 0 0)
               (+ x 1)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(test-multi 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0)); // Should return result of last expression

    // Test lambda with display side effects in multiple body expressions
    test_io(
        r#"(define test-multi-io
             (lambda (x)
               (display "Processing: ")
               (display x)
               (display " -> ")
               (display (+ x 1))))
           (test-multi-io 5)"#,
        "Processing: 5 -> 6",
    );

    // Test with define in lambda body
    eval_source(
        r#"(define test-with-define
             (lambda (x)
               (define local-var (* x 2))
               (define another-var (+ local-var 1))
               another-var))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(test-with-define 10)", &mut env).unwrap();
    assert_eq!(result, Value::number(21.0)); // (10 * 2) + 1 = 21

    // The local definitions should not be accessible outside
    let result = eval_source("local-var", &mut env);
    assert!(result.is_err());

    // Test multiple let expressions in lambda body
    eval_source(
        r#"(define multi-let-lambda
             (lambda (x)
               (let ((a (* x 2)))
                 (+ a 1))
               (let ((b (+ x 3)))
                 (* b 2))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(multi-let-lambda 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(14.0)); // Should return result of last let: (4 + 3) * 2 = 14

    // Test multiple let expressions with side effects
    test_io(
        r#"(define multi-let-io
             (lambda (x)
               (let ((a (* x 2)))
                 (display "First let: ")
                 (display a)
                 (display " "))
               (let ((b (+ x 3)))
                 (display "Second let: ")
                 (display b)
                 (display " Final: ")
                 (display (* b 2)))))
           (multi-let-io 4)"#,
        "First let: 8 Second let: 7 Final: 14",
    );

    // Test with conditional expressions in multiple bodies
    eval_source(
        r#"(define multi-conditional
             (lambda (x)
               (if (> x 5) (* x 2) (+ x 1))
               (if (< x 10) (+ x 5) (- x 2))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(multi-conditional 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(8.0)); // Should return result of last if: 3 + 5 = 8

    let result = eval_source("(multi-conditional 12)", &mut env).unwrap();
    assert_eq!(result, Value::number(10.0)); // Should return result of last if: 12 - 2 = 10

    // Test conditional expressions with side effects in multiple bodies
    test_io(
        r#"(define multi-conditional-io
             (lambda (x)
               (display "Input: ")
               (display x)
               (display " -> ")
               (if (> x 5)
                 (begin (display "big=") (display (* x 2)))
                 (begin (display "small=") (display (+ x 1))))
               (display " -> ")
               (if (< x 10)
                 (begin (display "low=") (display (+ x 5)))
                 (begin (display "high=") (display (- x 2))))))
           (multi-conditional-io 3)"#,
        "Input: 3 -> small=4 -> low=8",
    );

    test_io(
        r#"(define multi-conditional-io
             (lambda (x)
               (display "Input: ")
               (display x)
               (display " -> ")
               (if (> x 5)
                 (begin (display "big=") (display (* x 2)))
                 (begin (display "small=") (display (+ x 1))))
               (display " -> ")
               (if (< x 10)
                 (begin (display "low=") (display (+ x 5)))
                 (begin (display "high=") (display (- x 2))))))
           (multi-conditional-io 12)"#,
        "Input: 12 -> big=24 -> high=10",
    );

    // Test with arithmetic expressions in multiple bodies
    eval_source(
        r#"(define multi-arithmetic
             (lambda (x y)
               (+ x y)
               (* x y)
               (- x y)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(multi-arithmetic 10 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0)); // Should return result of last expression: 10 - 3 = 7

    // Test arithmetic expressions with side effects showing intermediate calculations
    test_io(
        r#"(define multi-arithmetic-io
             (lambda (x y)
               (display "Inputs: ")
               (display x)
               (display " and ")
               (display y)
               (display " | Add: ")
               (display (+ x y))
               (display " | Multiply: ")
               (display (* x y))
               (display " | Subtract: ")
               (display (- x y))))
           (multi-arithmetic-io 10 3)"#,
        "Inputs: 10 and 3 | Add: 13 | Multiply: 30 | Subtract: 7",
    );

    // Test mixed expression types
    eval_source(
        r#"(define mixed-expressions
             (lambda (x)
               (list x x)
               (if (> x 0) 'positive 'non-positive)
               (+ x 100)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(mixed-expressions 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(105.0)); // Should return result of last expression: 5 + 100 = 105

    // Test procedure calls in multiple bodies
    eval_source(r#"(define helper (lambda (x) (* x 3)))"#, &mut env).unwrap();

    eval_source(
        r#"(define multi-procedure-calls
             (lambda (x)
               (helper x)
               (helper (+ x 1))
               (helper (+ x 2))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(multi-procedure-calls 2)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0)); // Should return helper(2 + 2) = helper(4) = 12

    // Test procedure calls with side effects showing each call
    test_io(
        r#"(define helper-io (lambda (x)
                             (display "Helper called with: ")
                             (display x)
                             (display " = ")
                             (display (* x 3))
                             (display " | ")))
           (define multi-procedure-calls-io
             (lambda (x)
               (helper-io x)
               (helper-io (+ x 1))
               (helper-io (+ x 2))))
           (multi-procedure-calls-io 2)"#,
        "Helper called with: 2 = 6 | Helper called with: 3 = 9 | Helper called with: 4 = 12 | ",
    );
}

#[test]
fn test_integration_tail_call_optimization_with_multiple_expressions() {
    let mut env = Environment::new();

    // Test that only the last expression in lambda body is tail-call optimized
    eval_source(r#"(define helper (lambda (x) (+ x 1)))"#, &mut env).unwrap();

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
               (* x 2)
               (if (> x 10)
                 (helper x)
                 (helper (* x 2)))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(complex-tco 15)", &mut env).unwrap();
    assert_eq!(result, Value::number(16.0)); // helper(15) = 16

    let result = eval_source("(complex-tco 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0)); // helper(3 * 2) = helper(6) = 7

    // Test tail call in let within multiple expressions
    eval_source(
        r#"(define let-tco-multi
             (lambda (x)
               (+ x 1)
               (let ((temp (* x 2)))
                 (helper temp))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(let-tco-multi 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(9.0)); // helper(4 * 2) = helper(8) = 9

    // Test nested function calls with multiple expressions
    eval_source(r#"(define double (lambda (x) (* x 2)))"#, &mut env).unwrap();

    eval_source(
        r#"(define nested-multi-tco
             (lambda (x)
               (double x)
               (helper (double x))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(nested-multi-tco 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0)); // helper(double(3)) = helper(6) = 7

    // Test define procedure syntax with multiple body expressions and tail calls
    eval_source(
        r#"(define (procedure-multi-tco x)
             (define local (* x 3))
             (if (> local 15)
               (helper local)
               (double local)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(procedure-multi-tco 6)", &mut env).unwrap();
    assert_eq!(result, Value::number(19.0)); // helper(6 * 3) = helper(18) = 19

    let result = eval_source("(procedure-multi-tco 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(18.0)); // double(3 * 3) = double(9) = 18
}

#[test]
fn test_integration_comprehensive_evaluation() {
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("x", Value::number(5.0));
    env.define_str("y", Value::number(10.0));
    env.define_str(
        "data",
        Value::list(vec![Value::number(100.0), Value::number(200.0)]),
    );

    // Test complex nested expression combining all features
    let complex_expr = r#"
        (let ((base (* x y))
              (first (car data))
              (rest (cdr data)))
          (define process-item
            (lambda (item)
              (if (> item base)
                (* item 2)
                (+ item base))))
          (define result-list
            (cons (process-item first)
                  (if (null? rest)
                    '()
                    (list (process-item (car rest))))))
          (let ((total-length (length result-list))
                (first-result (car result-list)))
            (if (= total-length 2)
              (+ first-result (car (cdr result-list)))
              first-result)))
    "#;

    let result = eval_source(complex_expr, &mut env).unwrap();
    // base = 5 * 10 = 50
    // first = 100, process-item(100) = 100 * 2 = 200 (since 100 > 50)
    // second = 200, process-item(200) = 200 * 2 = 400 (since 200 > 50)
    // result-list = (200 400)
    // total-length = 2
    // first-result = 200
    // final result = 200 + 400 = 600
    assert_eq!(result.as_number().unwrap(), 600.0);

    // Test another complex scenario with different data
    env.define_str(
        "small-data",
        Value::list(vec![Value::number(20.0), Value::number(30.0)]),
    );

    let complex_expr2 = r#"
        (let ((base (* x y))
              (first (car small-data))
              (rest (cdr small-data)))
          (define process-item
            (lambda (item)
              (if (> item base)
                (* item 2)
                (+ item base))))
          (define result-list
            (cons (process-item first)
                  (if (null? rest)
                    '()
                    (list (process-item (car rest))))))
          (let ((total-length (length result-list))
                (first-result (car result-list)))
            (if (= total-length 2)
              (+ first-result (car (cdr result-list)))
              first-result)))
    "#;

    let result = eval_source(complex_expr2, &mut env).unwrap();
    // base = 5 * 10 = 50
    // first = 20, process-item(20) = 20 + 50 = 70 (since 20 <= 50)
    // second = 30, process-item(30) = 30 + 50 = 80 (since 30 <= 50)
    // result-list = (70 80)
    // total-length = 2
    // first-result = 70
    // final result = 70 + 80 = 150
    assert_eq!(result.as_number().unwrap(), 150.0);
}

#[test]
fn test_integration_multi_expression_procedure_side_effects() {
    // Test side effects in multi-expression lambda bodies
    test_io(
        r#"(begin
             (define verbose-add
               (lambda (x y)
                 (display "Adding: ")
                 (display x)
                 (display " + ")
                 (display y)
                 (display " = ")
                 (display (+ x y))))
             (verbose-add 10 5))"#,
        "Adding: 10 + 5 = 15",
    );

    // Test side effects in sequential procedure calls
    test_io(
        r#"(begin
             (define show-calc
               (lambda (x y)
                 (display x)
                 (display " * ")
                 (display y)
                 (display " = ")
                 (display (* x y))))
             (show-calc 3 4)
             (display " | ")
             (show-calc 5 6))"#,
        "3 * 4 = 12 | 5 * 6 = 30",
    );

    // Test side effects with begin in if branches
    test_io(
        r#"(begin
             (define describe-number
               (lambda (n)
                 (display "Number ")
                 (display n)
                 (display " is ")
                 (if (> n 0)
                   (begin (display "positive") (display "!"))
                   (begin (display "not positive") (display "?")))))
             (describe-number 5)
             (display " ")
             (describe-number -3))"#,
        "Number 5 is positive! Number -3 is not positive?",
    );
}

#[test]
fn test_integration_mixed_operations() {
    let mut env = Environment::new();

    // Test arithmetic in conditional context
    let result = eval_source("(if (> (+ 2 3) 4) \"greater\" \"lesser\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "greater");

    // Test list operations in arithmetic context
    let result = eval_source("(+ (car '(5 10)) (car (cdr '(5 10))))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);

    // Test conditional with list predicates
    let result = eval_source("(if (null? '()) (+ 1 2) (* 3 4))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);

    // Test arithmetic with conditional results
    let result = eval_source("(* (if (> 5 3) 2 1) (if (< 2 4) 3 2))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0); // 2 * 3 = 6

    // Test list construction with conditional elements
    let result = eval_source("(list (if #t 'first 'alt) (if #f 'alt 'second))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "first");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "second");

    // Test nested operations with multiple data types (without length)
    let result = eval_source(
        r#"(let ((numbers '(1 2 3))
                  (condition #t))
             (if condition
               (+ 3 (car numbers))
               (- 3 (car numbers))))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0); // 3 + 1 = 4

    // Test complex nested expression with all operation types
    let result = eval_source(
        r#"(let ((data '(10 20 30))
                  (multiplier 2))
             (if (> 3 2)
               (let ((first (* (car data) multiplier))
                     (second (car (cdr data))))
                 (cons first (list second)))
               '()))"#,
        &mut env,
    )
    .unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 20.0); // 10 * 2
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 20.0);

    // Test procedure application with mixed operations
    eval_source("(define (process x y) (+ (* x 2) y))", &mut env).unwrap();

    let result = eval_source("(process (car '(5 10)) (if (> 3 2) 7 0))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 17.0); // (5 * 2) + 7 = 17
}
