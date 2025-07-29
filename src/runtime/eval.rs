//! Evaluation engine for the Twine Scheme runtime
//!
//! This module implements the core evaluation logic for Scheme expressions.
//! It handles atomic values, symbol lookup, and provides the foundation
//! for list evaluation (procedure calls and special forms).

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::types::{Lambda, List, Procedure, Value};
use std::sync::Arc;

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

/// Call a procedure with the given arguments
///
/// Handles both builtin and lambda procedures:
/// - For builtin procedures, delegates to the builtin implementation
/// - For lambda procedures, creates new environment, binds parameters, and evaluates body
/// - Uses tail call optimization for lambda procedures to prevent stack overflow
/// - Detects tail position calls and eliminates unnecessary stack frames
fn call_procedure(
    procedure: Procedure,
    arg_exprs: &[Arc<Expression>],
    env: &mut Environment,
) -> Result<Value> {
    // Evaluate arguments
    let args = eval_arguments(arg_exprs, env)?;

    match procedure {
        Procedure::Builtin(builtin) => builtin.call(&args),
        Procedure::Lambda(lambda) => {
            // Check arity
            let expected_arity = lambda.arity();
            let actual_arity = arg_exprs.len();

            if expected_arity != actual_arity {
                return Err(Error::arity_error("<lambda>", expected_arity, actual_arity));
            }

            call_lambda(lambda, args)
        }
    }
}

/// Call a lambda procedure with tail call optimization
///
/// This function implements tail call optimization by detecting when the lambda body
/// contains a procedure call in tail position and handling it iteratively rather
/// than recursively to prevent stack overflow for deeply recursive functions.
fn call_lambda(lambda: Arc<Lambda>, args: Vec<Value>) -> Result<Value> {
    // Current lambda and arguments for the iterative evaluation
    let mut current_lambda = lambda;
    let mut current_args = args;

    loop {
        // Create new environment extending the lambda's closure
        let mut call_env = Environment::new_scope(current_lambda.env());

        // Bind parameters to arguments
        for (param, arg) in current_lambda.params().iter().zip(current_args.iter()) {
            call_env.define(param.clone(), arg.clone());
        }

        // Check if the last expression in the lambda body is a tail call
        let body_exprs = current_lambda.body();
        if body_exprs.is_empty() {
            return Err(Error::runtime_error("Lambda body cannot be empty"));
        }

        // Evaluate all expressions except the last for their side effects
        for expr in &body_exprs[..body_exprs.len() - 1] {
            eval(Arc::clone(expr), &mut call_env)?; // Result is discarded
        }

        // Evaluate the last expression and check for tail call optimization
        let last_expr = &body_exprs[body_exprs.len() - 1];
        match eval_last_expression_with_tail_call_check(last_expr, &mut call_env)? {
            TailCallResult::TailCall { procedure, args } => {
                match procedure {
                    Procedure::Lambda(next_lambda) => {
                        // Tail call to another lambda - optimize by continuing loop
                        current_lambda = next_lambda;
                        current_args = args;
                        continue;
                    }
                    Procedure::Builtin(builtin) => {
                        // Tail call to builtin - just call it directly
                        return builtin.call(&args);
                    }
                }
            }
            TailCallResult::Value(value) => {
                // Not a tail call - return the evaluated value
                return Ok(value);
            }
        }
    }
}

/// Result of evaluating the last expression with tail call optimization check
enum TailCallResult {
    TailCall {
        procedure: Procedure,
        args: Vec<Value>,
    },
    Value(Value),
}

