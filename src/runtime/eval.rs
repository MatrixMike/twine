//! Evaluation engine for the Twine Scheme runtime
//!
//! This module implements the core evaluation logic for Scheme expressions.
//! It handles atomic values, symbol lookup, and provides the foundation
//! for list evaluation (procedure calls and special forms).

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::types::{List, Value};

use super::{Environment, builtin};

/// Evaluate a Scheme expression in the given environment
///
/// This is the core evaluation function that handles:
/// - Self-evaluating atoms (numbers, booleans, strings)
/// - Symbol lookup in the environment
/// - Basic list evaluation framework
///
/// # Arguments
/// * `expr` - The expression to evaluate
/// * `env` - The environment for symbol lookup
///
/// # Returns
/// * `Result<Value>` - The evaluated value or an error
pub fn eval(expr: &Expression, env: &Environment) -> Result<Value> {
    match expr {
        // Atoms are handled based on their value type
        Expression::Atom(value) => eval_atom(value, env),

        // Lists represent procedure calls or special forms
        Expression::List(elements) => eval_list(elements, env),

        // Quoted expressions prevent evaluation
        Expression::Quote(boxed_expr) => eval_quote(boxed_expr),
    }
}

/// Evaluate an atomic expression
///
/// Atoms are evaluated based on their type:
/// - Numbers, booleans, strings, and lists are self-evaluating
/// - Symbols are looked up in the environment
fn eval_atom(value: &Value, env: &Environment) -> Result<Value> {
    match value {
        // Self-evaluating values
        Value::Number(_) | Value::Boolean(_) | Value::String(_) | Value::List(_) => {
            Ok(value.clone())
        }

        // Symbols need environment lookup
        Value::Symbol(symbol) => env.lookup(symbol),

        // Handle nil value
        Value::Nil => Ok(Value::Nil),
    }
}

/// Evaluate a list expression
///
/// Lists represent compound expressions in Scheme:
/// - Empty lists evaluate to themselves
/// - Non-empty lists represent procedure calls: (procedure arg1 arg2 ...)
fn eval_list(elements: &[Expression], env: &Environment) -> Result<Value> {
    // Empty list evaluates to empty list
    if elements.is_empty() {
        return Ok(Value::List(List::new()));
    }

    // Non-empty lists are procedure calls: (procedure arg1 arg2 ...)
    let procedure_expr = &elements[0];
    let arg_exprs = &elements[1..];

    // Check if the procedure expression is a builtin procedure symbol
    if let Expression::Atom(Value::Symbol(symbol)) = procedure_expr {
        // Evaluate all arguments
        let mut args = Vec::new();
        for arg_expr in arg_exprs {
            args.push(eval(arg_expr, env)?);
        }

        // Handle builtin procedures through centralized dispatch
        if let Some(result) = builtin::dispatch(symbol.as_str(), &args) {
            result
        } else {
            // Not a builtin procedure, try to evaluate as normal procedure call
            let _procedure = eval(procedure_expr, env)?;
            Err(Error::runtime_error(&format!(
                "Unknown procedure: '{}'",
                symbol.as_str()
            )))
        }
    } else {
        // Evaluate the procedure expression and handle other types of procedures
        let procedure = eval(procedure_expr, env)?;

        // Evaluate all arguments
        let mut args = Vec::new();
        for arg_expr in arg_exprs {
            args.push(eval(arg_expr, env)?);
        }

        Err(Error::runtime_error(&format!(
            "Procedure call expects symbol, got {}",
            procedure.type_name()
        )))
    }
}

/// Evaluate a quoted expression
///
/// Quoted expressions prevent evaluation and return the quoted expression
/// as a value without evaluating it.
fn eval_quote(expr: &Expression) -> Result<Value> {
    // Convert the quoted expression back to a Value
    expression_to_value(expr)
}

