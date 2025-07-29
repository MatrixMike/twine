//! Integration tests for let binding expressions
//!
//! This file contains integration tests for let binding functionality:
//! - Basic let binding
//! - Lexical scoping and shadowing
//! - Simultaneous binding semantics
//! - Multiple body expressions
//! - Let with other language features
//! - Error handling

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
fn test_integration_let_binding_basic() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test basic let binding: (let ((x 42)) x)
    let result = eval_source("(let ((x 42)) x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Test multiple bindings: (let ((x 10) (y 20)) y)
    let result = eval_source("(let ((x 10) (y 20)) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0);

    // Test let with arithmetic in body
    let result = eval_source("(let ((x 10) (y 5)) (+ x y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);

    // Test let with different data types
    let result = eval_source("(let ((name \"Alice\") (age 30)) name)", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "Alice");

    let result = eval_source("(let ((flag #t) (count 0)) flag)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
}

#[test]
fn test_integration_let_binding_lexical_scoping() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Define x in outer environment
    env.define_str("x", Value::number(100.0));

    // Test lexical scoping - inner x should shadow outer x
    let result = eval_source("(let ((x 42)) x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // After let, outer x should still be accessible
    let result = eval_source("x", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 100.0);

    // Test that let doesn't affect outer environment
    let result = eval_source("(let ((y 999)) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 999.0);

    // y should not be accessible outside let
    let result = eval_source("y", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_let_binding_simultaneous() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Define x in outer environment
    env.define_str("x", Value::number(10.0));

    // Test simultaneous binding - y should see outer x, not inner x
    let result = eval_source("(let ((x 42) (y x)) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0); // y sees outer x

    // Test that both bindings work
    let result = eval_source("(let ((x 42) (y x)) x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0); // inner x

    // Test simultaneous binding with expressions
    let result = eval_source("(let ((a (+ 1 2)) (b (* 3 4))) (+ a b))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0); // 3 + 12
}

#[test]
fn test_integration_let_binding_multiple_body() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test multiple body expressions - should return last one
    let result = eval_source("(let ((x 5)) 1 2 3 x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    // Test with define in let body - should be scoped to let
    let result = eval_source("(let ((x 10)) (define y x) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test side effects in multiple expressions
    let result = eval_source("(let ((x 1)) (set! x 2) (set! x 3) x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);

    // Test with mixed expression types
    let result = eval_source("(let ((x 10)) \"ignored\" #f x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);
}

#[test]
fn test_integration_let_binding_with_lists() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with list operations
    let result = eval_source("(let ((lst '(1 2 3))) (car lst))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    // Test let with list construction
    let result = eval_source("(let ((x 42)) (list x x))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 42.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 42.0);

    // Test let with nested list operations
    let result = eval_source(
        "(let ((data '((a 1) (b 2)))) (car (cdr (car data))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    // Test let with list length
    let result = eval_source("(let ((items '(a b c d))) (length items))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);

    // Test let with cons operations
    let result = eval_source(
        "(let ((head 'first) (tail '(second third))) (cons head tail))",
        &mut env,
    )
    .unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "first");
}

#[test]
fn test_integration_let_binding_nested() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test nested let expressions
    let result = eval_source("(let ((x 10)) (let ((y 20)) (+ x y)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Test nested let with shadowing
    let result = eval_source("(let ((x 10)) (let ((x 20)) x))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0);

    // Test access to outer binding in nested let
    let result = eval_source("(let ((x 10)) (let ((y 20)) (+ x y)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Test deeply nested let
    let result = eval_source(
        "(let ((a 1)) (let ((b 2)) (let ((c 3)) (+ a b c))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0);

    // Test nested let with different scoping
    let result = eval_source(
        "(let ((x 1)) (let ((x 2) (y x)) (let ((x 3)) (+ x y))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0); // 3 + 1 (y sees outer x)
}

#[test]
fn test_integration_let_binding_with_conditionals() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with if expressions
    let result = eval_source("(let ((x 10) (y 20)) (if (< x y) x y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test let binding boolean values
    let result = eval_source("(let ((flag #t)) (if flag 'yes 'no))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "yes");

    // Test conditional let bindings
    let result = eval_source("(let ((value (if (> 5 3) 100 200))) value)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 100.0);

    // Test let in conditional branches
    let result = eval_source("(if #t (let ((x 42)) x) (let ((y 99)) y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Test complex conditional with let
    let result = eval_source(
        r#"(let ((threshold 50) (value 75))
             (if (> value threshold)
               (let ((bonus 10)) (+ value bonus))
               (let ((penalty 5)) (- value penalty))))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 85.0);
}

#[test]
fn test_integration_let_binding_with_define() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with define - bindings defined in let body stay in let scope
    let result = eval_source("(let ((x 42)) (define local-var x) local-var)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // The binding defined in let should NOT be accessible in outer scope
    assert!(env.lookup_str("local-var").is_err());

    // Test define procedure in let
    let result = eval_source(
        "(let ((factor 2)) (define (double x) (* x factor)) (double 21))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // The procedure should not be accessible outside let
    let result = eval_source("(double 5)", &mut env);
    assert!(result.is_err());

    // Test multiple defines in let
    let result = eval_source(
        r#"(let ((base 10))
             (define increment 5)
             (define (add-increment x) (+ x increment))
             (add-increment base))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);
}

#[test]
fn test_integration_let_binding_error_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with undefined identifier in binding expression
    let result = eval_source("(let ((x undefined-var)) x)", &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unbound identifier")
    );

    // Test malformed let syntax - no bindings
    let result = eval_source("(let)", &mut env);
    assert!(result.is_err());

    // Test malformed let syntax - no body
    let result = eval_source("(let ((x 1)))", &mut env);
    assert!(result.is_err());

    // Test malformed binding - not a list
    let result = eval_source("(let (x) x)", &mut env);
    assert!(result.is_err());

    // Test malformed binding - wrong number of elements
    let result = eval_source("(let ((x)) x)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(let ((x 1 2)) x)", &mut env);
    assert!(result.is_err());

    // Test binding to non-identifier
    let result = eval_source("(let ((42 1)) 42)", &mut env);
    assert!(result.is_err());

    // Test error in binding expression
    let result = eval_source("(let ((x (+ 1 \"not-number\"))) x)", &mut env);
    assert!(result.is_err());

    // Test error in body expression
    let result = eval_source("(let ((x 1)) (+ x \"not-number\"))", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_comprehensive_binding_behavior() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test complex scenario combining define and let
    eval_source("(define base 10)", &mut env).unwrap();

    let result = eval_source(
        "(let ((multiplier 3) (offset 5))
           (define computed (* base multiplier))
           (define result (+ computed offset))
           result)",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 35.0); // (10 * 3) + 5

    // Verify that let-scoped definitions don't leak
    assert!(env.lookup_str("multiplier").is_err());
    assert!(env.lookup_str("offset").is_err());
    assert!(env.lookup_str("computed").is_err());
    assert!(env.lookup_str("result").is_err());

    // But base should still be accessible
    let result = eval_source("base", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test let with lambda and closure
    eval_source("(define global-factor 100)", &mut env).unwrap();

    let result = eval_source(
        r#"(let ((local-factor 2))
             (define make-multiplier
               (lambda (x)
                 (* x local-factor global-factor)))
             (make-multiplier 3))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 600.0); // 3 * 2 * 100

    // Test complex nested scoping
    let result = eval_source(
        r#"(let ((x 1))
             (let ((x 2) (y x))
               (let ((x 3) (z y))
                 (+ x y z))))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0); // 3 + 1 + 1 (y and z both see original x)
}

#[test]
fn test_integration_let_with_procedures() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with lambda expressions
    let result = eval_source(
        "(let ((add-one (lambda (x) (+ x 1)))) (add-one 41))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Test let with procedure definitions
    eval_source("(define (square x) (* x x))", &mut env).unwrap();

    let result = eval_source("(let ((value 7)) (square value))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 49.0);

    // Test let capturing environment for procedures
    let result = eval_source(
        r#"(let ((base 100))
             (define (add-to-base x) (+ x base))
             (let ((base 200))
               (add-to-base 50)))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 150.0); // Uses captured base = 100

    // Test let with higher-order procedures
    let result = eval_source(
        r#"(let ((apply-twice (lambda (f x) (f (f x))))
                   (add-one (lambda (x) (+ x 1))))
             (apply-twice add-one 5))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 7.0); // add-one applied twice to 5
}
