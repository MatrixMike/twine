//! Control flow special forms
//!
//! This module implements control flow special forms that control the order
//! of expression evaluation, including conditionals and sequencing constructs.
//!
//! ## Current Special Forms
//! - `if`: Conditional expressions
//! - `begin`: Expression sequencing
//! - `and`: Logical AND with short-circuit evaluation
//! - `or`: Logical OR with short-circuit evaluation
//!
//! ## Future Special Forms (planned)
//! - `cond`: Multi-way conditionals
//! - `case`: Pattern matching conditionals
//! - `when`/`unless`: Single-branch conditionals

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::utils::eval_sequence;
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::Value;
use std::sync::Arc;

/// Evaluate an if special form
///
/// Syntax: (if `test` `consequent` `alternative`)
/// - Evaluates `test`
/// - If the result is truthy (anything except #f), evaluates and returns `consequent`
/// - If the result is falsy (#f), evaluates and returns `alternative`
pub fn eval_if(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    // if requires exactly 3 arguments: test, consequent, alternative
    if args.len() != 3 {
        return Err(crate::Error::arity_error("if", 3, args.len()));
    }

    let test_expr = Arc::clone(&args[0]);
    let consequent_expr = Arc::clone(&args[1]);
    let alternative_expr = Arc::clone(&args[2]);

    // Evaluate the test expression
    let test_result = eval(test_expr, env)?;

    // In Scheme, only #f is false, everything else is true
    if test_result.is_truthy() {
        eval(consequent_expr, env)
    } else {
        eval(alternative_expr, env)
    }
}

/// Evaluate a begin special form
///
/// Syntax: (begin `expr1` `expr2` ... `exprN`)
/// - Evaluates all expressions in sequence
/// - Returns the value of the last expression
/// - If no expressions are provided, returns Nil
pub fn eval_begin(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    eval_sequence(args, env)
}

/// Evaluate an and special form
///
/// Syntax: (and `expr1` `expr2` ... `exprN`)
/// - Evaluates expressions left to right
/// - If any expression evaluates to #f, returns #f immediately (short-circuit)
/// - If all expressions are truthy, returns the value of the last expression
/// - If no expressions are provided, returns #t
pub fn eval_and(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::boolean(true));
    }

    let mut result = Value::boolean(true);

    // Evaluate expressions left to right, short-circuiting on #f
    for expr in args {
        result = eval(Arc::clone(expr), env)?;
        if !result.is_truthy() {
            return Ok(Value::boolean(false));
        }
    }

    // Return the last result if all were truthy
    Ok(result)
}