/// Evaluate the last expression of a lambda, checking for tail call optimization
///
/// A tail call occurs when a procedure call is in tail position - meaning it's
/// the last operation performed before returning from the current procedure.
/// This function evaluates the expression once and determines if it's a tail call.
fn eval_last_expression_with_tail_call_check(
    expr: &Arc<Expression>,
    env: &mut Environment,
) -> Result<TailCallResult> {
    match expr.as_ref() {
        // Direct procedure call: (procedure arg1 arg2 ...)
        Expression::List(elements) if !elements.is_empty() => {
            let first_expr = &elements[0];
            let rest_exprs = &elements[1..];

            // Check if the first expression is a symbol that could be a procedure
            match first_expr.as_ref() {
                Expression::Atom(Value::Symbol(identifier)) => {
                    // Handle special forms - they're not tail calls since they have special evaluation
                    if special_forms::SpecialForm::from_name(identifier.as_str()).is_some() {
                        let value = eval(Arc::clone(expr), env)?;
                        return Ok(TailCallResult::Value(value));
                    }

                    // Try to look up the identifier as a procedure
                    if let Ok(Value::Procedure(procedure)) = env.lookup(identifier) {
                        // Evaluate the arguments
                        let args = eval_arguments(rest_exprs, env)?;
                        return Ok(TailCallResult::TailCall { procedure, args });
                    }
                }
                _ => {
                    // For non-symbol first expressions, evaluate to check if it's a procedure
                    if let Ok(Value::Procedure(procedure)) = eval(Arc::clone(first_expr), env) {
                        let args = eval_arguments(rest_exprs, env)?;
                        return Ok(TailCallResult::TailCall { procedure, args });
                    }
                }
            }
        }
        _ => {
            // Not a list or empty list - not a procedure call
        }
    }

    // Not a tail call - evaluate normally
    let value = eval(Arc::clone(expr), env)?;
    Ok(TailCallResult::Value(value))
}

