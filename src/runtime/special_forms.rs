//! Special forms for the Twine Scheme runtime
//!
//! This module contains all special forms organized by category.
//! Special forms have unique evaluation rules that differ from normal
//! procedure calls (arguments are not automatically evaluated).

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::{Symbol, Value};

/// Dispatch a special form evaluation
///
/// This function serves as the central dispatch point for all special forms.
/// It returns `Some(result)` if the symbol corresponds to a special form,
/// or `None` if the symbol is not a special form.
///
/// # Arguments
/// * `identifier` - The special form name as a Symbol
/// * `args` - The unevaluated argument expressions (special forms control evaluation)
/// * `env` - The environment for evaluation context
///
/// # Returns
/// * `Option<Result<Value>>` - Some(result) for special forms, None for unknown identifiers
pub fn dispatch(
    identifier: &Symbol,
    args: &[Expression],
    env: &Environment,
) -> Option<Result<Value>> {
    match identifier.as_str() {
        // Conditional expressions
        "if" => Some(eval_if(args, env)),

        // Return None for unknown identifiers - not a special form
        _ => None,
    }
}

/// Evaluate an if special form
///
/// Syntax: (if <test> <consequent> <alternative>)
/// - Evaluates <test>
/// - If the result is truthy (anything except #f), evaluates and returns <consequent>
/// - If the result is falsy (#f), evaluates and returns <alternative>
fn eval_if(args: &[Expression], env: &Environment) -> Result<Value> {
    // if requires exactly 3 arguments: test, consequent, alternative
    if args.len() != 3 {
        return Err(crate::Error::runtime_error(&format!(
            "if: expected 3 arguments, got {}",
            args.len()
        )));
    }

    let test_expr = &args[0];
    let consequent_expr = &args[1];
    let alternative_expr = &args[2];

    // Evaluate the test expression
    let test_value = eval(test_expr, env)?;

    // In Scheme, only #f is false, everything else is true
    if is_truthy(&test_value) {
        eval(consequent_expr, env)
    } else {
        eval(alternative_expr, env)
    }
}

/// Determine if a value is truthy in Scheme semantics
///
/// In Scheme, only #f is false. Everything else, including 0, empty lists,
/// empty strings, and nil, is considered true.
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Boolean(false) => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Expression;
    use crate::runtime::environment::Environment;
    use crate::types::Value;

    #[test]
    fn test_dispatch_if_special_form() {
        let env = Environment::new();

        // Test if special form dispatch
        let args = vec![
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("yes")),
            Expression::atom(Value::string("no")),
        ];

        let result = dispatch(&Symbol::new("if"), &args, &env).unwrap().unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");
    }

    #[test]
    fn test_dispatch_unknown_special_form() {
        let env = Environment::new();
        let args = vec![Expression::atom(Value::number(1.0))];

        // Unknown special form should return None
        let result = dispatch(&Symbol::new("unknown-form"), &args, &env);
        assert!(result.is_none());

        // Test with future special form that doesn't exist yet
        let result = dispatch(&Symbol::new("define"), &args, &env);
        assert!(result.is_none());
    }

    #[test]
    fn test_dispatch_error_propagation() {
        let env = Environment::new();

        // Test that errors from special forms are properly propagated
        // if with wrong arity should error
        let args = vec![Expression::atom(Value::boolean(true))]; // Missing consequent and alternative

        let result = dispatch(&Symbol::new("if"), &args, &env).unwrap();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("if: expected 3 arguments, got 1")
        );
    }

    #[test]
    fn test_is_truthy_function() {
        // Test the is_truthy helper function directly
        assert!(is_truthy(&Value::boolean(true)));
        assert!(!is_truthy(&Value::boolean(false)));
        assert!(is_truthy(&Value::number(0.0)));
        assert!(is_truthy(&Value::number(42.0)));
        assert!(is_truthy(&Value::string("")));
        assert!(is_truthy(&Value::string("hello")));
        assert!(is_truthy(&Value::List(crate::types::List::new())));
        assert!(is_truthy(&Value::Nil));
        assert!(is_truthy(&Value::symbol("test")));
    }

    #[test]
    fn test_eval_if_comprehensive() {
        let env = Environment::new();

        // Test true condition
        let args_true = vec![
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("consequent")),
            Expression::atom(Value::string("alternative")),
        ];
        let result = eval_if(&args_true, &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "consequent");

        // Test false condition
        let args_false = vec![
            Expression::atom(Value::boolean(false)),
            Expression::atom(Value::string("consequent")),
            Expression::atom(Value::string("alternative")),
        ];
        let result = eval_if(&args_false, &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "alternative");

        // Test arity error
        let args_wrong = vec![Expression::atom(Value::boolean(true))];
        let result = eval_if(&args_wrong, &env);
        assert!(result.is_err());
    }
}
