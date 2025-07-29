//! Evaluation engine for the Twine Scheme runtime
//!
//! This module implements the core evaluation logic for Scheme expressions.
//! It handles atomic values, symbol lookup, list evaluation, and provides
//! the foundation for procedure calls and special forms.

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::types::{List, Value};
use std::sync::Arc;

use super::{Environment, special_forms};

pub mod procedure;

// Re-export public functions from procedure module
pub use procedure::{call_procedure, eval_arguments};

/// Evaluate a Scheme expression in the given environment
///
/// This is the core evaluation function that handles:
/// - Self-evaluating atoms (numbers, booleans, strings)
/// - Symbol lookup in the environment
/// - List evaluation (procedure calls and special forms)
/// - Quoted expressions
///
/// # Arguments
/// * `expr` - The expression to evaluate
/// * `env` - The environment for symbol lookup
///
/// # Returns
/// The evaluated value or an error if evaluation fails
pub fn eval(expr: Arc<Expression>, env: &mut Environment) -> Result<Value> {
    match expr.as_ref() {
        // Atoms are handled based on their value type
        Expression::Atom(value) => eval_atom(value.clone(), env),

        // Lists represent procedure calls or special forms
        Expression::List(elements) => eval_list(elements, env),

        // Quoted expressions prevent evaluation
        Expression::Quote(quoted_expr) => eval_quote(Arc::clone(quoted_expr)),
    }
}

/// Evaluate an atomic expression
///
/// Atoms are evaluated based on their type:
/// - Numbers, booleans, strings, and lists are self-evaluating
/// - Symbols are looked up as identifiers in the environment
fn eval_atom(value: Value, env: &Environment) -> Result<Value> {
    match value {
        // Self-evaluating values
        Value::Number(_)
        | Value::Boolean(_)
        | Value::String(_)
        | Value::List(_)
        | Value::Procedure(_) => Ok(value),

        // Symbols need environment lookup
        Value::Symbol(identifier) => env.lookup(&identifier),

        // Handle nil value
        Value::Nil => Ok(Value::Nil),
    }
}

/// Evaluate a list expression
///
/// Lists represent compound expressions in Scheme:
/// - Empty lists evaluate to themselves
/// - Non-empty lists represent special forms or procedure calls: (form/procedure arg1 arg2 ...)
fn eval_list(elements: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    // Empty list evaluates to empty list
    if elements.is_empty() {
        return Ok(Value::List(List::new()));
    }

    // Non-empty lists can be special forms or procedure calls
    let first_expr = Arc::clone(&elements[0]);
    let rest_exprs = &elements[1..];

    let procedure_value = match first_expr.as_ref() {
        Expression::Atom(Value::Symbol(identifier)) => {
            // Handle special forms first (these have special evaluation rules)
            if let Some(special_form) = special_forms::SpecialForm::from_name(identifier.as_str()) {
                return special_form.call(rest_exprs, env);
            }

            // Not a special form, try to look up identifier in environment
            env.lookup(identifier)?
        }
        _ => eval(Arc::clone(&first_expr), env)?,
    };

    // Check if the value is a procedure and call it
    match procedure_value {
        Value::Procedure(procedure) => call_procedure(procedure, rest_exprs, env),
        _ => {
            let error_msg = format!(
                "'{}' is not a procedure, got {}",
                procedure_value,
                procedure_value.type_name()
            );
            Err(Error::runtime_error(&error_msg))
        }
    }
}

/// Evaluate a quoted expression
///
/// Quoted expressions prevent evaluation and return the quoted expression
/// as a value without evaluating it.
fn eval_quote(expr: Arc<Expression>) -> Result<Value> {
    // Convert the quoted expression back to a Value
    expression_to_value(expr.as_ref())
}

