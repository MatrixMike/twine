//! Integration tests for Twine Scheme Interpreter
//!
//! This file contains comprehensive end-to-end integration tests that verify
//! the complete evaluation pipeline from source code parsing through execution.
//! These tests exercise the interaction between lexer, parser, evaluator, and
//! runtime environment components.

use twine_scheme::Result;

// Helper function for end-to-end evaluation testing
fn eval_source(
    source: &str,
    env: &mut twine_scheme::runtime::Environment,
) -> Result<twine_scheme::types::Value> {
    use twine_scheme::parser::Parser;
    use twine_scheme::runtime::eval::eval;

    let mut parser = Parser::new(source.to_string())?;
    let expr = parser.parse_expression()?.expr;
    eval(&expr, env)
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
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("#f", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

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
    assert_eq!(result.as_boolean().unwrap(), true);

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

    // With variables
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
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(= a 5)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(= a b)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    // Less than
    let result = eval_source("(< 3 5)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(< b a)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    // Greater than
    let result = eval_source("(> 5 3)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(> a b)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    // Less than or equal
    let result = eval_source("(<= 3 3)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(<= b a)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    // Greater than or equal
    let result = eval_source("(>= 5 5)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(>= a b)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);
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
    assert_eq!(list.get(2).unwrap().as_boolean().unwrap(), true);
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
    assert_eq!(result.as_boolean().unwrap(), true);

    // Test null? with non-empty list
    let result = eval_source("(null? '(1 2 3))", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    // Test null? with single element list
    let result = eval_source("(null? '(only))", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    // Test null? with non-list values
    let result = eval_source("(null? 42)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    let result = eval_source("(null? \"string\")", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    let result = eval_source("(null? #t)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);
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
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(= (car (cdr (cdr my-list))) 3)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);
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
    assert_eq!(result.as_boolean().unwrap(), true);
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
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(= 5 5 6)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    // Test ordering with multiple arguments
    let result = eval_source("(< 1 2 3 4)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(< 1 2 2 4)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), false);

    let result = eval_source("(> 4 3 2 1)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    // Test <= and >= with equal values
    let result = eval_source("(<= 1 2 2 3)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    let result = eval_source("(>= 3 2 2 1)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);

    // Test with negative numbers
    let result = eval_source("(< -5 -2 0 3)", &mut env).unwrap();
    assert_eq!(result.as_boolean().unwrap(), true);
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
fn test_integration_variable_binding_with_lists() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Define some list variables
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
    assert_eq!(result.as_boolean().unwrap(), true);

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
