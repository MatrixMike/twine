//! Control flow special forms
//!
//! This module implements control flow special forms that control the order
//! of expression evaluation, including conditionals and sequencing constructs.
//!
//! ## Current Special Forms
//! - `if`: Conditional expressions
//!
//! ## Future Special Forms (planned)
//! - `cond`: Multi-way conditionals
//! - `case`: Pattern matching conditionals
//! - `when`/`unless`: Single-branch conditionals
//! - `begin`: Expression sequencing

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::Value;

/// Evaluate an if special form
///
/// Syntax: (if <test> <consequent> <alternative>)
/// - Evaluates <test>
/// - If the result is truthy (anything except #f), evaluates and returns <consequent>
/// - If the result is falsy (#f), evaluates and returns <alternative>
pub fn eval_if(args: &[Expression], env: &mut Environment) -> Result<Value> {
    // if requires exactly 3 arguments: test, consequent, alternative
    if args.len() != 3 {
        return Err(crate::Error::arity_error("if", 3, args.len()));
    }

    let test_expr = &args[0];
    let consequent_expr = &args[1];
    let alternative_expr = &args[2];

    // Evaluate the test expression
    let test_value = eval(test_expr, env)?;

    // In Scheme, only #f is false, everything else is true
    if test_value.is_truthy() {
        eval(consequent_expr, env)
    } else {
        eval(alternative_expr, env)
    }
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
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("consequent")),
            Expression::atom(Value::string("alternative")),
        ];
        let result = eval_if(&args_true, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "consequent");

        // Test false condition
        let args_false = vec![
            Expression::atom(Value::boolean(false)),
            Expression::atom(Value::string("consequent")),
            Expression::atom(Value::string("alternative")),
        ];
        let result = eval_if(&args_false, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "alternative");

        // Test arity error
        let args_wrong = vec![Expression::atom(Value::boolean(true))];
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
                Expression::atom(truthy),
                Expression::atom(Value::string("true-branch")),
                Expression::atom(Value::string("false-branch")),
            ];
            let result = eval_if(&args, &mut env).unwrap();
            assert_eq!(result.as_string().unwrap(), "true-branch");
        }

        // Test false value
        let args_false = vec![
            Expression::atom(Value::boolean(false)),
            Expression::atom(Value::string("true-branch")),
            Expression::atom(Value::string("false-branch")),
        ];
        let result = eval_if(&args_false, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "false-branch");
    }
}