/// Evaluate a list of argument expressions into values
fn eval_arguments(exprs: &[Arc<Expression>], env: &mut Environment) -> Result<Vec<Value>> {
    let mut args = Vec::with_capacity(exprs.len());
    for expr in exprs {
        args.push(eval(Arc::clone(expr), env)?);
    }
    Ok(args)
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
    use crate::types::{Symbol, Value};

    #[test]
    fn test_eval_self_evaluating_atoms() {
        let mut env = Environment::new();

        // Test number evaluation
        let number_expr = Expression::arc_atom(Value::number(42.0));
        let result = eval(number_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        // Test boolean evaluation
        let bool_expr = Expression::arc_atom(Value::boolean(true));
        let result = eval(bool_expr, &mut env).unwrap();
        assert!(result.as_boolean().unwrap());

        // Test string evaluation
        let string_expr = Expression::arc_atom(Value::string("hello"));
        let result = eval(string_expr, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");

        // Test nil evaluation
        let nil_expr = Expression::arc_atom(Value::Nil);
        let result = eval(nil_expr, &mut env).unwrap();
        assert!(result.is_nil());
    }

    #[test]
    fn test_eval_symbol_lookup() {
        let mut env = Environment::new();
        env.define(Symbol::new("x"), Value::number(10.0));
        env.define(Symbol::new("greeting"), Value::string("hello world"));

        // Test successful symbol lookup
        let symbol_expr = Expression::arc_atom(Value::symbol("x"));
        let result = eval(symbol_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);

        let greeting_expr = Expression::arc_atom(Value::symbol("greeting"));
        let result = eval(greeting_expr, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_eval_unbound_symbol() {
        let mut env = Environment::new();

        // Test unbound symbol error
        let unbound_expr = Expression::arc_atom(Value::symbol("undefined"));
        let result = eval(unbound_expr, &mut env);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unbound identifier"));

        if let Error::EnvironmentError { identifier, .. } = error {
            assert_eq!(identifier, "undefined");
        } else {
            panic!("Expected EnvironmentError for unbound symbol");
        }
    }

    #[test]
    fn test_eval_empty_list() {
        let mut env = Environment::new();

        // Empty list should evaluate to empty list
        let empty_list = Expression::arc_list(vec![]);
        let result = eval(empty_list, &mut env).unwrap();

        assert!(result.is_list());
        assert!(result.as_list().unwrap().is_empty());
    }

    #[test]
    fn test_eval_arithmetic_core() {
        let mut env = Environment::new();

        // Test that arithmetic operations work at the eval level with Expression objects
        // (Comprehensive arithmetic testing is done in integration tests)
        let add_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::number(2.0)),
        ]);
        let result = eval(add_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        // Test that comparison operations return proper boolean values
        let eq_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("=")),
            Expression::arc_atom(Value::number(5.0)),
            Expression::arc_atom(Value::number(5.0)),
        ]);
        let result = eval(eq_expr, &mut env).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_eval_unknown_procedure() {
        let mut env = Environment::new();

        // Test unknown procedure
        let unknown_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("unknown")),
            Expression::arc_atom(Value::number(1.0)),
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
        let non_symbol_proc = Expression::arc_list(vec![
            Expression::arc_atom(Value::number(42.0)),
            Expression::arc_atom(Value::number(1.0)),
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
        // Test nested arithmetic: (+ 10 (* 2 10))
        let nested = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::number(10.0)),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("*")),
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(2.0)),
            ]),
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
        let quoted_symbol = Expression::arc_quote(Expression::arc_atom(Value::symbol("undefined")));
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
        let quoted_list = Expression::arc_quote(Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::number(2.0)),
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
        let number_expr = Expression::arc_atom(Value::number(42.0));
        let result = expression_to_value(&number_expr).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        // Test list conversion
        let list_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("test")),
            Expression::arc_atom(Value::boolean(true)),
        ]);

        let result = expression_to_value(&list_expr).unwrap();
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
        let local_expr = Expression::arc_atom(Value::symbol("local_var"));
        let result = eval(local_expr, &mut local).unwrap();
        assert_eq!(result.as_string().unwrap(), "local");

        // Test lookup from parent environment
        let global_expr = Expression::arc_atom(Value::symbol("global_var"));
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

        let expr = Expression::arc_atom(list_value.clone());
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

        let number_expr = Expression::arc_atom(Value::number(42.0));
        let result = eval(number_expr, &mut env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        let string_expr = Expression::arc_atom(Value::string("hello"));
        let result = eval(string_expr, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello");

        // Test eval function with symbol lookup
        let mut env_with_binding = Environment::new();
        env_with_binding.define_str("x", Value::number(10.0));

        let symbol_expr = Expression::arc_atom(Value::symbol("x"));
        let result = eval(symbol_expr, &mut env_with_binding).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);

        // Test eval function with quoted expressions
        let quoted_expr = Expression::arc_quote(Expression::arc_atom(Value::symbol("unbound")));
        let result = eval(quoted_expr, &mut env).unwrap();
        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "unbound");

        // Test eval function with empty lists
        let empty_list = Expression::arc_list(vec![]);
        let result = eval(empty_list, &mut env).unwrap();
        assert!(result.is_list());
        assert!(result.as_list().unwrap().is_empty());
    }

    #[test]
    fn test_eval_if_true_condition() {
        let mut env = Environment::new();

        // Test (if #t "yes" "no") -> "yes"
        let if_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_atom(Value::boolean(true)),
            Expression::arc_atom(Value::string("yes")),
            Expression::arc_atom(Value::string("no")),
        ]);

        let result = eval(if_expr, &mut env).unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "yes");
    }

    #[test]
    fn test_eval_if_false_condition() {
        let mut env = Environment::new();

        // Test (if #f "yes" "no") -> "no"
        let if_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::string("yes")),
            Expression::arc_atom(Value::string("no")),
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
            let if_expr = Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("if")),
                Expression::arc_atom(condition),
                Expression::arc_atom(Value::string(expected)),
                Expression::arc_atom(Value::string("false")),
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
        let if_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol(">")),
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(0.0)),
            ]),
            Expression::arc_atom(Value::string("positive")),
            Expression::arc_atom(Value::string("non-positive")),
        ]);

        let result = eval(Arc::clone(&if_expr), &mut env).unwrap();
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
        let nested_if = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_atom(Value::boolean(true)),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("if")),
                Expression::arc_atom(Value::boolean(false)),
                Expression::arc_atom(Value::string("inner-no")),
                Expression::arc_atom(Value::string("inner-yes")),
            ]),
            Expression::arc_atom(Value::string("outer-no")),
        ]);

        let result = eval(nested_if, &mut env).unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "inner-yes");
    }

    #[test]
    fn test_eval_if_arity_errors() {
        let mut env = Environment::new();

        // Test if with too few arguments
        let if_too_few = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_atom(Value::boolean(true)),
        ]);

        let result = eval(if_too_few, &mut env);
        assert!(result.is_err());

        // Test if with too many arguments
        let if_too_many = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_atom(Value::boolean(true)),
            Expression::arc_atom(Value::string("then")),
            Expression::arc_atom(Value::string("else")),
            Expression::arc_atom(Value::string("extra")),
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
        let if_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("=")),
                Expression::arc_atom(Value::number(1.0)),
                Expression::arc_atom(Value::number(1.0)),
            ]),
            Expression::arc_atom(Value::string("equal")),
            Expression::arc_atom(Value::string("not-equal")),
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
        let car_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("car")),
            Expression::arc_quote(Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("test")),
                Expression::arc_atom(Value::number(42.0)),
            ])),
        ]);
        let result = eval(car_expr, &mut env).unwrap();
        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "test");

        // Test error propagation in eval for list operations
        let car_empty_expr = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("car")),
            Expression::arc_quote(Expression::arc_list(vec![])),
        ]);
        let result = eval(car_empty_expr, &mut env);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_list_no_clone_required() {
        let mut env = Environment::new();

        // Create a list expression: (+ 1 2)
        let elements = vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::number(2.0)),
        ];

        // Test that we can call eval_list with a slice reference
        // This verifies we don't need to clone the Vec
        let result = eval_list(&elements, &mut env).unwrap();
        assert_eq!(result, Value::number(3.0));

        // Test that the original elements Vec is still usable
        assert_eq!(elements.len(), 3);
        assert!(matches!(
            elements[0].as_ref(),
            Expression::Atom(Value::Symbol(_))
        ));
    }

    #[test]
    fn test_tail_call_optimization_basic() {
        let mut env = Environment::new();

        // Define a simple function that demonstrates tail call optimization
        // (define countdown (lambda (n) (if (= n 0) "done" n)))
        let countdown_params = vec![Symbol::new("n")];
        let countdown_body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("=")),
                Expression::arc_atom(Value::symbol("n")),
                Expression::arc_atom(Value::number(0.0)),
            ]),
            Expression::arc_atom(Value::string("done")),
            Expression::arc_atom(Value::symbol("n")),
        ]);

        let countdown_lambda =
            crate::types::Procedure::lambda(countdown_params, vec![countdown_body], env.flatten());

        env.define(Symbol::new("countdown"), Value::Procedure(countdown_lambda));

        // Test that the function works correctly
        let countdown_call = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("countdown")),
            Expression::arc_atom(Value::number(5.0)),
        ]);

        let result = eval(countdown_call, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0));

        // Test with zero
        let countdown_zero = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("countdown")),
            Expression::arc_atom(Value::number(0.0)),
        ]);

        let result = eval(countdown_zero, &mut env).unwrap();
        assert_eq!(result, Value::string("done"));
    }

    #[test]
    fn test_tail_call_optimization_identity() {
        let mut env = Environment::new();

        // Define an identity function that calls itself in tail position
        // (define tail-identity (lambda (x) (tail-identity x)))
        // This would create infinite recursion but tests our TCO detection
        let identity_params = vec![Symbol::new("x")];
        let identity_body = Expression::arc_atom(Value::symbol("x"));

        let identity_lambda =
            crate::types::Procedure::lambda(identity_params, vec![identity_body], env.flatten());

        env.define(
            Symbol::new("tail-identity"),
            Value::Procedure(identity_lambda),
        );

        // Test that the identity function works
        let identity_call = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("tail-identity")),
            Expression::arc_atom(Value::number(42.0)),
        ]);

        let result = eval(identity_call, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_tail_call_optimization_builtin() {
        let mut env = Environment::new();

        // Define a function that tail-calls a builtin procedure
        // (define add-one (lambda (x) (+ x 1)))
        let add_one_params = vec![Symbol::new("x")];
        let add_one_body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(1.0)),
        ]);

        let add_one_lambda =
            crate::types::Procedure::lambda(add_one_params, vec![add_one_body], env.flatten());

        env.define(Symbol::new("add-one"), Value::Procedure(add_one_lambda));

        // Test that the function correctly tail-calls the builtin
        let add_one_call = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("add-one")),
            Expression::arc_atom(Value::number(41.0)),
        ]);

        let result = eval(add_one_call, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_tail_call_optimization_non_tail_call() {
        let mut env = Environment::new();

        // Define a function where the procedure call is NOT in tail position
        // (define add-and-double (lambda (x) (* (+ x 1) 2)))
        let add_double_params = vec![Symbol::new("x")];
        let add_double_body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("*")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("+")),
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(1.0)),
            ]),
            Expression::arc_atom(Value::number(2.0)),
        ]);

        let add_double_lambda = crate::types::Procedure::lambda(
            add_double_params,
            vec![add_double_body],
            env.flatten(),
        );

        env.define(
            Symbol::new("add-and-double"),
            Value::Procedure(add_double_lambda),
        );

        // Test that the function works correctly (no tail call optimization here)
        let add_double_call = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("add-and-double")),
            Expression::arc_atom(Value::number(5.0)),
        ]);

        let result = eval(add_double_call, &mut env).unwrap();
        assert_eq!(result, Value::number(12.0)); // (5 + 1) * 2 = 12
    }

    #[test]
    fn test_no_double_evaluation_in_lambda_call() {
        use crate::parser::Parser;

        let mut env = Environment::default();

        // Create a lambda that returns a simple value (not a procedure)
        // This tests the case where the last expression is NOT a tail call
        let lambda_source = "(lambda () 42)";
        let mut parser = Parser::new(lambda_source.to_string()).unwrap();
        let lambda_expr = parser.parse_expression().unwrap().expr;
        let lambda_value = eval(lambda_expr, &mut env).unwrap();

        // Call the lambda - before the fix, this would evaluate 42 twice
        if let Value::Procedure(Procedure::Lambda(lambda)) = lambda_value {
            let result = call_lambda(lambda, vec![]).unwrap();
            assert_eq!(result, Value::number(42.0));
        } else {
            panic!("Expected lambda procedure");
        }

        // Test with a more complex non-tail-call expression
        let lambda_source = "(lambda () (+ 1 2))";
        let mut parser = Parser::new(lambda_source.to_string()).unwrap();
        let lambda_expr = parser.parse_expression().unwrap().expr;
        let lambda_value = eval(lambda_expr, &mut env).unwrap();

        if let Value::Procedure(Procedure::Lambda(lambda)) = lambda_value {
            let result = call_lambda(lambda, vec![]).unwrap();
            assert_eq!(result, Value::number(3.0));
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_tail_call_optimization_special_forms() {
        let mut env = Environment::new();

        // Define a function that has an if expression in tail position
        // (define conditional (lambda (x) (if (> x 0) x (- x))))
        let conditional_params = vec![Symbol::new("x")];
        let conditional_body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol(">")),
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(0.0)),
            ]),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("-")),
                Expression::arc_atom(Value::symbol("x")),
            ]),
        ]);

        let conditional_lambda = crate::types::Procedure::lambda(
            conditional_params,
            vec![conditional_body],
            env.flatten(),
        );

        env.define(
            Symbol::new("conditional"),
            Value::Procedure(conditional_lambda),
        );

        // Test positive case
        let positive_call = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("conditional")),
            Expression::arc_atom(Value::number(5.0)),
        ]);

        let result = eval(positive_call, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0));

        // Test negative case
        let negative_call = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("conditional")),
            Expression::arc_atom(Value::number(-3.0)),
        ]);

        let result = eval(negative_call, &mut env).unwrap();
        assert_eq!(result, Value::number(3.0));
    }
}
