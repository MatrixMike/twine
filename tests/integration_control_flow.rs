//! Integration tests for conditional expressions
//!
//! This file contains integration tests for conditional functionality:
//! - Basic if expressions
//! - Conditional truthiness and falsy values
//! - Nested conditionals
//! - Conditionals with other language features

mod common;

use common::eval_source;

#[test]
fn test_integration_conditional_expressions() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("x", Value::number(5.0));
    env.define_str("y", Value::number(-3.0));

    // Basic conditionals
    let result = eval_source("(if #t \"yes\" \"no\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "yes");

    let result = eval_source("(if #f \"yes\" \"no\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "no");

    // Conditionals with comparisons
    let result = eval_source("(if (> x 0) \"positive\" \"not positive\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "positive");

    let result = eval_source("(if (< y 0) \"negative\" \"not negative\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "negative");

    // Conditionals with arithmetic
    let result = eval_source("(if (= (+ 2 3) 5) \"correct\" \"wrong\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "correct");

    // Nested expressions in branches
    let result = eval_source("(if (> x y) (+ x 10) (- y 1))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0); // x > y, so 5 + 10 = 15

    // Return different types
    let result = eval_source("(if #t 42 \"false\")", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    let result = eval_source("(if #f 42 \"false\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "false");
}

#[test]
fn test_integration_conditional_truthiness() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test Scheme truthiness - only #f is false
    let result = eval_source("(if 0 \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "truthy");

    let result = eval_source("(if \"\" \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "truthy");

    let result = eval_source("(if '() \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "truthy");

    let result = eval_source("(if 'symbol \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "truthy");

    // Only #f should be falsy
    let result = eval_source("(if #f \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "falsy");

    // #t should be truthy
    let result = eval_source("(if #t \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "truthy");
}

#[test]
fn test_integration_nested_conditionals() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("score", Value::number(85.0));

    // Nested if expressions
    let result = eval_source(
        "(if (>= score 90) \"A\" (if (>= score 80) \"B\" (if (>= score 70) \"C\" \"F\")))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "B");

    // Test different score ranges
    env.define_str("score", Value::number(95.0));
    let result = eval_source(
        "(if (>= score 90) \"A\" (if (>= score 80) \"B\" (if (>= score 70) \"C\" \"F\")))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "A");

    env.define_str("score", Value::number(65.0));
    let result = eval_source(
        "(if (>= score 90) \"A\" (if (>= score 80) \"B\" (if (>= score 70) \"C\" \"F\")))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "F");

    // Nested conditionals with different data types
    env.define_str("flag", Value::boolean(true));
    env.define_str("count", Value::number(3.0));

    let result = eval_source("(if flag (if (> count 0) count 0) -1)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);
}

#[test]
fn test_integration_conditionals_with_lists() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test if with null? predicate
    let result = eval_source("(if (null? '()) \"empty\" \"not empty\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "empty");

    let result = eval_source("(if (null? '(a)) \"empty\" \"not empty\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "not empty");

    // Test conditional with list operations
    let result = eval_source(
        "(if (> (length '(1 2 3)) 2) (car '(success fail)) (car '(fail success)))",
        &mut env,
    )
    .unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "success");

    // Conditional list construction
    let result = eval_source("(if #t '(1 2 3) '(4 5 6))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 1.0);

    // Using list elements in condition
    let result = eval_source(
        "(if (> (car '(10 5)) (car (cdr '(10 5)))) \"first-larger\" \"second-larger\")",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "first-larger");
}

#[test]
fn test_integration_conditionals_with_let() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with if expressions
    let result = eval_source("(let ((x 10) (y 20)) (if (< x y) x y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test let binding boolean values
    let result = eval_source("(let ((flag #t)) (if flag 'yes 'no))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "yes");

    // Conditional let bindings
    let result = eval_source(
        "(let ((choice (if (> 5 3) 'first 'second))) choice)",
        &mut env,
    )
    .unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "first");

    // Complex let with nested conditionals
    let result = eval_source(
        r#"(let ((a 10) (b 5))
             (if (> a b)
               (let ((diff (- a b)))
                 (if (> diff 3)
                   "big difference"
                   "small difference"))
               "b is larger"))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "big difference");
}

#[test]
fn test_integration_conditionals_error_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Too few arguments
    let result = eval_source("(if)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(if #t)", &mut env);
    assert!(result.is_err());

    // Too many arguments (if only takes 3)
    let result = eval_source("(if #t 1 2 3)", &mut env);
    assert!(result.is_err());

    // Condition evaluation error
    let result = eval_source("(if undefined-var \"yes\" \"no\")", &mut env);
    assert!(result.is_err());

    // Error in then branch
    let result = eval_source("(if #t (+ 1 \"not-number\") \"else\")", &mut env);
    assert!(result.is_err());

    // Error in else branch
    let result = eval_source("(if #f \"then\" (+ 1 \"not-number\"))", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_conditionals_with_procedures() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Define helper procedures
    eval_source("(define (positive? x) (> x 0))", &mut env).unwrap();
    eval_source("(define (abs x) (if (< x 0) (- x) x))", &mut env).unwrap();

    // Use procedure in condition
    let result = eval_source("(if (positive? 5) \"yes\" \"no\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "yes");

    let result = eval_source("(if (positive? -3) \"yes\" \"no\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "no");

    // Use conditional procedure
    let result = eval_source("(abs -10)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    let result = eval_source("(abs 7)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 7.0);

    // Conditional with lambda
    let result = eval_source(
        "((if #t (lambda (x) (+ x 1)) (lambda (x) (- x 1))) 5)",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0);
}

#[test]
fn test_integration_conditionals_comprehensive() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Complex scenario combining multiple features
    env.define_str(
        "data",
        Value::list(vec![
            Value::number(42.0),
            Value::string("test"),
            Value::boolean(true),
        ]),
    );

    // Multi-level conditional with various operations
    let result = eval_source(
        r#"(let ((first (car data))
                  (rest (cdr data)))
             (if (> first 40)
               (if (null? rest)
                 "first-only"
                 (let ((second (car rest)))
                   (if (string? second)
                     "has-string"
                     "no-string")))
               "small-first"))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "has-string");

    // Test with different first value
    env.define_str(
        "data",
        Value::list(vec![Value::number(20.0), Value::string("test")]),
    );

    let result = eval_source(
        r#"(let ((first (car data))
                  (rest (cdr data)))
             (if (> first 40)
               (if (null? rest)
                 "first-only"
                 (let ((second (car rest)))
                   (if (string? second)
                     "has-string"
                     "no-string")))
               "small-first"))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "small-first");
}

#[test]
fn test_integration_and_short_circuit_evaluation() {
    use crate::common::test_io;

    // Test and short-circuit: should not evaluate second expression when first is false
    test_io(
        r#"(and (begin (display "first") #f) (begin (display "second") #t))"#,
        "first",
    );

    // Test and all truthy: should evaluate all expressions
    test_io(
        r#"(and (begin (display "first") #t) (begin (display "second") #t) (begin (display "third") 42))"#,
        "firstsecondthird",
    );

    // Test and with early false: should stop at first false
    test_io(
        r#"(and (begin (display "eval1") 1) (begin (display "eval2") #f) (begin (display "eval3") 3))"#,
        "eval1eval2",
    );

    // Test empty and
    test_io(r#"(and)"#, "");
}

#[test]
fn test_integration_or_short_circuit_evaluation() {
    use crate::common::test_io;

    // Test or short-circuit: should not evaluate second expression when first is truthy
    test_io(
        r#"(or (begin (display "first") 42) (begin (display "second") #f))"#,
        "first",
    );

    // Test or all false: should evaluate all expressions
    test_io(
        r#"(or (begin (display "first") #f) (begin (display "second") #f))"#,
        "firstsecond",
    );

    // Test or with early truthy: should stop at first truthy
    test_io(
        r#"(or (begin (display "eval1") #f) (begin (display "eval2") "found") (begin (display "eval3") #t))"#,
        "eval1eval2",
    );

    // Test empty or
    test_io(r#"(or)"#, "");
}

#[test]
fn test_integration_nested_and_or_with_display() {
    use crate::common::test_io;

    // Test nested and/or with display to verify evaluation order
    test_io(
        r#"(and (begin (display "and1") #t)
                  (or (begin (display "or1") #f)
                      (begin (display "or2") #t))
                  (begin (display "and2") #t))"#,
        "and1or1or2and2",
    );

    // Test and inside or - and should short-circuit, then or continues
    test_io(
        r#"(or (begin (display "or1") #f)
                 (and (begin (display "and1") #t)
                      (begin (display "and2") #f)
                      (begin (display "and3") #t))
                 (begin (display "or2") #t))"#,
        "or1and1and2or2",
    );

    // Test complex nested logic with procedure calls
    test_io(
        r#"(and (begin (display "start") #t)
                  (or (begin (display "try1") #f)
                      (begin (display "try2") #f)
                      (begin (display "success") #t))
                  (begin (display "end") #t))"#,
        "starttry1try2successend",
    );
}

#[test]
fn test_integration_and_or_return_values() {
    use crate::common::eval_source;
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test and returns last value when all truthy
    let result = eval_source(r#"(and 1 2 "hello" 42)"#, &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Test and returns #f when any is false
    let result = eval_source(r#"(and 1 #f 3)"#, &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    // Test or returns first truthy value
    let result = eval_source(r#"(or #f #f "found" 42)"#, &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "found");

    // Test or returns #f when all false
    let result = eval_source(r#"(or #f #f #f)"#, &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    // Test empty and/or
    let result = eval_source(r#"(and)"#, &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source(r#"(or)"#, &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);
}

#[test]
fn test_integration_and_or_with_conditionals() {
    use crate::common::{eval_source, test_io};
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Define some test variables
    eval_source("(define x 10)", &mut env).unwrap();
    eval_source("(define y 5)", &mut env).unwrap();

    // Test and in if condition
    let result = eval_source(
        r#"(if (and (> x 0) (< y 10)) "both-true" "not-both")"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "both-true");

    // Test or in if condition
    let result = eval_source(
        r#"(if (or (< x 0) (> y 10)) "one-true" "both-false")"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "both-false");

    // Test complex logic with display to verify short-circuit in conditional context
    test_io(
        r#"(define result (if (and (begin (display "test1") #t)
                                     (begin (display "test2") #f)
                                     (begin (display "test3") #t))
                              "and-true"
                              "and-false"))
           (display result)"#,
        "test1test2and-false",
    );
}

#[test]
fn test_integration_if_branch_evaluation() {
    use crate::common::test_io;

    // Test if true branch: only consequent should be evaluated
    test_io(
        r#"(if (begin (display "condition") #t)
               (begin (display "then") "result")
               (begin (display "else") "other"))"#,
        "conditionthen",
    );

    // Test if false branch: only alternative should be evaluated
    test_io(
        r#"(if (begin (display "condition") #f)
               (begin (display "then") "result")
               (begin (display "else") "other"))"#,
        "conditionelse",
    );

    // Test if with complex condition evaluation
    test_io(
        r#"(if (begin (display "eval-cond") (> 5 3))
               (begin (display "true-branch") 42)
               (begin (display "false-branch") 0))"#,
        "eval-condtrue-branch",
    );

    // Test if with side effects in unused branch (should not execute)
    test_io(
        r#"(if #t
               (begin (display "executed") 1)
               (begin (display "not-executed") 2))"#,
        "executed",
    );

    test_io(
        r#"(if #f
               (begin (display "not-executed") 1)
               (begin (display "executed") 2))"#,
        "executed",
    );
}

#[test]
fn test_integration_if_nested_with_display() {
    use crate::common::test_io;

    // Test nested if expressions with display to track evaluation
    test_io(
        r#"(if (begin (display "outer-cond") #t)
               (if (begin (display "inner-cond") #f)
                   (begin (display "inner-then") "inner-result")
                   (begin (display "inner-else") "inner-alt"))
               (begin (display "outer-else") "outer-alt"))"#,
        "outer-condinner-condinner-else",
    );

    // Test if with multiple nested levels
    test_io(
        r#"(if (begin (display "level1") #t)
               (if (begin (display "level2") #t)
                   (begin (display "result") "found")
                   (begin (display "level2-else") "not-found"))
               (begin (display "level1-else") "error"))"#,
        "level1level2result",
    );

    // Test early termination in nested structure
    test_io(
        r#"(if (begin (display "check1") #f)
               (if (begin (display "should-not-run") #t)
                   (begin (display "deep-then") 1)
                   (begin (display "deep-else") 2))
               (begin (display "early-exit") "stopped"))"#,
        "check1early-exit",
    );
}

#[test]
fn test_integration_if_with_procedures_and_display() {
    use crate::common::test_io;

    // Test with inline procedure definitions - all in one test_io call
    test_io(
        r#"(begin
             (define (test-positive x)
               (begin (display "testing-positive")
                      (> x 0)))
             (define (positive-action x)
               (begin (display "positive-action")
                      (+ x 10)))
             (define (negative-action x)
               (begin (display "negative-action")
                      (- x 10)))
             (if (test-positive 5)
                 (positive-action 5)
                 (negative-action 5)))"#,
        "testing-positivepositive-action",
    );

    // Test with negative value - should call other branch
    test_io(
        r#"(begin
             (define (test-positive x)
               (begin (display "testing-positive")
                      (> x 0)))
             (define (positive-action x)
               (begin (display "positive-action")
                      (+ x 10)))
             (define (negative-action x)
               (begin (display "negative-action")
                      (- x 10)))
             (if (test-positive -3)
                 (positive-action -3)
                 (negative-action -3)))"#,
        "testing-positivenegative-action",
    );
}

#[test]
fn test_integration_if_evaluation_order_with_variables() {
    use crate::common::test_io;

    // Test that variable assignments in non-executed branches don't happen
    test_io(
        r#"(begin
             (define x 0)
             (if #t
                 (begin (display "true-branch")
                        (define x 1)
                        x)
                 (begin (display "false-branch")
                        (define x 2)
                        x))
             (display x))"#,
        "true-branch1",
    );

    test_io(
        r#"(begin
             (define y 0)
             (if #f
                 (begin (display "true-branch")
                        (define y 1)
                        y)
                 (begin (display "false-branch")
                        (define y 2)
                        y))
             (display y))"#,
        "false-branch2",
    );
}
