//! Integration tests for list operations
//!
//! This file contains integration tests for list functionality:
//! - List construction (list)
//! - List access operations (car, cdr)
//! - Cons operations
//! - Null predicate and list predicates
//! - Nested list operations
//! - List error handling

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
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 0);

    // Test mixed type list
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

    // Test car on single element list
    let result = eval_source("(car '(single))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "single");

    // Test cdr on single element list (should return empty list)
    let result = eval_source("(cdr '(single))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 0);

    // Test with numbers
    let result = eval_source("(car '(1 2 3))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);
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

    // Test cons with single element
    let result = eval_source("(cons 'a 'b)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "b");

    // Test cons with mixed types
    let result = eval_source("(cons 1 '(\"hello\" #f))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 1.0);
    assert_eq!(list.get(1).unwrap().as_string().unwrap(), "hello");
    assert!(!list.get(2).unwrap().as_boolean().unwrap());
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
    let result = eval_source("(null? '(a))", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test null? with non-list values
    let result = eval_source("(null? 42)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(null? \"string\")", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    let result = eval_source("(null? #t)", &mut env).unwrap();
    assert!(!result.as_boolean().unwrap());

    // Test null? with cdr of single element list
    let result = eval_source("(null? (cdr '(only)))", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
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
    let result = eval_source("(car (cdr '((a b) (c d))))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "c");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "d");

    // Test deeply nested access
    let result = eval_source("(car (cdr (car '((a b c) (d e f)))))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "b");

    // Test nested cons operations
    let result = eval_source("(cons (cons 'a '(b)) '(c d))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    // First element should be (a b)
    let first = list.get(0).unwrap().as_list().unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(first.get(0).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(first.get(1).unwrap().as_symbol().unwrap(), "b");
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

    // Test list construction with conditional expressions
    let result = eval_source("(list (if #t 'yes 'no) (if #f 1 2))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "yes");
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 2.0);

    // Test cons with expressions
    let result = eval_source("(cons (+ 10 5) '(20 30))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 15.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 20.0);
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 30.0);
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

    // Both should be lists with same content
    assert!(original_result.is_list());
    assert!(reconstructed_result.is_list());

    let orig_list = original_result.as_list().unwrap();
    let recon_list = reconstructed_result.as_list().unwrap();

    assert_eq!(orig_list.len(), recon_list.len());
    assert_eq!(orig_list.len(), 3);

    for i in 0..3 {
        assert_eq!(
            orig_list.get(i).unwrap().as_symbol().unwrap(),
            recon_list.get(i).unwrap().as_symbol().unwrap()
        );
    }

    // Test more complex reconstruction
    let result = eval_source(
        "(cons 'new (cons (car '(a b)) (cdr (cdr '(a b c)))))",
        &mut env,
    )
    .unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "new");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(list.get(2).unwrap().as_symbol().unwrap(), "c");
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

    let result = eval_source("(car \"string\")", &mut env);
    assert!(result.is_err());

    let result = eval_source("(car #t)", &mut env);
    assert!(result.is_err());

    // Test cdr on non-list
    let result = eval_source("(cdr 42)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cdr \"string\")", &mut env);
    assert!(result.is_err());

    // Test wrong number of arguments
    let result = eval_source("(car)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(car '(a) '(b))", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cons)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cons 1)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(cons 1 2 3)", &mut env);
    assert!(result.is_err());
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

    // Compare with evaluated list
    let evaluated = eval_source("(list '+ 1 2)", &mut env).unwrap();
    assert!(evaluated.is_list());
    let eval_list = evaluated.as_list().unwrap();
    assert_eq!(eval_list.len(), 3);
    assert_eq!(eval_list.get(0).unwrap().as_symbol().unwrap(), "+");
    assert_eq!(eval_list.get(1).unwrap().as_number().unwrap(), 1.0);
    assert_eq!(eval_list.get(2).unwrap().as_number().unwrap(), 2.0);

    // Test nested quoted lists
    let result = eval_source("'((a b) (c d))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);

    let first_sublist = list.get(0).unwrap().as_list().unwrap();
    assert_eq!(first_sublist.len(), 2);
    assert_eq!(first_sublist.get(0).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(first_sublist.get(1).unwrap().as_symbol().unwrap(), "b");

    // Test operations on quoted lists
    let result = eval_source("(car '(quoted list))", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "quoted");

    let result = eval_source("(length '(a b c d e))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);
}

#[test]
fn test_integration_list_length_operations() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test length of various lists
    let result = eval_source("(length '())", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 0.0);

    let result = eval_source("(length '(single))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    let result = eval_source("(length '(a b c d e))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    // Test length with constructed lists
    let result = eval_source("(length (list 1 2 3 4))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);

    let result = eval_source("(length (cons 'a '(b c)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0);

    // Test length in arithmetic expressions
    let result = eval_source("(+ (length '(1 2)) (length '(a b c)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0); // 2 + 3

    // Test length with nested lists
    let result = eval_source("(length '((a b) (c d) (e f)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 3.0); // Three sublists

    // Test length error cases
    let result = eval_source("(length 42)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(length \"string\")", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_list_comprehensive() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Define complex list data
    env.define_str(
        "nested-data",
        Value::list(vec![
            Value::list(vec![Value::number(1.0), Value::number(2.0)]),
            Value::list(vec![
                Value::string("hello".to_string()),
                Value::string("world".to_string()),
            ]),
            Value::list(vec![Value::boolean(true), Value::boolean(false)]),
        ]),
    );

    // Test complex nested operations
    let result = eval_source("(car (car nested-data))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    let result = eval_source("(car (cdr (car (cdr nested-data))))", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "world");

    // Test list reconstruction with complex data
    let result = eval_source("(cons (list 'new 'first) (cdr nested-data))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);

    // First element should be (new first)
    let first = list.get(0).unwrap().as_list().unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(first.get(0).unwrap().as_symbol().unwrap(), "new");
    assert_eq!(first.get(1).unwrap().as_symbol().unwrap(), "first");

    // Test combining arithmetic with list operations
    let result = eval_source(
        "(+ (length (car nested-data)) (length (car (cdr nested-data))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0); // 2 + 2
}
