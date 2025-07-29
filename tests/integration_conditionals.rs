//! Integration tests for conditional expressions
//!
//! This file contains integration tests for conditional functionality:
//! - Basic if expressions
//! - Conditional truthiness and falsy values
//! - Nested conditionals
//! - Conditionals with other language features

mod common;

use common::eval_source;
use twine_scheme::types::Value;

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