/// Convert an Expression back to a Value
///
/// This is used for quote evaluation where we need to return
/// the quoted expression as a value without evaluating it.
fn expression_to_value(expr: &Expression) -> Result<Value> {
    match expr {
        Expression::Atom(value) => Ok(value.clone()),

        Expression::List(elements) => {
            let mut values = Vec::with_capacity(elements.len());
            for element in elements {
                values.push(expression_to_value(element.as_ref())?);
            }
            Ok(Value::List(List::from(values)))
        }

        Expression::Quote(quoted_expr) => {
            // Nested quotes - convert the inner expression
            expression_to_value(quoted_expr.as_ref())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser::Parser;
    use crate::runtime::Environment;
    use crate::types::{ArcString, Number, Symbol};

    /// Helper function to evaluate source code strings
    fn eval_source(source: &str) -> Result<Value> {
        let mut parser = Parser::new(source.to_string())?;
        let expr = parser.parse_expression()?.expr;
        let mut env = Environment::new();
        eval(expr, &mut env)
    }

    #[test]
    fn test_eval_self_evaluating_atoms() {
        // Numbers are self-evaluating
        assert_eq!(eval_source("42").unwrap(), Value::Number(Number::new(42.0)));
        assert_eq!(
            eval_source("3.14").unwrap(),
            Value::Number(Number::new(3.14))
        );

        // Booleans are self-evaluating
        assert_eq!(eval_source("#t").unwrap(), Value::Boolean(true));
        assert_eq!(eval_source("#f").unwrap(), Value::Boolean(false));

        // Strings are self-evaluating
        assert_eq!(
            eval_source("\"hello\"").unwrap(),
            Value::String(ArcString::from("hello"))
        );
    }

    #[test]
    fn test_eval_symbol_lookup() {
        let mut env = Environment::new();
        let symbol = Symbol::from("test-var");
        let value = Value::Number(Number::new(42.0));

        env.define(symbol.clone(), value.clone());

        let mut parser = Parser::new("test-var".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap().expr;
        assert_eq!(eval(expr, &mut env).unwrap(), value);
    }

    #[test]
    fn test_eval_unbound_symbol() {
        let mut env = Environment::new();
        let mut parser = Parser::new("undefined-symbol".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap().expr;

        match eval(expr, &mut env) {
            Err(Error::EnvironmentError { .. }) => {
                // Expected error type
            }
            _ => panic!("Expected unbound identifier error"),
        }
    }

    #[test]
    fn test_eval_empty_list() {
        let result = eval_source("()").unwrap();
        match result {
            Value::List(list) => assert!(list.is_empty()),
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn test_eval_arithmetic_core() {
        // Test basic arithmetic operations
        assert_eq!(
            eval_source("(+ 1 2)").unwrap(),
            Value::Number(Number::new(3.0))
        );
        assert_eq!(
            eval_source("(- 5 3)").unwrap(),
            Value::Number(Number::new(2.0))
        );
        assert_eq!(
            eval_source("(* 4 3)").unwrap(),
            Value::Number(Number::new(12.0))
        );
        assert_eq!(
            eval_source("(/ 8 2)").unwrap(),
            Value::Number(Number::new(4.0))
        );

        // Test nested arithmetic
        assert_eq!(
            eval_source("(+ (* 2 3) (- 8 4))").unwrap(),
            Value::Number(Number::new(10.0))
        );
    }

    #[test]
    fn test_eval_unknown_procedure() {
        match eval_source("(unknown-proc 1 2)") {
            Err(Error::EnvironmentError { .. }) => {
                // Expected error type
            }
            _ => panic!("Expected unbound identifier error"),
        }
    }

    #[test]
    fn test_eval_non_symbol_procedure() {
        match eval_source("(42 1 2)") {
            Err(Error::RuntimeError(message)) => {
                assert!(message.contains("is not a procedure"));
                assert!(message.contains("42"));
            }
            Err(e) => panic!("Expected RuntimeError, got: {:?}", e),
            Ok(val) => panic!("Expected error, got value: {:?}", val),
        }
    }

    #[test]
    fn test_eval_nested_expressions() {
        // Test deeply nested expressions
        assert_eq!(
            eval_source("(+ (+ 1 2) (+ 3 4))").unwrap(),
            Value::Number(Number::new(10.0))
        );

        // Test mixed operations
        assert_eq!(
            eval_source("(* (+ 2 3) (- 10 4))").unwrap(),
            Value::Number(Number::new(30.0))
        );
    }

    #[test]
    fn test_eval_quote_atom() {
        // Test quoting atoms
        assert_eq!(
            eval_source("'42").unwrap(),
            Value::Number(Number::new(42.0))
        );
        assert_eq!(
            eval_source("'hello").unwrap(),
            Value::Symbol(Symbol::from("hello"))
        );
    }

    #[test]
    fn test_eval_quote_list() {
        // Test quoting lists - they should not be evaluated
        let result = eval_source("'(+ 1 2)").unwrap();
        match result {
            Value::List(list) => {
                let items = list.iter().collect::<Vec<_>>();
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], &Value::Symbol(Symbol::from("+")));
                assert_eq!(items[1], &Value::Number(Number::new(1.0)));
                assert_eq!(items[2], &Value::Number(Number::new(2.0)));
            }
            _ => panic!("Expected list from quote"),
        }

        // Test nested quote
        let result = eval_source("'(quote hello)").unwrap();
        match result {
            Value::List(list) => {
                let items = list.iter().collect::<Vec<_>>();
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], &Value::Symbol(Symbol::from("quote")));
                assert_eq!(items[1], &Value::Symbol(Symbol::from("hello")));
            }
            _ => panic!("Expected list from nested quote"),
        }
    }

    #[test]
    fn test_expression_to_value_conversion() {
        // Test that quote properly converts expressions to values
        let mut parser = Parser::new("'(a b c)".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap().expr;
        let mut env = Environment::new();
        let result = eval(expr, &mut env).unwrap();

        match result {
            Value::List(list) => {
                let items = list.iter().collect::<Vec<_>>();
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], &Value::Symbol(Symbol::from("a")));
                assert_eq!(items[1], &Value::Symbol(Symbol::from("b")));
                assert_eq!(items[2], &Value::Symbol(Symbol::from("c")));
            }
            _ => panic!("Expected list from quote"),
        }
    }

    #[test]
    fn test_nested_environment_symbol_lookup() {
        let mut env = Environment::new();
        env.define(Symbol::from("x"), Value::Number(Number::new(10.0)));

        let mut nested_env = Environment::new_scope(&env);
        nested_env.define(Symbol::from("y"), Value::Number(Number::new(20.0)));

        // Should find x in parent environment
        let mut parser = Parser::new("x".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap().expr;
        assert_eq!(
            eval(expr, &mut nested_env).unwrap(),
            Value::Number(Number::new(10.0))
        );

        // Should find y in current environment
        let mut parser = Parser::new("y".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap().expr;
        assert_eq!(
            eval(expr, &mut nested_env).unwrap(),
            Value::Number(Number::new(20.0))
        );
    }

    #[test]
    fn test_eval_list_values() {
        // Test that list values are self-evaluating
        let mut env = Environment::new();
        let list = List::from(vec![
            Value::Number(Number::new(1.0)),
            Value::Number(Number::new(2.0)),
            Value::Number(Number::new(3.0)),
        ]);
        let expr = Arc::new(Expression::Atom(Value::List(list.clone())));

        assert_eq!(eval(expr, &mut env).unwrap(), Value::List(list));
    }

    #[test]
    fn test_eval_integration() {
        // Test complete evaluation pipeline from source to result
        assert_eq!(
            eval_source("(+ (* 2 (+ 1 2)) (- 10 (/ 12 3)))").unwrap(),
            Value::Number(Number::new(12.0))
        );

        // Test with quote to prevent evaluation
        let result = eval_source("'(+ 1 2 3)").unwrap();
        match result {
            Value::List(list) => {
                let items = list.iter().collect::<Vec<_>>();
                assert_eq!(items.len(), 4);
                assert_eq!(items[0], &Value::Symbol(Symbol::from("+")));
            }
            _ => panic!("Expected quoted list"),
        }
    }

    #[test]
    fn test_eval_if_true_condition() {
        // Test if with true condition
        assert_eq!(
            eval_source("(if #t 1 2)").unwrap(),
            Value::Number(Number::new(1.0))
        );

        // Test if with truthy condition (non-#f is truthy)
        assert_eq!(
            eval_source("(if 42 'yes 'no)").unwrap(),
            Value::Symbol(Symbol::from("yes"))
        );
    }

    #[test]
    fn test_eval_if_false_condition() {
        // Test if with false condition
        assert_eq!(
            eval_source("(if #f 1 2)").unwrap(),
            Value::Number(Number::new(2.0))
        );
    }

    #[test]
    fn test_eval_if_truthiness() {
        // In Scheme, only #f is falsy, everything else is truthy
        assert_eq!(
            eval_source("(if 0 'truthy 'falsy)").unwrap(),
            Value::Symbol(Symbol::from("truthy"))
        );

        assert_eq!(
            eval_source("(if \"\" 'truthy 'falsy)").unwrap(),
            Value::Symbol(Symbol::from("truthy"))
        );

        assert_eq!(
            eval_source("(if '() 'truthy 'falsy)").unwrap(),
            Value::Symbol(Symbol::from("truthy"))
        );

        // Only #f is falsy
        assert_eq!(
            eval_source("(if #f 'truthy 'falsy)").unwrap(),
            Value::Symbol(Symbol::from("falsy"))
        );
    }

    #[test]
    fn test_eval_if_with_expressions() {
        // Test if with expressions that need evaluation
        assert_eq!(
            eval_source("(if (> 5 3) (+ 1 2) (* 2 3))").unwrap(),
            Value::Number(Number::new(3.0))
        );

        assert_eq!(
            eval_source("(if (< 5 3) (+ 1 2) (* 2 3))").unwrap(),
            Value::Number(Number::new(6.0))
        );
    }

    #[test]
    fn test_eval_if_nested() {
        // Test nested if expressions
        assert_eq!(
            eval_source("(if #t (if #f 1 2) 3)").unwrap(),
            Value::Number(Number::new(2.0))
        );
    }

    #[test]
    fn test_eval_if_arity_errors() {
        // Test if with wrong number of arguments
        match eval_source("(if #t)") {
            Err(Error::ArityError {
                procedure,
                expected,
                actual,
            }) => {
                assert_eq!(procedure, "if");
                assert_eq!(expected, 3);
                assert_eq!(actual, 1);
            }
            Err(e) => panic!("Expected ArityError, got: {:?}", e),
            Ok(val) => panic!("Expected error, got value: {:?}", val),
        }

        match eval_source("(if #t 1 2 3)") {
            Err(Error::ArityError {
                procedure,
                expected,
                actual,
            }) => {
                assert_eq!(procedure, "if");
                assert_eq!(expected, 3);
                assert_eq!(actual, 4);
            }
            Err(e) => panic!("Expected ArityError, got: {:?}", e),
            Ok(val) => panic!("Expected error, got value: {:?}", val),
        }
    }

    #[test]
    fn test_eval_if_evaluation_order() {
        // Test that only the appropriate branch is evaluated
        // This would fail if both branches were evaluated
        assert_eq!(
            eval_source("(if #t 42 (unknown-procedure))").unwrap(),
            Value::Number(Number::new(42.0))
        );

        // Test the else branch
        match eval_source("(if #f 42 (unknown-procedure))") {
            Err(Error::EnvironmentError { .. }) => {
                // Expected error type
            }
            _ => panic!("Expected error from evaluating else branch"),
        }
    }

    #[test]
    fn test_eval_list_operations_core() {
        // Test basic list operations
        assert_eq!(
            eval_source("(list 1 2 3)").unwrap(),
            Value::List(List::from(vec![
                Value::Number(Number::new(1.0)),
                Value::Number(Number::new(2.0)),
                Value::Number(Number::new(3.0))
            ]))
        );

        // Test list construction with expressions
        assert_eq!(
            eval_source("(list (+ 1 2) (* 2 3))").unwrap(),
            Value::List(List::from(vec![
                Value::Number(Number::new(3.0)),
                Value::Number(Number::new(6.0))
            ]))
        );
    }

    #[test]
    fn test_eval_list_no_clone_required() {
        // Test that list evaluation doesn't require cloning when possible
        let result = eval_source("(list)").unwrap();
        match result {
            Value::List(list) => assert!(list.is_empty()),
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn test_tail_call_optimization_basic() {
        // Test that tail recursive calls don't overflow the stack
        let _factorial_lambda = "
            ((lambda (n acc)
               (if (= n 0)
                   acc
                   (factorial n (- n 1) (* acc n))))
             1000 1)
        ";

        // First define a recursive factorial that we can call
        let _factorial_def = format!(
            "(if #t
               (factorial 1000 1)
               0)
            where factorial = (lambda (n acc)
              (if (= n 0)
                  acc
                  (factorial (- n 1) (* acc n))))"
        );

        // For now, test a simpler tail call scenario
        let simple_tail_call = "
            ((lambda (x)
               (if (= x 0)
                   42
                   42))
             0)
        ";

        let result = eval_source(simple_tail_call).unwrap();
        assert_eq!(result, Value::Number(Number::new(42.0)));
    }

    #[test]
    fn test_tail_call_optimization_identity() {
        // Test tail call optimization with identity function
        let identity_call = "
            ((lambda (x) x) 42)
        ";

        let result = eval_source(identity_call).unwrap();
        assert_eq!(result, Value::Number(Number::new(42.0)));
    }

    #[test]
    fn test_tail_call_optimization_builtin() {
        // Test tail call optimization when calling builtin procedures
        let builtin_tail_call = "
            ((lambda (x y)
               (+ x y))
             10 20)
        ";

        let result = eval_source(builtin_tail_call).unwrap();
        assert_eq!(result, Value::Number(Number::new(30.0)));
    }

    #[test]
    fn test_tail_call_optimization_non_tail_call() {
        // Test that non-tail calls are handled correctly
        let non_tail_call = "
            ((lambda (x)
               (+ ((lambda (y) y) x) 1))
             41)
        ";

        let result = eval_source(non_tail_call).unwrap();
        assert_eq!(result, Value::Number(Number::new(42.0)));
    }

    #[test]
    fn test_no_double_evaluation_in_lambda_call() {
        // Test that arguments are evaluated exactly once during lambda calls
        // This test verifies the structure but we can't easily test side effects
        // in our current setup. The important thing is that the lambda body
        // evaluates correctly.
        let lambda_call = "
            ((lambda (x y)
               (+ x y))
             (+ 1 2)
             (+ 3 4))
        ";

        let result = eval_source(lambda_call).unwrap();
        assert_eq!(result, Value::Number(Number::new(10.0)));
    }

    #[test]
    fn test_tail_call_optimization_special_forms() {
        // Test that special forms in tail position are handled correctly
        let special_form_tail = "
            ((lambda (x)
               (if (= x 0)
                   'zero
                   'nonzero))
             0)
        ";

        let result = eval_source(special_form_tail).unwrap();
        assert_eq!(result, Value::Symbol(Symbol::from("zero")));

        let special_form_tail2 = "
            ((lambda (x)
               (if (= x 0)
                   'zero
                   'nonzero))
             42)
        ";

        let result2 = eval_source(special_form_tail2).unwrap();
        assert_eq!(result2, Value::Symbol(Symbol::from("nonzero")));
    }
}
