//! Integration tests for basic Scheme language features
//!
//! This file contains integration tests for fundamental language constructs:
//! - Self-evaluating atoms (numbers, strings, booleans)
//! - Symbol lookup and identifier binding
//! - Quoted expressions
//! - Basic error handling

mod common;

use common::eval_source;
use twine_scheme::runtime::Environment;
use twine_scheme::types::Value;

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

    // Empty string should remain empty
    let result = eval_source("\"\"", &mut env).unwrap();
    assert!(result.as_string().unwrap().is_empty());
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

    // Case sensitivity
    let result = eval_source("X", &mut env);
    assert!(result.is_err()); // X should be different from x
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

    let result = eval_source("'\"hello\"", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "hello");

    // Quoted lists
    let result = eval_source("'(1 2 3)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 1.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 2.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 3.0);
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

    // Wrong number of arguments
    let result = eval_source("(+)", &mut env);
    assert!(result.is_err());

    // Invalid procedure call
    let result = eval_source("(42 1 2)", &mut env);
    assert!(result.is_err());

    // Malformed expressions should be caught by parser
    // These would fail at parse time, not evaluation time
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
    assert_eq!(result.as_number().unwrap(), 20.0); // 10 + (5 * 2) = 20

    // Nested quotes and symbols
    let result = eval_source("'(quote x)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "quote");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "x");

    // Mixed data types in expressions
    let result = eval_source("(if #t \"success\" 'failure)", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "success");

    // Complex nested structure
    env.define_str("result", Value::number(42.0));
    let result = eval_source("'(the answer is result)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "the");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "answer");
    assert_eq!(list.get(2).unwrap().as_symbol().unwrap(), "is");
    assert_eq!(list.get(3).unwrap().as_symbol().unwrap(), "result");
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
        "words",
        Value::list(vec![Value::string("hello"), Value::string("world")]),
    );

    env.define_str(
        "mixed",
        Value::list(vec![
            Value::number(42.0),
            Value::string("answer"),
            Value::boolean(true),
        ]),
    );

    // Use identifiers bound to lists
    let result = eval_source("(car numbers)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    let result = eval_source("(car words)", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "hello");

    let result = eval_source("(car (cdr mixed))", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "answer");

    // Check length
    let result = eval_source("(length numbers)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);

    // Combine with arithmetic
    let result = eval_source("(+ (car numbers) (car (cdr numbers)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0); // 10 + 20
}