/// Evaluate an or special form
///
/// Syntax: (or `expr1` `expr2` ... `exprN`)
/// - Evaluates expressions left to right
/// - If any expression evaluates to a truthy value, returns that value immediately (short-circuit)
/// - If all expressions are falsy (#f), returns #f
/// - If no expressions are provided, returns #f
pub fn eval_or(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::boolean(false));
    }

    // Evaluate expressions left to right, short-circuiting on truthy values
    for expr in args {
        let result = eval(Arc::clone(expr), env)?;
        if result.is_truthy() {
            return Ok(result);
        }
    }

    // All expressions were falsy
    Ok(Value::boolean(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Expression;
    use crate::runtime::environment::Environment;
    use crate::types::Value;

    #[test]
    fn test_eval_if_comprehensive() {
        let mut env = Environment::new();

        // Test true condition
        let args_true = vec![
            Expression::arc_atom(Value::boolean(true)),
            Expression::arc_atom(Value::string("consequent")),
            Expression::arc_atom(Value::string("alternative")),
        ];
        let result = eval_if(&args_true, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "consequent");

        // Test false condition
        let args_false = vec![
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::string("consequent")),
            Expression::arc_atom(Value::string("alternative")),
        ];
        let result = eval_if(&args_false, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "alternative");

        // Test arity error
        let args_wrong = vec![Expression::arc_atom(Value::boolean(true))];
        let result = eval_if(&args_wrong, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("if: expected 3 arguments, got 1")
        );
    }

    #[test]
    fn test_eval_if_truthiness() {
        let mut env = Environment::new();

        // Test various truthy values
        let truthy_values = vec![
            Value::boolean(true),
            Value::number(0.0),
            Value::number(42.0),
            Value::string(""),
            Value::string("hello"),
            Value::List(crate::types::List::new()),
            Value::Nil,
        ];

        for truthy in truthy_values {
            let args = vec![
                Expression::arc_atom(truthy),
                Expression::arc_atom(Value::string("true-branch")),
                Expression::arc_atom(Value::string("false-branch")),
            ];
            let result = eval_if(&args, &mut env).unwrap();
            assert_eq!(result.as_string().unwrap(), "true-branch");
        }

        // Test false value
        let args_false = vec![
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::string("true-branch")),
            Expression::arc_atom(Value::string("false-branch")),
        ];
        let result = eval_if(&args_false, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "false-branch");
    }

    #[test]
    fn test_eval_begin_empty() {
        let mut env = Environment::new();

        // Empty begin should return Nil
        let args: Vec<Arc<Expression>> = vec![];
        let result = eval_begin(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_begin_single_expression() {
        let mut env = Environment::new();

        // Single expression should return its value
        let args = vec![Expression::arc_atom(Value::number(42.0))];
        let result = eval_begin(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_begin_multiple_expressions() {
        let mut env = Environment::new();

        // Multiple expressions should return the value of the last one
        let args = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::string("middle")),
            Expression::arc_atom(Value::boolean(true)),
        ];
        let result = eval_begin(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_eval_begin_with_side_effects() {
        let mut env = Environment::new();

        // Test that all expressions are evaluated (side effects would occur)
        // We can't easily test side effects here, but we can test that
        // variables are defined in sequence
        let args = vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("define")),
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(10.0)),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("define")),
                Expression::arc_atom(Value::symbol("y")),
                Expression::arc_atom(Value::number(20.0)),
            ]),
            Expression::arc_atom(Value::symbol("x")),
        ];

        let result = eval_begin(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(10.0));

        // Verify both variables were defined
        use crate::types::Symbol;
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(10.0));
        assert_eq!(env.lookup(&Symbol::new("y")).unwrap(), Value::number(20.0));
    }

    #[test]
    fn test_eval_and_empty() {
        let mut env = Environment::new();

        // Empty and should return #t
        let args: Vec<Arc<Expression>> = vec![];
        let result = eval_and(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_eval_and_all_truthy() {
        let mut env = Environment::new();

        // All truthy values should return the last value
        let args = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::string("hello")),
            Expression::arc_atom(Value::number(42.0)),
        ];
        let result = eval_and(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_and_with_false() {
        let mut env = Environment::new();

        // Should short-circuit on first #f
        let args = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::string("should-not-evaluate")),
        ];
        let result = eval_and(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_eval_and_single_false() {
        let mut env = Environment::new();

        // Single false value
        let args = vec![Expression::arc_atom(Value::boolean(false))];
        let result = eval_and(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_eval_or_empty() {
        let mut env = Environment::new();

        // Empty or should return #f
        let args: Vec<Arc<Expression>> = vec![];
        let result = eval_or(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_eval_or_with_truthy() {
        let mut env = Environment::new();

        // Should short-circuit on first truthy value
        let args = vec![
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::number(42.0)),
            Expression::arc_atom(Value::string("should-not-evaluate")),
        ];
        let result = eval_or(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_or_all_false() {
        let mut env = Environment::new();

        // All false values should return #f
        let args = vec![
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::boolean(false)),
        ];
        let result = eval_or(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_eval_or_single_truthy() {
        let mut env = Environment::new();

        // Single truthy value
        let args = vec![Expression::arc_atom(Value::string("hello"))];
        let result = eval_or(&args, &mut env).unwrap();
        assert_eq!(result, Value::string("hello"));
    }
}
