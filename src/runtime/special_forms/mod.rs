//! Special forms for the Twine Scheme runtime
//!
//! This module contains all special forms organized by category.
//! Special forms have unique evaluation rules that differ from normal
//! procedure calls (arguments are not automatically evaluated).

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::environment::Environment;
use crate::types::Value;

/// Enumeration of all special forms
///
/// Each variant represents a specific special form, eliminating the need
/// to store both function pointers and names. This provides type safety and
/// eliminates redundancy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialForm {
    // Control flow expressions
    If,

    // Binding and definition forms
    Define,
    Let,

    // Concurrency forms
    Async,
}

impl SpecialForm {
    /// Get the display name for this special form
    pub fn name(self) -> &'static str {
        match self {
            SpecialForm::If => "if",
            SpecialForm::Define => "define",
            SpecialForm::Let => "let",
            SpecialForm::Async => "async",
        }
    }

    /// Execute this special form with the given arguments
    pub fn call(self, args: &[Expression], env: &mut Environment) -> Result<Value> {
        match self {
            SpecialForm::If => control_flow::eval_if(args, env),
            SpecialForm::Define => binding::eval_define(args, env),
            SpecialForm::Let => binding::eval_let(args, env),
            SpecialForm::Async => concurrency::eval_async(args, env),
        }
    }

    /// Parse a special form name into its corresponding SpecialForm
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "if" => Some(SpecialForm::If),
            "define" => Some(SpecialForm::Define),
            "let" => Some(SpecialForm::Let),
            "async" => Some(SpecialForm::Async),
            _ => None,
        }
    }
}

pub mod binding;
pub mod concurrency;
pub mod control_flow;

/// Dispatch a special form evaluation
///
/// This function serves as the central dispatch point for all special forms.
/// It returns `Some(result)` if the name corresponds to a special form, or
/// `None` if the name is not a special form.
///
/// # Arguments
/// * `name` - The special form name
/// * `args` - The unevaluated argument expressions (special forms control evaluation)
/// * `env` - The environment for evaluation context
///
/// # Returns
/// * `Option<Result<Value>>` - Some(result) for special forms, None for unknown identifiers
pub fn dispatch(name: &str, args: &[Expression], env: &mut Environment) -> Option<Result<Value>> {
    // Try to parse as a special form
    SpecialForm::from_name(name).map(|special_form| special_form.call(args, env))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Expression;
    use crate::runtime::environment::Environment;
    use crate::types::{Symbol, Value};

    #[test]
    fn test_dispatch_if_special_form() {
        let mut env = Environment::new();

        // Test if special form dispatch
        let args = vec![
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("yes")),
            Expression::atom(Value::string("no")),
        ];

        let result = dispatch("if", &args, &mut env).unwrap().unwrap();
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

        let result = dispatch("define", &args, &mut env).unwrap().unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));
    }

    #[test]
    fn test_dispatch_async_special_form() {
        let mut env = Environment::new();

        // Test async special form dispatch - currently returns not implemented error
        let args = vec![Expression::atom(Value::number(42.0))];

        let result = dispatch("async", &args, &mut env).unwrap();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_dispatch_unknown_special_form() {
        let mut env = Environment::new();
        let args = vec![Expression::atom(Value::number(1.0))];

        // Unknown special form should return None
        let result = dispatch("unknown-form", &args, &mut env);
        assert!(result.is_none());

        // Test with future special form that doesn't exist yet
        let result = dispatch("lambda", &args, &mut env);
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

        let result = dispatch("let", &args, &mut env).unwrap().unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_dispatch_error_propagation() {
        let mut env = Environment::new();

        // Test that errors from special forms are properly propagated
        // if with wrong arity should error
        let args = vec![Expression::atom(Value::boolean(true))]; // Missing consequent and alternative

        let result = dispatch("if", &args, &mut env).unwrap();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("if: expected 3 arguments, got 1")
        );
    }

    #[test]
    fn test_special_form_name() {
        assert_eq!(SpecialForm::If.name(), "if");
        assert_eq!(SpecialForm::Define.name(), "define");
        assert_eq!(SpecialForm::Let.name(), "let");
        assert_eq!(SpecialForm::Async.name(), "async");
    }

    #[test]
    fn test_special_form_from_name() {
        assert_eq!(SpecialForm::from_name("if"), Some(SpecialForm::If));
        assert_eq!(SpecialForm::from_name("define"), Some(SpecialForm::Define));
        assert_eq!(SpecialForm::from_name("let"), Some(SpecialForm::Let));
        assert_eq!(SpecialForm::from_name("async"), Some(SpecialForm::Async));

        // Test unknown names
        assert_eq!(SpecialForm::from_name("unknown"), None);
        assert_eq!(SpecialForm::from_name(""), None);
        assert_eq!(SpecialForm::from_name("lambda"), None);
    }

    #[test]
    fn test_special_form_call() {
        let mut env = Environment::new();

        // Test if special form
        let args = vec![
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("yes")),
            Expression::atom(Value::string("no")),
        ];
        let result = SpecialForm::If.call(&args, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");

        // Test define special form
        let args = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ];
        let result = SpecialForm::Define.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));

        // Test let special form
        let bindings = Expression::List(vec![Expression::List(vec![
            Expression::atom(Value::symbol("y")),
            Expression::atom(Value::number(100.0)),
        ])]);
        let body = Expression::atom(Value::symbol("y"));
        let args = vec![bindings, body];
        let result = SpecialForm::Let.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(100.0));
    }

    #[test]
    fn test_special_form_call_errors() {
        let mut env = Environment::new();

        // Test error propagation for invalid arguments
        let args = vec![Expression::atom(Value::boolean(true))]; // Missing consequent and alternative
        let result = SpecialForm::If.call(&args, &mut env);
        assert!(result.is_err());

        // Test async not implemented error
        let args = vec![Expression::atom(Value::number(42.0))];
        let result = SpecialForm::Async.call(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_special_form_equality_and_hash() {
        use std::collections::HashSet;

        // Test equality
        assert_eq!(SpecialForm::If, SpecialForm::If);
        assert_ne!(SpecialForm::If, SpecialForm::Define);

        // Test that they can be used in HashSet (implements Hash + Eq)
        let mut set = HashSet::new();
        set.insert(SpecialForm::If);
        set.insert(SpecialForm::Define);
        set.insert(SpecialForm::If); // Duplicate should not increase size

        assert_eq!(set.len(), 2);
        assert!(set.contains(&SpecialForm::If));
        assert!(set.contains(&SpecialForm::Define));
        assert!(!set.contains(&SpecialForm::Let));
    }

    #[test]
    fn test_special_form_debug() {
        let debug_output = format!("{:?}", SpecialForm::If);
        assert_eq!(debug_output, "If");

        let debug_output = format!("{:?}", SpecialForm::Async);
        assert_eq!(debug_output, "Async");
    }

    #[test]
    fn test_special_form_copy_clone() {
        let original = SpecialForm::If;
        let copied = original; // Copy trait
        let cloned = original; // Clone trait

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
        assert_eq!(copied, cloned);
    }

    #[test]
    fn test_dispatch_uses_special_form_enum() {
        let mut env = Environment::new();

        // Test that dispatch properly uses the SpecialForm enum internally
        let args = vec![
            Expression::atom(Value::boolean(true)),
            Expression::atom(Value::string("yes")),
            Expression::atom(Value::string("no")),
        ];
        let result = dispatch("if", &args, &mut env).unwrap().unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");

        // Test that unknown special forms return None
        let result = dispatch("unknown-form", &args, &mut env);
        assert!(result.is_none());
    }
}
