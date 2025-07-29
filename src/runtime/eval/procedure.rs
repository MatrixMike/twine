//! Procedure evaluation and calling logic
//!
//! This module handles the evaluation and calling of Scheme procedures,
//! including both builtin procedures and user-defined lambda procedures.
//! It implements tail call optimization for recursive lambda calls.

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::runtime::{Environment, special_forms};
use crate::types::{Lambda, Procedure, Value};
use std::sync::Arc;

use super::eval;

/// Call a procedure with the given argument expressions
///
/// This function evaluates the argument expressions and then calls the appropriate
/// procedure type (builtin or lambda). For lambda procedures, it includes arity
/// checking and tail call optimization.
pub fn call_procedure(
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
pub fn call_lambda(lambda: Arc<Lambda>, args: Vec<Value>) -> Result<Value> {
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

/// Evaluate the last expression in a lambda body with tail call optimization check
///
/// This function checks if the last expression is a procedure call in tail position
/// and returns either a tail call optimization opportunity or the evaluated value.
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
///
/// This function evaluates each expression in the argument list and returns
/// a vector of the resulting values in the same order.
pub fn eval_arguments(exprs: &[Arc<Expression>], env: &mut Environment) -> Result<Vec<Value>> {
    let mut args = Vec::with_capacity(exprs.len());
    for expr in exprs {
        args.push(eval(Arc::clone(expr), env)?);
    }
    Ok(args)
}
