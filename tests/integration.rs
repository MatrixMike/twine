//! Integration tests for Twine Scheme Interpreter
//!
//! This file contains comprehensive end-to-end integration tests that verify
//! the complete evaluation pipeline from source code parsing through execution.
//! These tests exercise the interaction between lexer, parser, evaluator, and
//! runtime environment components.

use twine_scheme::runtime::Environment;
use twine_scheme::types::{Symbol, Value};
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
fn test_integration_self_evaluating_atoms() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Numbers
    let result = eval_source("42", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    let result = eval_source("-17.5", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), -17.5);

    // Booleans
    let result = eval_source("#t", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("#f", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Strings
    let result = eval_source("\"hello world\"", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "hello world");

    let result = eval_source("\"\"", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "");
}

#[test]
fn test_integration_symbol_lookup() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("x", Value::number(42.0));
    env.define_str("name", Value::string("Scheme"));
    env.define_str("flag", Value::boolean(true));

    // Symbol lookup
    let result = eval_source("x", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    let result = eval_source("name", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "Scheme");

    let result = eval_source("flag", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Unbound symbol should error
    let result = eval_source("undefined", &mut env);
    assert!(result.is_err());
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

    // With identifiers
    let result = eval_source("(+ x y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 13.0);

    let result = eval_source("(* x y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Nested arithmetic
    let result = eval_source("(+ (* 2 3) (- 10 5))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 11.0);
}

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

    let result = eval_source("(= a 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(= a b)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Less than
    let result = eval_source("(< 3 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(< b a)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Greater than
    let result = eval_source("(> 5 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(> a b)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Less than or equal
    let result = eval_source("(<= 3 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(<= b a)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Greater than or equal
    let result = eval_source("(>= 5 5)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(>= a b)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
}

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

    // Conditionals with expressions
    let result = eval_source("(if (> x 0) \"positive\" \"non-positive\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "positive");

    let result = eval_source("(if (> y 0) \"positive\" \"non-positive\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "non-positive");

    // Scheme truthiness (only #f is false)
    let result = eval_source("(if 0 \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "truthy");

    let result = eval_source("(if \"\" \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "truthy");

    // Nested conditionals
    let result = eval_source("(if #t (if #f 1 2) 3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 2.0);

    let result = eval_source("(if #f (if #t 1 2) 3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);
}

#[test]
fn test_integration_quoted_expressions() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Quoted atoms
    let result = eval_source("'x", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "x");

    let result = eval_source("'42", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Quoted lists (should not be evaluated)
    let result = eval_source("'(+ 1 2)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "+");

    // Nested quotes
    let result = eval_source("''x", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "x");
}

#[test]
fn test_integration_complex_expressions() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("a", Value::number(10.0));
    env.define_str("b", Value::number(5.0));
    env.define_str("c", Value::number(2.0));

    // Complex arithmetic with conditionals
    let result = eval_source("(if (> a b) (+ a (* b c)) (- a b))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0); // 10 + (5 * 2)

    // Nested comparisons
    let result = eval_source(
        "(if (and #t (> a 0)) \"positive\" \"not positive\")",
        &mut env,
    );
    // This should fail because we haven't implemented 'and' yet
    assert!(result.is_err());

    // But this should work
    let result = eval_source(
        "(if (> a 0) (if (> b 0) \"both positive\" \"mixed\") \"negative\")",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_string().unwrap(), "both positive");

    // Complex nested expression
    let result = eval_source("(+ (* (if (> a b) a b) c) (- a c))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 28.0); // (10 * 2) + (10 - 2)
}

#[test]
fn test_integration_error_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Unbound symbol
    let result = eval_source("undefined-symbol", &mut env);
    assert!(result.is_err());

    // Type error in arithmetic
    let result = eval_source("(+ 1 \"not a number\")", &mut env);
    assert!(result.is_err());

    // Wrong arity for comparison operations (need exactly 2 args)
    let result = eval_source("(= 1)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(if #t)", &mut env);
    assert!(result.is_err());

    // Division by zero
    let result = eval_source("(/ 1 0)", &mut env);
    assert!(result.is_err());

    // Unknown procedure
    let result = eval_source("(unknown-proc 1 2)", &mut env);
    assert!(result.is_err());
}

// T2.3.5: Comprehensive integration tests for evaluation (20+ tests)
// These tests cover arithmetic, conditionals, and list operations

#[test]
fn test_integration_basic_list_construction() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test list construction with literal values
    let result = eval_source("(list 1 2 3)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 1.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 2.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 3.0);

    // Test empty list construction
    let result = eval_source("(list)", &mut env).unwrap();
    assert!(result.is_list());
    assert!(result.as_list().unwrap().is_empty());

    // Test mixed type list construction
    let result = eval_source("(list 42 \"hello\" #t)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 42.0);
    assert_eq!(list.get(1).unwrap().as_string().unwrap(), "hello");
    assert!(list.get(2).unwrap().as_boolean().unwrap());
}

#[test]
fn test_integration_list_access_operations() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test car operation
    let result = eval_source("(car '(apple banana cherry))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "apple");

    // Test cdr operation
    let result = eval_source("(cdr '(apple banana cherry))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "banana");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "cherry");

    // Test car of single element list
    let result = eval_source("(car '(only))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "only");

    // Test cdr of single element list returns empty list
    let result = eval_source("(cdr '(only))", &mut env).unwrap();
    assert!(result.is_list());
    assert!(result.as_list().unwrap().is_empty());
}

#[test]
fn test_integration_cons_operations() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test basic cons operation
    let result = eval_source("(cons 'first '(second third))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "first");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "second");
    assert_eq!(list.get(2).unwrap().as_symbol().unwrap(), "third");

    // Test cons with empty list
    let result = eval_source("(cons 42 '())", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 42.0);

    // Test cons with number and string list
    let result = eval_source("(cons 100 '(\"hello\" \"world\"))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 100.0);
    assert_eq!(list.get(1).unwrap().as_string().unwrap(), "hello");
    assert_eq!(list.get(2).unwrap().as_string().unwrap(), "world");
}

#[test]
fn test_integration_null_predicate() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test null? with empty list
    let result = eval_source("(null? '())", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test null? with non-empty list
    let result = eval_source("(null? '(1 2 3))", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test null? with single element list
    let result = eval_source("(null? '(only))", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test null? with non-list values
    let result = eval_source("(null? 42)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(null? \"string\")", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(null? #t)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());
}

#[test]
fn test_integration_nested_list_operations() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test car of car (nested access)
    let result = eval_source("(car (car '((a b) (c d))))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "a");

    // Test cdr of car
    let result = eval_source("(cdr (car '((a b) (c d))))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "b");

    // Test car of cdr
    let result = eval_source("(car (cdr '(first second third)))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "second");

    // Test nested cons operations
    let result = eval_source("(cons 'x (cons 'y '(z)))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "x");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "y");
    assert_eq!(list.get(2).unwrap().as_symbol().unwrap(), "z");
}

#[test]
fn test_integration_list_with_expressions() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test list construction with arithmetic expressions
    let result = eval_source("(list (+ 1 2) (* 3 4) (- 10 5))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 3.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 12.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 5.0);

    // Test cons with computed values
    let result = eval_source("(cons (+ 10 20) '(40 50))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 30.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 40.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 50.0);

    // Test car/cdr with constructed lists
    let result = eval_source("(car (list 'a 'b 'c))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "a");
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

    // Test arithmetic operations on list elements
    let result = eval_source("(+ (car my-list) 10)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 11.0);

    let result = eval_source("(* (car (cdr my-list)) 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test comparison with list elements
    let result = eval_source("(> (car my-list) 0)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(= (car (cdr (cdr my-list))) 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
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

    // Test nested conditionals with list operations
    let result = eval_source("(if (null? '()) '(default) (car '(a b c)))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "default");

    // Test conditional list construction
    let result = eval_source("(if #t (list 1 2) (list 3 4))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 1.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 2.0);
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
    assert_eq!(result.as_number().unwrap(), 21.0);

    // Test arithmetic as list elements
    let result = eval_source("(car (list (+ 5 5) (* 2 3)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test comparison of list elements
    let result = eval_source("(> (car '(10 5)) (car (cdr '(10 5))))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
}

#[test]
fn test_integration_list_reconstruction() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test reconstructing a list using car and cdr
    let original = "'(a b c)";
    let reconstructed = "(cons (car '(a b c)) (cdr '(a b c)))";

    let original_result = eval_source(original, &mut env).unwrap();
    let reconstructed_result = eval_source(reconstructed, &mut env).unwrap();

    assert!(original_result.is_list());
    assert!(reconstructed_result.is_list());

    let orig_list = original_result.as_list().unwrap();
    let recon_list = reconstructed_result.as_list().unwrap();

    assert_eq!(orig_list.len(), recon_list.len());
    for i in 0..orig_list.len() {
        assert_eq!(
            orig_list.get(i).unwrap().as_symbol().unwrap(),
            recon_list.get(i).unwrap().as_symbol().unwrap()
        );
    }

    // Test building list element by element
    let step_by_step = "(cons 'a (cons 'b (cons 'c '())))";
    let step_result = eval_source(step_by_step, &mut env).unwrap();

    assert!(step_result.is_list());
    let step_list = step_result.as_list().unwrap();
    assert_eq!(step_list.len(), 3);
    assert_eq!(step_list.get(0).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(step_list.get(1).unwrap().as_symbol().unwrap(), "b");
    assert_eq!(step_list.get(2).unwrap().as_symbol().unwrap(), "c");
}

#[test]
fn test_integration_list_error_conditions() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test car on empty list
    let result = eval_source("(car '())", &mut env);
    assert!(result.is_err());

    // Test cdr on empty list
    let result = eval_source("(cdr '())", &mut env);
    assert!(result.is_err());

    // Test car on non-list
    let result = eval_source("(car 42)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cdr \"string\")", &mut env);
    assert!(result.is_err());

    // Test cdr on non-list
    let result = eval_source("(cons)", &mut env);
    assert!(result.is_err());

    // Test cons with non-list second argument
    let result = eval_source("(cons 1 42)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cons 'a \"not-a-list\")", &mut env);
    assert!(result.is_err());
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

    // Test unary operations
    let result = eval_source("(- 5)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), -5.0);

    let result = eval_source("(/ 4)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.25);

    // Test identity elements
    let result = eval_source("(+)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);

    let result = eval_source("(*)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    // Test negative numbers
    let result = eval_source("(+ -5 10)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    let result = eval_source("(* -2 -3)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0);
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

    // Test ordering with multiple arguments
    let result = eval_source("(< 1 2 3 4)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(< 1 2 2 4)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(> 4 3 2 1)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test <= and >= with equal values
    let result = eval_source("(<= 1 2 2 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(>= 3 2 2 1)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    // Test with negative numbers
    let result = eval_source("(< -5 -2 0 3)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
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

    let result = eval_source("(if #f \"truthy\" \"falsy\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "falsy");

    // Test with list predicates
    let result = eval_source("(if (null? '()) \"empty\" \"not-empty\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "empty");

    let result = eval_source("(if (null? '(a)) \"empty\" \"not-empty\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "not-empty");
}

#[test]
fn test_integration_mixed_operations() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test arithmetic in conditional context
    let result = eval_source("(if (> (+ 2 3) 4) \"greater\" \"lesser\")", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "greater");

    // Test list operations in arithmetic context
    let result = eval_source("(+ (car '(5 10)) (car (cdr '(5 10))))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);

    // Test conditional list selection
    let result = eval_source("(car (if #t '(first second) '(third fourth)))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "first");

    // Test complex nested expression
    let result = eval_source(
        "(cons (if (< 2 3) 'yes 'no) (list (+ 1 1) (* 2 2)))",
        &mut env,
    )
    .unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "yes");
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 2.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 4.0);
}

#[test]
fn test_integration_list_with_quotes() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test quoted lists don't evaluate contents
    let result = eval_source("'(+ 1 2)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "+");
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 1.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 2.0);

    // Test list operations on quoted lists
    let result = eval_source("(car '(a b c))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "a");

    // Test nested quotes
    let result = eval_source("(car '(quote x))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "quote");

    // Test mixing quoted and unquoted
    let result = eval_source("(cons (+ 1 2) '(4 5))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 3.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 4.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 5.0);
}

#[test]
fn test_integration_identifier_binding_with_lists() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Define some list identifiers
    env.define_str(
        "numbers",
        Value::list(vec![
            Value::number(10.0),
            Value::number(20.0),
            Value::number(30.0),
        ]),
    );
    env.define_str(
        "symbols",
        Value::list(vec![
            Value::symbol("alpha"),
            Value::symbol("beta"),
            Value::symbol("gamma"),
        ]),
    );
    env.define_str("empty", Value::empty_list());

    // Test operations on bound lists
    let result = eval_source("(car numbers)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    let result = eval_source("(null? empty)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());

    let result = eval_source("(cons 'prefix symbols)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "prefix");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "alpha");

    // Test arithmetic with bound list elements
    let result = eval_source("(+ (car numbers) (car (cdr numbers)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Test conditional with bound lists
    let result = eval_source("(if (null? empty) 'yes 'no)", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "yes");
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

    // Test empty bindings: (let () 'hello)
    let result = eval_source("(let () 'hello)", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "hello");
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

    // Verify outer environment unchanged
    assert_eq!(env.lookup_str("x").unwrap().as_number().unwrap(), 100.0);

    // Test accessing outer scope from within let
    env.define_str("y", Value::number(50.0));
    let result = eval_source("(let ((x 42)) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 50.0);
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
    assert_eq!(result.as_number().unwrap(), 10.0); // Should be outer x, not 42

    // Test with expressions that reference each other indirectly
    env.define_str("a", Value::number(5.0));
    env.define_str("b", Value::number(7.0));
    let result = eval_source("(let ((x a) (y b)) (+ x y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 12.0);
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

    // Verify y is NOT accessible in outer scope (correct Scheme behavior)
    assert!(env.lookup_str("y").is_err());
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

    // Test let with cons
    let result = eval_source("(let ((x 'hello) (y '(world))) (cons x y))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "hello");
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

    // Test nested let accessing outer scope
    let result = eval_source("(let ((x 10) (y 5)) (let ((z 3)) (+ x y z)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 18.0);
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
    assert_eq!(result.as_symbol().unwrap(), "yes");

    // Test complex conditional logic
    let result = eval_source(
        "(let ((a 5) (b 0)) (if (= b 0) 'division-by-zero a))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_symbol().unwrap(), "division-by-zero");
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
    assert!(env.lookup_str("x").is_err());

    // Test that outer definitions are still accessible in let
    eval_source("(define outer 100)", &mut env).unwrap();
    let result = eval_source("(let ((inner 200)) outer)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 100.0);

    // Verify outer scope unchanged
    assert_eq!(env.lookup_str("outer").unwrap().as_number().unwrap(), 100.0);
    assert!(env.lookup_str("inner").is_err());
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

    // Test let with undefined identifier in body
    let result = eval_source("(let ((x 42)) undefined-var)", &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unbound identifier")
    );

    // Test invalid let syntax - should be caught by parser or evaluator
    // Note: These might be caught at parse time depending on implementation
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
           (+ computed offset))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 35.0); // (10 * 3) + 5

    // Verify what's in the environment
    assert_eq!(env.lookup_str("base").unwrap().as_number().unwrap(), 10.0);
    // computed was defined in let scope, so it should NOT be accessible in outer scope
    assert!(env.lookup_str("computed").is_err());
    assert!(env.lookup_str("multiplier").is_err()); // Should not be accessible
    assert!(env.lookup_str("offset").is_err()); // Should not be accessible

    // Test let expressions that build on each other
    let result = eval_source(
        "(let ((x 1))
           (let ((y (+ x 1)))
             (let ((z (+ y 1)))
               (list x y z))))",
        &mut env,
    )
    .unwrap();

    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 1.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 2.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 3.0);
}

#[test]
fn test_integration_comprehensive_evaluation() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();
    env.define_str("x", Value::number(5.0));
    env.define_str("y", Value::number(10.0));
    env.define_str(
        "data",
        Value::list(vec![Value::number(100.0), Value::number(200.0)]),
    );

    // Test comprehensive expression combining all features
    let complex_expr = r#"
        (if (> x 0)
            (cons (+ x y) data)
            '(error))
    "#;

    let result = eval_source(complex_expr, &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 15.0); // x + y
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 100.0); // first of data
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 200.0); // second of data

    // Test another complex expression
    let complex_expr2 = r#"
        (car (if (> (car data) y)
                 (cdr data)
                 data))
    "#;

    let result = eval_source(complex_expr2, &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 200.0);

    // Test arithmetic-heavy list operation
    let arithmetic_expr = r#"
        (list (+ x y)
              (* x y)
              (/ (car data) x))
    "#;

    let result = eval_source(arithmetic_expr, &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 15.0); // 5 + 10
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 50.0); // 5 * 10
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 20.0); // 100 / 5
}

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
    assert_eq!(proc.name(), "<lambda>");
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

    // Verify the lambda captured the environment
    let captured_env = proc.env().unwrap();
    let captured_value = captured_env.lookup(&Symbol::new("outer-value")).unwrap();
    assert_eq!(captured_value.as_number().unwrap(), 100.0);
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
            .contains("lambda: expected 2 arguments")
    );

    // Test lambda with non-list parameter
    let invalid_params = "(lambda x 42)";
    let result = eval_source(invalid_params, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("parameters must be enclosed in parentheses")
    );

    // Test lambda with duplicate parameters
    let duplicate_params = "(lambda (x x) 42)";
    let result = eval_source(duplicate_params, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("duplicate parameter")
    );

    // Test lambda with non-symbol parameter
    let non_symbol_param = "(lambda (42) x)";
    let result = eval_source(non_symbol_param, &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("parameter must be a symbol")
    );
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

    // The lambda should capture the lexical environment
    let captured_env = proc.env().unwrap();
    let outer_value = captured_env.lookup(&Symbol::new("outer")).unwrap();
    assert_eq!(outer_value.as_number().unwrap(), 10.0);
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

    let params = proc.params().unwrap();
    assert_eq!(params[0], Symbol::new("if"));
    assert_eq!(params[1], Symbol::new("define"));
    assert_eq!(params[2], Symbol::new("lambda"));
}

// T3.1.3: Function Application Tests

#[test]
fn test_integration_lambda_application_basic() {
    let mut env = Environment::new();

    // Define a simple lambda and call it
    let lambda_def = "(define add1 (lambda (x) (+ x 1)))";
    eval_source(lambda_def, &mut env).unwrap();

    // Call the lambda
    let call_result = eval_source("(add1 5)", &mut env).unwrap();
    assert_eq!(call_result, Value::number(6.0));

    // Call with expression argument
    let call_with_expr = eval_source("(add1 (* 2 3))", &mut env).unwrap();
    assert_eq!(call_with_expr, Value::number(7.0));
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

    // Call with expression arguments
    let result_expr = eval_source("(add3 (+ 1 1) (* 2 2) (- 5 2))", &mut env).unwrap();
    assert_eq!(result_expr, Value::number(9.0)); // 2 + 4 + 3
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

    // Change x and call again - should still use captured x
    eval_source("(define x 20)", &mut env).unwrap();
    let result2 = eval_source("(addx 5)", &mut env).unwrap();
    assert_eq!(result2, Value::number(15.0)); // Still uses original x=10
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

    // More complex nesting
    let complex_result = eval_source("(add1 (double (add1 2)))", &mut env).unwrap();
    assert_eq!(complex_result, Value::number(7.0)); // add1(double(add1(2))) = add1(double(3)) = add1(6) = 7
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
}

#[test]
fn test_integration_lambda_application_complex_expression() {
    let mut env = Environment::new();

    // Call lambda directly without defining it first
    let direct_call = "((lambda (x y) (+ (* x x) (* y y))) 3 4)";
    let result = eval_source(direct_call, &mut env).unwrap();
    assert_eq!(result, Value::number(25.0)); // 3² + 4² = 9 + 16 = 25
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
    let result = eval_source("(add2 5 10 15)", &mut env);
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
}

#[test]
fn test_integration_lambda_application_error_cases() {
    let mut env = Environment::new();

    // Test calling non-procedure
    eval_source("(define x 42)", &mut env).unwrap();
    let result = eval_source("(x 1 2 3)", &mut env);
    assert!(result.is_err());
    if let Err(Error::RuntimeError(msg)) = result {
        assert!(msg.contains("'x' is not a procedure, got number"));
    } else {
        panic!("Expected RuntimeError for calling non-procedure");
    }

    // Test calling undefined identifier
    let result = eval_source("(undefined-proc 1 2)", &mut env);
    assert!(result.is_err());
    if let Err(Error::EnvironmentError { .. }) = result {
        // Expected
    } else {
        panic!("Expected UnboundIdentifier error");
    }
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
    assert_eq!(result, Value::number(6.0)); // Uses parameter x=5, not outer x=100

    // Outer x should be unchanged
    let outer_x = eval_source("x", &mut env).unwrap();
    assert_eq!(outer_x, Value::number(100.0));
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

    // Test the created function
    let result = eval_source("(add5 7)", &mut env).unwrap();
    assert_eq!(result, Value::number(22.0)); // 5 + 7 + 10 = 22

    // Test with expressions
    let expr_result = eval_source("(add5 (* 2 3))", &mut env).unwrap();
    assert_eq!(expr_result, Value::number(21.0)); // 5 + 6 + 10 = 21
}