/// Convert an Expression back to a Value
///
/// This is used for quote evaluation where we need to return
/// the quoted expression as a value without evaluating it.
fn expression_to_value(expr: &Expression) -> Result<Value> {
    match expr {
        Expression::Atom(value) => Ok(value.clone()),

        Expression::List(elements) => {
            let mut values = Vec::new();
            for element in elements {
                values.push(expression_to_value(element)?);
            }
            Ok(Value::List(List::from(values)))
        }

        Expression::Quote(boxed_expr) => {
            // Nested quotes - convert the inner expression
            expression_to_value(boxed_expr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_eval_self_evaluating_atoms() {
        let env = Environment::new();

        // Test number evaluation
        let number_expr = Expression::atom(Value::number(42.0));
        let result = eval(&number_expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        // Test boolean evaluation
        let bool_expr = Expression::atom(Value::boolean(true));
        let result = eval(&bool_expr, &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test string evaluation
        let string_expr = Expression::atom(Value::string("hello"));
        let result = eval(&string_expr, &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");

        // Test nil evaluation
        let nil_expr = Expression::atom(Value::Nil);
        let result = eval(&nil_expr, &env).unwrap();
        assert!(result.is_nil());
    }

    #[test]
    fn test_eval_symbol_lookup() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(10.0));
        env.define_str("greeting", Value::string("hello world"));

        // Test successful symbol lookup
        let symbol_expr = Expression::atom(Value::symbol("x"));
        let result = eval(&symbol_expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);

        let greeting_expr = Expression::atom(Value::symbol("greeting"));
        let result = eval(&greeting_expr, &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_eval_unbound_symbol() {
        let env = Environment::new();

        // Test unbound symbol error
        let symbol_expr = Expression::atom(Value::symbol("undefined"));
        let result = eval(&symbol_expr, &env);

        assert!(result.is_err());
        if let Err(Error::EnvironmentError { identifier, .. }) = result {
            assert_eq!(identifier, "undefined");
        } else {
            panic!("Expected EnvironmentError for unbound symbol");
        }
    }

    #[test]
    fn test_eval_empty_list() {
        let env = Environment::new();

        // Empty list should evaluate to empty list
        let empty_list = Expression::list(vec![]);
        let result = eval(&empty_list, &env).unwrap();

        assert!(result.is_list());
        assert!(result.as_list().unwrap().is_empty());
    }

    #[test]
    fn test_eval_arithmetic_operations() {
        let env = Environment::new();

        // Test addition
        let add_expr = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]);
        let result = eval(&add_expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        // Test subtraction
        let sub_expr = Expression::list(vec![
            Expression::atom(Value::symbol("-")),
            Expression::atom(Value::number(10.0)),
            Expression::atom(Value::number(3.0)),
        ]);
        let result = eval(&sub_expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 7.0);

        // Test multiplication
        let mul_expr = Expression::list(vec![
            Expression::atom(Value::symbol("*")),
            Expression::atom(Value::number(3.0)),
            Expression::atom(Value::number(4.0)),
        ]);
        let result = eval(&mul_expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 12.0);

        // Test division
        let div_expr = Expression::list(vec![
            Expression::atom(Value::symbol("/")),
            Expression::atom(Value::number(15.0)),
            Expression::atom(Value::number(3.0)),
        ]);
        let result = eval(&div_expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);

        // Test equality
        let eq_expr = Expression::list(vec![
            Expression::atom(Value::symbol("=")),
            Expression::atom(Value::number(5.0)),
            Expression::atom(Value::number(5.0)),
        ]);
        let result = eval(&eq_expr, &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test less than
        let lt_expr = Expression::list(vec![
            Expression::atom(Value::symbol("<")),
            Expression::atom(Value::number(3.0)),
            Expression::atom(Value::number(5.0)),
        ]);
        let result = eval(&lt_expr, &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_eval_unknown_procedure() {
        let env = Environment::new();

        // Test unknown procedure
        let unknown_expr = Expression::list(vec![
            Expression::atom(Value::symbol("unknown")),
            Expression::atom(Value::number(1.0)),
        ]);
        let result = eval(&unknown_expr, &env);
        assert!(result.is_err());
        if let Err(Error::EnvironmentError { identifier, .. }) = result {
            assert_eq!(identifier, "unknown");
        } else {
            panic!("Expected EnvironmentError for unbound symbol");
        }
    }

    #[test]
    fn test_eval_non_symbol_procedure() {
        let env = Environment::new();

        // Test non-symbol as procedure
        let non_symbol_proc = Expression::list(vec![
            Expression::atom(Value::number(42.0)),
            Expression::atom(Value::number(1.0)),
        ]);
        let result = eval(&non_symbol_proc, &env);
        assert!(result.is_err());
        if let Err(Error::RuntimeError(msg)) = result {
            assert!(msg.contains("Procedure call expects symbol, got number"));
        } else {
            panic!("Expected RuntimeError for non-symbol procedure");
        }
    }

    #[test]
    fn test_eval_arithmetic_with_variables() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(10.0));
        env.define_str("y", Value::number(5.0));

        // Test arithmetic with variables: (+ x y)
        let add_vars = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
        ]);
        let result = eval(&add_vars, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 15.0);

        // Test nested arithmetic: (* (+ x y) 2)
        let nested = Expression::list(vec![
            Expression::atom(Value::symbol("*")),
            Expression::list(vec![
                Expression::atom(Value::symbol("+")),
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::symbol("y")),
            ]),
            Expression::atom(Value::number(2.0)),
        ]);
        let result = eval(&nested, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 30.0);
    }

    #[test]
    fn test_eval_comparison_operations() {
        let env = Environment::new();

        // Test greater than
        let gt_expr = Expression::list(vec![
            Expression::atom(Value::symbol(">")),
            Expression::atom(Value::number(5.0)),
            Expression::atom(Value::number(3.0)),
        ]);
        let result = eval(&gt_expr, &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test less than or equal
        let lte_expr = Expression::list(vec![
            Expression::atom(Value::symbol("<=")),
            Expression::atom(Value::number(3.0)),
            Expression::atom(Value::number(3.0)),
        ]);
        let result = eval(&lte_expr, &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test greater than or equal
        let gte_expr = Expression::list(vec![
            Expression::atom(Value::symbol(">=")),
            Expression::atom(Value::number(5.0)),
            Expression::atom(Value::number(3.0)),
        ]);
        let result = eval(&gte_expr, &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_eval_quote_atom() {
        let env = Environment::new();

        // Quoted atom should return the atom value without evaluation
        let quoted_symbol = Expression::quote(Expression::atom(Value::symbol("undefined")));
        let result = eval(&quoted_symbol, &env).unwrap();

        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "undefined");

        // Even though "undefined" is not bound in environment,
        // it should not error because it's quoted
    }

    #[test]
    fn test_eval_quote_list() {
        let env = Environment::new();

        // Quoted list should return the list structure without evaluation
        let quoted_list = Expression::quote(Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]));

        let result = eval(&quoted_list, &env).unwrap();

        assert!(result.is_list());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);

        // Check the contents
        assert!(list.get(0).unwrap().is_symbol());
        assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "+");
        assert_eq!(list.get(1).unwrap().as_number().unwrap(), 1.0);
        assert_eq!(list.get(2).unwrap().as_number().unwrap(), 2.0);
    }

    #[test]
    fn test_expression_to_value_conversion() {
        // Test atom conversion
        let number_expr = Expression::atom(Value::number(42.0));
        let result = expression_to_value(&number_expr).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        // Test list conversion
        let list_expr = Expression::list(vec![
            Expression::atom(Value::symbol("test")),
            Expression::atom(Value::boolean(true)),
        ]);

        let result = expression_to_value(&list_expr).unwrap();
        assert!(result.is_list());

        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "test");
        assert_eq!(list.get(1).unwrap().as_boolean().unwrap(), true);
    }

    #[test]
    fn test_nested_environment_symbol_lookup() {
        let mut global = Environment::new();
        global.define_str("global_var", Value::number(100.0));

        let mut local = Environment::new_scope(&global);
        local.define_str("local_var", Value::string("local"));

        // Test lookup from local environment
        let local_expr = Expression::atom(Value::symbol("local_var"));
        let result = eval(&local_expr, &local).unwrap();
        assert_eq!(result.as_string().unwrap(), "local");

        // Test lookup from parent environment
        let global_expr = Expression::atom(Value::symbol("global_var"));
        let result = eval(&global_expr, &local).unwrap();
        assert_eq!(result.as_number().unwrap(), 100.0);
    }

    #[test]
    fn test_eval_list_values() {
        let env = Environment::new();

        // Test that list values are self-evaluating
        let list_value = Value::List(List::from(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]));

        let expr = Expression::atom(list_value.clone());
        let result = eval(&expr, &env).unwrap();

        assert!(result.is_list());
        let result_list = result.as_list().unwrap();
        assert_eq!(result_list.len(), 3);
        assert_eq!(result_list.get(0).unwrap().as_number().unwrap(), 1.0);
        assert_eq!(result_list.get(1).unwrap().as_number().unwrap(), 2.0);
        assert_eq!(result_list.get(2).unwrap().as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_eval_integration() {
        use crate::parser::Expression;
        use crate::types::Value;

        // Test eval function with self-evaluating atoms
        let env = Environment::new();

        let number_expr = Expression::atom(Value::number(42.0));
        let result = eval(&number_expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        let string_expr = Expression::atom(Value::string("hello"));
        let result = eval(&string_expr, &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");

        // Test eval function with symbol lookup
        let mut env_with_binding = Environment::new();
        env_with_binding.define_str("x", Value::number(10.0));

        let symbol_expr = Expression::atom(Value::symbol("x"));
        let result = eval(&symbol_expr, &env_with_binding).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);

        // Test eval function with quoted expressions
        let quoted_expr = Expression::quote(Expression::atom(Value::symbol("unbound")));
        let result = eval(&quoted_expr, &env).unwrap();
        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "unbound");

        // Test eval function with empty list
        let empty_list = Expression::list(vec![]);
        let result = eval(&empty_list, &env).unwrap();
        assert!(result.is_list());
        assert!(result.as_list().unwrap().is_empty());
    }
}
