//! Evaluation engine for the Twine Scheme runtime
//!
//! This module implements the core evaluation logic for Scheme expressions.
//! It handles atomic values, symbol lookup, and provides the foundation
//! for list evaluation (procedure calls and special forms).

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::types::{List, Value};

use super::{Environment, special_forms};

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
pub fn eval(expr: Expression, env: &mut Environment) -> Result<Value> {
    match expr {
        // Atoms are handled based on their value type
        Expression::Atom(value) => eval_atom(value, env),

        // Lists represent procedure calls or special forms
        Expression::List(elements) => eval_list(elements, env),

        // Quoted expressions prevent evaluation
        Expression::Quote(boxed_expr) => eval_quote(*boxed_expr),
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
fn eval_list(elements: Vec<Expression>, env: &mut Environment) -> Result<Value> {
    // Empty list evaluates to empty list
    if elements.is_empty() {
        return Ok(Value::List(List::new()));
    }

    // Non-empty lists can be special forms or procedure calls
    let mut elements_iter = elements.into_iter();
    let first_expr = elements_iter.next().unwrap();
    let rest_exprs: Vec<Expression> = elements_iter.collect();

    let procedure_value = match first_expr {
        Expression::Atom(Value::Symbol(identifier)) => {
            // Handle special forms first (these have special evaluation rules)
            if let Some(special_form) = special_forms::SpecialForm::from_name(identifier.as_str()) {
                return special_form.call(rest_exprs, env);
            }

            // Not a special form, try to look up identifier in environment
            env.lookup(&identifier)?
        }
        first_expr => eval(first_expr, env)?,
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

/// Call a procedure with the given arguments
///
/// Handles both builtin and lambda procedures:
/// - For builtin procedures, delegates to the builtin implementation
/// - For lambda procedures, creates new environment, binds parameters, and evaluates body
fn call_procedure(
    procedure: crate::types::Procedure,
    arg_exprs: Vec<Expression>,
    env: &mut Environment,
) -> Result<Value> {
    match procedure {
        crate::types::Procedure::Builtin(builtin) => {
            // Evaluate arguments for builtin procedures
            let args = eval_arguments(arg_exprs, env)?;
            builtin.call(&args)
        }
        crate::types::Procedure::Lambda(lambda) => {
            // Check arity
            let expected_arity = lambda.arity();
            let actual_arity = arg_exprs.len();

            if expected_arity != actual_arity {
                return Err(Error::arity_error("<lambda>", expected_arity, actual_arity));
            }

            // Evaluate arguments
            let args = eval_arguments(arg_exprs, env)?;

            // Create new environment extending the lambda's closure
            let mut call_env = Environment::new_scope(lambda.env());

            // Bind parameters to arguments
            for (param, arg) in lambda.params().iter().zip(args.iter()) {
                call_env.define(param.clone(), arg.clone());
            }

            // Evaluate the body in the new environment
            //
            // TODO: Remove lambda body clone from here.
            eval(lambda.body().clone(), &mut call_env)
        }
    }
}

/// Evaluate a list of argument expressions into values
fn eval_arguments(exprs: Vec<Expression>, env: &mut Environment) -> Result<Vec<Value>> {
    let mut args = Vec::with_capacity(exprs.len());
    for expr in exprs {
        args.push(eval(expr, env)?);
    }
    Ok(args)
}

/// Evaluate a quoted expression
///
/// Quoted expressions prevent evaluation and return the quoted expression
/// as a value without evaluating it.
fn eval_quote(expr: Expression) -> Result<Value> {
    // Convert the quoted expression back to a Value
    expression_to_value(expr)
}

/// Convert an Expression back to a Value
///
/// This is used for quote evaluation where we need to return
/// the quoted expression as a value without evaluating it.
fn expression_to_value(expr: Expression) -> Result<Value> {
    match expr {
        Expression::Atom(value) => Ok(value),

        Expression::List(elements) => {
            let mut values = Vec::with_capacity(elements.len());
            for element in elements {
                values.push(expression_to_value(element)?);
            }
            Ok(Value::List(List::from(values)))
        }

        Expression::Quote(boxed_expr) => {
            // Nested quotes - convert the inner expression
            expression_to_value(*boxed_expr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_eval_self_evaluating_atoms() {
        let mut env = Environment::new();

        // Test number evaluation
        let number_expr = Expression::atom(Value::number(42.0));
        let result = eval(number_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        // Test boolean evaluation
        let bool_expr = Expression::atom(Value::boolean(true));
        let result = eval(bool_expr, &mut env).unwrap();
        assert!(result.as_boolean().unwrap());

        // Test string evaluation
        let string_expr = Expression::atom(Value::string("hello"));
        let result = eval(string_expr, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");

        // Test nil evaluation
        let nil_expr = Expression::atom(Value::Nil);
        let result = eval(nil_expr, &mut env).unwrap();
        assert!(result.is_nil());
    }

    #[test]
    fn test_eval_symbol_lookup() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(10.0));
        env.define_str("greeting", Value::string("hello world"));

        // Test successful symbol lookup
        let symbol_expr = Expression::atom(Value::symbol("x"));
        let result = eval(symbol_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);

        let greeting_expr = Expression::atom(Value::symbol("greeting"));
        let result = eval(greeting_expr, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_eval_unbound_symbol() {
        let mut env = Environment::new();

        // Test unbound symbol error
        let symbol_expr = Expression::atom(Value::symbol("undefined"));
        let result = eval(symbol_expr, &mut env);

        assert!(result.is_err());
        if let Err(Error::EnvironmentError { identifier, .. }) = result {
            assert_eq!(identifier, "undefined");
        } else {
            panic!("Expected EnvironmentError for unbound symbol");
        }
    }

    #[test]
    fn test_eval_empty_list() {
        let mut env = Environment::new();

        // Empty list should evaluate to empty list
        let empty_list = Expression::list(vec![]);
        let result = eval(empty_list, &mut env).unwrap();

        assert!(result.is_list());
        assert!(result.as_list().unwrap().is_empty());
    }

    #[test]
    fn test_eval_arithmetic_core() {
        let mut env = Environment::new();

        // Test that arithmetic operations work at the eval level with Expression objects
        // (Comprehensive arithmetic testing is done in integration tests)
        let add_expr = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]);
        let result = eval(add_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        // Test that comparison operations return proper boolean values
        let eq_expr = Expression::list(vec![
            Expression::atom(Value::symbol("=")),
            Expression::atom(Value::number(5.0)),
            Expression::atom(Value::number(5.0)),
        ]);
        let result = eval(eq_expr, &mut env).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_eval_unknown_procedure() {
        let mut env = Environment::new();

        // Test unknown procedure
        let unknown_expr = Expression::list(vec![
            Expression::atom(Value::symbol("unknown")),
            Expression::atom(Value::number(1.0)),
        ]);
        let result = eval(unknown_expr, &mut env);
        assert!(result.is_err());
        if let Err(Error::EnvironmentError { identifier, .. }) = result {
            assert_eq!(identifier, "unknown");
        } else {
            panic!("Expected EnvironmentError for unbound symbol");
        }
    }

    #[test]
    fn test_eval_non_symbol_procedure() {
        let mut env = Environment::new();

        // Test non-symbol as procedure
        let non_symbol_proc = Expression::list(vec![
            Expression::atom(Value::number(42.0)),
            Expression::atom(Value::number(1.0)),
        ]);
        let result = eval(non_symbol_proc, &mut env);
        assert!(result.is_err());
        if let Err(Error::RuntimeError(msg)) = result {
            assert!(msg.contains("is not a procedure, got number"));
        } else {
            panic!("Expected RuntimeError for non-symbol procedure");
        }
    }

    #[test]
    fn test_eval_nested_expressions() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(10.0));
        env.define_str("y", Value::number(5.0));

        // Test that eval properly handles nested Expression structures
        let nested = Expression::list(vec![
            Expression::atom(Value::symbol("*")),
            Expression::list(vec![
                Expression::atom(Value::symbol("+")),
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::symbol("y")),
            ]),
            Expression::atom(Value::number(2.0)),
        ]);
        let result = eval(nested, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 30.0);
    }

    // Note: Comprehensive comparison testing is now done in integration tests.
    // eval.rs focuses on core evaluation behavior with Expression objects.

    #[test]
    fn test_eval_quote_atom() {
        let mut env = Environment::new();

        // Quoted atom should return the atom value without evaluation
        let quoted_symbol = Expression::quote(Expression::atom(Value::symbol("undefined")));
        let result = eval(quoted_symbol, &mut env).unwrap();

        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "undefined");

        // Even though "undefined" is not bound in environment,
        // it should not error because it's quoted
    }

    #[test]
    fn test_eval_quote_list() {
        let mut env = Environment::new();

        // Quoted list should return the list structure without evaluation
        let quoted_list = Expression::quote(Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]));

        let result = eval(quoted_list, &mut env).unwrap();

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
        let result = expression_to_value(number_expr).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        // Test list conversion
        let list_expr = Expression::list(vec![
            Expression::atom(Value::symbol("test")),
            Expression::atom(Value::boolean(true)),
        ]);

        let result = expression_to_value(list_expr).unwrap();
        assert!(result.is_list());

        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "test");
        assert!(list.get(1).unwrap().as_boolean().unwrap());
    }

    #[test]
    fn test_nested_environment_symbol_lookup() {
        let mut global = Environment::new();
        global.define_str("global_var", Value::number(100.0));

        let mut local = Environment::new_scope(&global);
        local.define_str("local_var", Value::string("local"));

        // Test lookup from local environment
        let local_expr = Expression::atom(Value::symbol("local_var"));
        let result = eval(local_expr, &mut local).unwrap();
        assert_eq!(result.as_string().unwrap(), "local");

        // Test lookup from parent environment
        let global_expr = Expression::atom(Value::symbol("global_var"));
        let result = eval(global_expr, &mut local).unwrap();
        assert_eq!(result.as_number().unwrap(), 100.0);
    }

    #[test]
    fn test_eval_list_values() {
        let mut env = Environment::new();

        // Test that list values are self-evaluating
        let list_value = Value::List(List::from(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]));

        let expr = Expression::atom(list_value.clone());
        let result = eval(expr, &mut env).unwrap();

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
        let mut env = Environment::new();

        let number_expr = Expression::atom(Value::number(42.0));
        let result = eval(number_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        let string_expr = Expression::atom(Value::string("hello"));
        let result = eval(string_expr, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");

        // Test eval function with symbol lookup
        let mut env_with_binding = Environment::new();
        env_with_binding.define_str("x", Value::number(10.0));

        let symbol_expr = Expression::atom(Value::symbol("x"));
        let result = eval(symbol_expr, &mut env_with_binding).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);

        // Test eval function with quoted expressions
        let quoted_expr = Expression::quote(Expression::atom(Value::symbol("unbound")));
        let result = eval(quoted_expr, &mut env).unwrap();
        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "unbound");

        // Test eval function with empty lists
        let empty_list = Expression::list(vec![]);
        let result = eval(empty_list, &mut env).unwrap();
        assert!(result.is_list());
        assert!(result.as_list().unwrap().is_empty());
    }

    #[test]
    fn test_eval_if_true_condition() {
        let mut env = Environment::new();

        // Test (if #t "yes" "no") -> "yes"
        let if_expr = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("yes")),
            Expression::atom(Value::string("no")),
        ]);

        let result = eval(if_expr, &mut env).unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "yes");
    }

    #[test]
    fn test_eval_if_false_condition() {
        let mut env = Environment::new();

        // Test (if #f "yes" "no") -> "no"
        let if_expr = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::atom(Value::boolean(false)),
            Expression::atom(Value::string("yes")),
            Expression::atom(Value::string("no")),
        ]);

        let result = eval(if_expr, &mut env).unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "no");
    }

    #[test]
    fn test_eval_if_truthiness() {
        let mut env = Environment::new();

        // Test that all non-#f values are truthy
        let test_cases = vec![
            (Value::number(0.0), "zero"),             // 0 is truthy in Scheme
            (Value::string(""), "empty-string"),      // Empty string is truthy
            (Value::List(List::new()), "empty-list"), // Empty list is truthy
            (Value::Nil, "nil"),                      // Nil is truthy
            (Value::number(42.0), "number"),          // Positive number is truthy
            (Value::string("hello"), "string"),       // Non-empty string is truthy
        ];

        for (condition, expected) in test_cases {
            let if_expr = Expression::list(vec![
                Expression::atom(Value::symbol("if")),
                Expression::atom(condition),
                Expression::atom(Value::string(expected)),
                Expression::atom(Value::string("false")),
            ]);

            let result = eval(if_expr, &mut env).unwrap();
            assert!(result.is_string());
            assert_eq!(result.as_string().unwrap(), expected);
        }
    }

    #[test]
    fn test_eval_if_with_expressions() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(5.0));

        // Test (if (> x 0) "positive" "non-positive") -> "positive"
        let if_expr = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::list(vec![
                Expression::atom(Value::symbol(">")),
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::number(0.0)),
            ]),
            Expression::atom(Value::string("positive")),
            Expression::atom(Value::string("non-positive")),
        ]);

        let result = eval(if_expr.clone(), &mut env).unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "positive");

        // Change x to -3 and test again
        env.define_str("x", Value::number(-3.0));

        let result2 = eval(if_expr, &mut env).unwrap();
        assert!(result2.is_string());
        assert_eq!(result2.as_string().unwrap(), "non-positive");
    }

    #[test]
    fn test_eval_if_nested() {
        let mut env = Environment::new();

        // Test nested if: (if #t (if #f "inner-no" "inner-yes") "outer-no")
        let nested_if = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::atom(Value::boolean(true)),
            Expression::list(vec![
                Expression::atom(Value::symbol("if")),
                Expression::atom(Value::boolean(false)),
                Expression::atom(Value::string("inner-no")),
                Expression::atom(Value::string("inner-yes")),
            ]),
            Expression::atom(Value::string("outer-no")),
        ]);

        let result = eval(nested_if, &mut env).unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "inner-yes");
    }

    #[test]
    fn test_eval_if_arity_errors() {
        let mut env = Environment::new();

        // Test if with too few arguments
        let if_too_few = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::atom(Value::boolean(true)),
        ]);

        let result = eval(if_too_few, &mut env);
        assert!(result.is_err());

        // Test if with too many arguments
        let if_too_many = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("then")),
            Expression::atom(Value::string("else")),
            Expression::atom(Value::string("extra")),
        ]);

        let result = eval(if_too_many, &mut env);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_if_evaluation_order() {
        let mut env = Environment::new();
        env.define_str("counter", Value::number(0.0));

        // This would test that only the condition and the chosen branch are evaluated
        // For now, we'll test a simpler case since we don't have side effects yet
        let if_expr = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::list(vec![
                Expression::atom(Value::symbol("=")),
                Expression::atom(Value::number(1.0)),
                Expression::atom(Value::number(1.0)),
            ]),
            Expression::atom(Value::string("equal")),
            Expression::atom(Value::string("not-equal")),
        ]);

        let result = eval(if_expr, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "equal");
    }

    // Note: Basic list operations are now comprehensively tested in integration tests.
    // These tests focus on eval-specific functionality with Expression objects.

    #[test]
    fn test_eval_list_operations_core() {
        let mut env = Environment::new();

        // Test that list operations work at the eval level with Expression objects
        let car_expr = Expression::list(vec![
            Expression::atom(Value::symbol("car")),
            Expression::quote(Expression::list(vec![
                Expression::atom(Value::symbol("test")),
                Expression::atom(Value::number(42.0)),
            ])),
        ]);
        let result = eval(car_expr, &mut env).unwrap();
        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "test");

        // Test error propagation in eval for list operations
        let car_empty_expr = Expression::list(vec![
            Expression::atom(Value::symbol("car")),
            Expression::quote(Expression::list(vec![])),
        ]);
        let result = eval(car_empty_expr, &mut env);
        assert!(result.is_err());
    }
}
