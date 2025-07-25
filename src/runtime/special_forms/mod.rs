//! Special forms for the Twine Scheme runtime
//!
//! This module contains all special forms organized by category.
//! Special forms have unique evaluation rules that differ from normal
//! procedure calls (arguments are not automatically evaluated).

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::environment::Environment;
use crate::types::{Symbol, Value};

pub mod binding;
pub mod control_flow;

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
    env: &mut Environment,
) -> Option<Result<Value>> {
    match identifier.as_str() {
        // Control flow expressions
        "if" => Some(control_flow::eval_if(args, env)),

        // Binding and definition forms
        "define" => Some(binding::eval_define(args, env)),
        "let" => Some(binding::eval_let(args, env)),

        // Return None for unknown identifiers - not a special form
        _ => None,
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
        let mut env = Environment::new();

        // Test if special form dispatch
        let args = vec![
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("yes")),
            Expression::atom(Value::string("no")),
        ];

        let result = dispatch(&Symbol::new("if"), &args, &mut env)
            .unwrap()
            .unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");
    }

    #[test]
    fn test_dispatch_define_special_form() {
        let mut env = Environment::new();

        // Test define special form dispatch
        let args = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ];

        let result = dispatch(&Symbol::new("define"), &args, &mut env)
            .unwrap()
            .unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));
    }

    #[test]
    fn test_dispatch_unknown_special_form() {
        let mut env = Environment::new();
        let args = vec![Expression::atom(Value::number(1.0))];

        // Unknown special form should return None
        let result = dispatch(&Symbol::new("unknown-form"), &args, &mut env);
        assert!(result.is_none());

        // Test with future special form that doesn't exist yet
        let result = dispatch(&Symbol::new("lambda"), &args, &mut env);
        assert!(result.is_none());
    }

    #[test]
    fn test_dispatch_let_special_form() {
        let mut env = Environment::new();

        // Test let special form dispatch: (let ((x 42)) x)
        let bindings = Expression::List(vec![Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ])]);
        let body = Expression::atom(Value::symbol("x"));
        let args = vec![bindings, body];

        let result = dispatch(&Symbol::new("let"), &args, &mut env)
            .unwrap()
            .unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_dispatch_error_propagation() {
        let mut env = Environment::new();

        // Test that errors from special forms are properly propagated
        // if with wrong arity should error
        let args = vec![Expression::atom(Value::boolean(true))]; // Missing consequent and alternative

        let result = dispatch(&Symbol::new("if"), &args, &mut env).unwrap();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("if: expected 3 arguments, got 1")
        );
    }
}
