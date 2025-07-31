//! Special forms for the Twine Scheme runtime
//!
//! This module contains all special forms organized by category.
//! Special forms have unique evaluation rules that differ from normal
//! procedure calls (arguments are not automatically evaluated).

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::environment::Environment;
use crate::types::Value;
use std::sync::Arc;

/// Enumeration of all special forms
///
/// Each variant represents a specific special form, eliminating the need
/// to store both function pointers and names. This provides type safety and
/// eliminates redundancy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialForm {
    // Control flow expressions
    If,
    Begin,

    // Logical operators
    And,
    Or,

    // Binding and definition forms
    Define,
    Let,
    LetStar,
    Letrec,
    LetrecStar,

    // Function creation
    Lambda,

    // Concurrency forms
    Async,
}

impl SpecialForm {
    /// Get the display name for this special form
    pub fn name(self) -> &'static str {
        match self {
            SpecialForm::If => "if",
            SpecialForm::Begin => "begin",
            SpecialForm::And => "and",
            SpecialForm::Or => "or",
            SpecialForm::Define => "define",
            SpecialForm::Let => "let",
            SpecialForm::LetStar => "let*",
            SpecialForm::Letrec => "letrec",
            SpecialForm::LetrecStar => "letrec*",
            SpecialForm::Lambda => "lambda",
            SpecialForm::Async => "async",
        }
    }

    /// Execute this special form with the given arguments
    pub fn call(self, args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
        match self {
            SpecialForm::If => control_flow::eval_if(args, env),
            SpecialForm::Begin => control_flow::eval_begin(args, env),
            SpecialForm::And => control_flow::eval_and(args, env),
            SpecialForm::Or => control_flow::eval_or(args, env),
            SpecialForm::Define => binding::eval_define(args, env),
            SpecialForm::Lambda => lambda::eval_lambda(args, env),
            SpecialForm::Let => binding::eval_let(args, env),
            SpecialForm::LetStar => binding::eval_let_star(args, env),
            SpecialForm::Letrec => binding::eval_letrec(args, env),
            SpecialForm::LetrecStar => binding::eval_letrec_star(args, env),
            SpecialForm::Async => concurrency::eval_async(args, env),
        }
    }

    /// Parse a special form name into its corresponding SpecialForm
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "if" => Some(SpecialForm::If),
            "begin" => Some(SpecialForm::Begin),
            "and" => Some(SpecialForm::And),
            "or" => Some(SpecialForm::Or),
            "define" => Some(SpecialForm::Define),
            "let" => Some(SpecialForm::Let),
            "let*" => Some(SpecialForm::LetStar),
            "letrec" => Some(SpecialForm::Letrec),
            "letrec*" => Some(SpecialForm::LetrecStar),
            "lambda" => Some(SpecialForm::Lambda),
            "async" => Some(SpecialForm::Async),
            _ => None,
        }
    }
}

pub mod binding;
pub mod concurrency;
pub mod control_flow;
pub mod lambda;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Expression;
    use crate::runtime::environment::Environment;
    use crate::types::{Symbol, Value};

    #[test]
    fn test_if_special_form_direct() {
        let mut env = Environment::new();

        // Test if special form direct access
        let args = vec![
            Expression::arc_atom(Value::boolean(true)),
            Expression::arc_atom(Value::string("yes")),
            Expression::arc_atom(Value::string("no")),
        ];

        let special_form = SpecialForm::from_name("if").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");
    }

    #[test]
    fn test_begin_special_form_direct() {
        let mut env = Environment::new();

        // Test begin special form direct access with multiple expressions
        let args = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::string("middle")),
            Expression::arc_atom(Value::boolean(true)),
        ];

        let special_form = SpecialForm::from_name("begin").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test empty begin
        let empty_args: Vec<Arc<Expression>> = vec![];
        let result = special_form.call(&empty_args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_define_special_form_direct() {
        let mut env = Environment::new();

        // Test define special form direct access
        let args = vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ];

        let special_form = SpecialForm::from_name("define").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));
    }

    #[test]
    fn test_async_special_form_direct() {
        let mut env = Environment::new();

        // Test async special form direct access - currently returns not implemented error
        let args = vec![Expression::arc_atom(Value::number(42.0))];

        let special_form = SpecialForm::from_name("async").unwrap();
        let result = special_form.call(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not yet implemented")
        );
    }

    #[test]
    fn test_unknown_special_form() {
        // Unknown special form should return None
        let result = SpecialForm::from_name("unknown-form");
        assert!(result.is_none());

        // Test with unknown special form that doesn't exist
        let result = SpecialForm::from_name("unknown-future-form");
        assert!(result.is_none());
    }

    #[test]
    fn test_let_special_form_direct() {
        let mut env = Environment::new();

        // Test let special form direct access: (let ((x 42)) x)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ])]);
        let body = Expression::arc_atom(Value::symbol("x"));
        let args = vec![bindings, body];

        let special_form = SpecialForm::from_name("let").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_lambda_special_form_direct() {
        let mut env = Environment::new();

        // Test lambda special form direct access: (lambda (x) x)
        let params = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]);
        let body = Expression::arc_atom(Value::symbol("x"));
        let args = vec![params, body];

        let special_form = SpecialForm::from_name("lambda").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(1));
            assert_eq!(proc.name(), "<lambda>");
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_special_form_error_propagation() {
        let mut env = Environment::new();

        // Test that errors from special forms are properly propagated
        // if with wrong arity should error
        let args = vec![Expression::arc_atom(Value::boolean(true))]; // Missing consequent and alternative

        let special_form = SpecialForm::from_name("if").unwrap();
        let result = special_form.call(&args, &mut env);
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
        assert_eq!(SpecialForm::Begin.name(), "begin");
        assert_eq!(SpecialForm::And.name(), "and");
        assert_eq!(SpecialForm::Or.name(), "or");
        assert_eq!(SpecialForm::Define.name(), "define");
        assert_eq!(SpecialForm::Let.name(), "let");
        assert_eq!(SpecialForm::LetStar.name(), "let*");
        assert_eq!(SpecialForm::Letrec.name(), "letrec");
        assert_eq!(SpecialForm::LetrecStar.name(), "letrec*");
        assert_eq!(SpecialForm::Lambda.name(), "lambda");
        assert_eq!(SpecialForm::Async.name(), "async");
    }

    #[test]
    fn test_special_form_from_name() {
        assert_eq!(SpecialForm::from_name("if"), Some(SpecialForm::If));
        assert_eq!(SpecialForm::from_name("begin"), Some(SpecialForm::Begin));
        assert_eq!(SpecialForm::from_name("and"), Some(SpecialForm::And));
        assert_eq!(SpecialForm::from_name("or"), Some(SpecialForm::Or));
        assert_eq!(SpecialForm::from_name("define"), Some(SpecialForm::Define));
        assert_eq!(SpecialForm::from_name("let"), Some(SpecialForm::Let));
        assert_eq!(SpecialForm::from_name("let*"), Some(SpecialForm::LetStar));
        assert_eq!(SpecialForm::from_name("letrec"), Some(SpecialForm::Letrec));
        assert_eq!(
            SpecialForm::from_name("letrec*"),
            Some(SpecialForm::LetrecStar)
        );
        assert_eq!(SpecialForm::from_name("lambda"), Some(SpecialForm::Lambda));
        assert_eq!(SpecialForm::from_name("async"), Some(SpecialForm::Async));

        // Test unknown names
        assert_eq!(SpecialForm::from_name("unknown"), None);
        assert_eq!(SpecialForm::from_name(""), None);
        assert_eq!(SpecialForm::from_name("unknown-form"), None);
    }

    #[test]
    fn test_special_form_call() {
        let mut env = Environment::new();

        // Test if special form
        let args = vec![
            Expression::arc_atom(Value::boolean(true)),
            Expression::arc_atom(Value::string("yes")),
            Expression::arc_atom(Value::string("no")),
        ];
        let result = SpecialForm::If.call(&args, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");

        // Test begin special form
        let args = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::string("middle")),
            Expression::arc_atom(Value::boolean(true)),
        ];
        let result = SpecialForm::Begin.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test define special form
        let args = vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ];
        let result = SpecialForm::Define.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));

        // Test let special form
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("y")),
            Expression::arc_atom(Value::number(100.0)),
        ])]);
        let body = Expression::arc_atom(Value::symbol("y"));
        let args = vec![bindings, body];
        let result = SpecialForm::Let.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(100.0));

        // Test lambda special form
        let params = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]);
        let body = Expression::arc_atom(Value::symbol("x"));
        let args = vec![params, body];
        let result = SpecialForm::Lambda.call(&args, &mut env).unwrap();
        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(1));
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_special_form_call_errors() {
        let mut env = Environment::new();

        // Test error propagation for invalid arguments
        let args = vec![Expression::arc_atom(Value::boolean(true))]; // Missing consequent and alternative
        let result = SpecialForm::If.call(&args, &mut env);
        assert!(result.is_err());

        // Test async not implemented error
        let args = vec![Expression::arc_atom(Value::number(42.0))];
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
        set.insert(SpecialForm::Begin);
        set.insert(SpecialForm::Define);
        set.insert(SpecialForm::If); // Duplicate should not increase size

        assert_eq!(set.len(), 3);
        assert!(set.contains(&SpecialForm::If));
        assert!(set.contains(&SpecialForm::Begin));
        assert!(set.contains(&SpecialForm::Define));
        assert!(!set.contains(&SpecialForm::Let));
        assert!(!set.contains(&SpecialForm::Lambda));
    }

    #[test]
    fn test_special_form_debug() {
        let debug_output = format!("{:?}", SpecialForm::If);
        assert_eq!(debug_output, "If");

        let debug_output = format!("{:?}", SpecialForm::Begin);
        assert_eq!(debug_output, "Begin");

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
    fn test_and_special_form_direct() {
        let mut env = Environment::new();

        // Test and special form with all truthy values
        let args = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::string("hello")),
            Expression::arc_atom(Value::number(42.0)),
        ];
        let special_form = SpecialForm::from_name("and").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));

        // Test and with false value (short-circuit)
        let args = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::string("not-evaluated")),
        ];
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(false));

        // Test empty and
        let args: Vec<Arc<Expression>> = vec![];
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_or_special_form_direct() {
        let mut env = Environment::new();

        // Test or special form with truthy value (short-circuit)
        let args = vec![
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::number(42.0)),
            Expression::arc_atom(Value::string("not-evaluated")),
        ];
        let special_form = SpecialForm::from_name("or").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));

        // Test or with all false values
        let args = vec![
            Expression::arc_atom(Value::boolean(false)),
            Expression::arc_atom(Value::boolean(false)),
        ];
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(false));

        // Test empty or
        let args: Vec<Arc<Expression>> = vec![];
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_special_form_enum_usage() {
        let mut env = Environment::new();

        // Test that SpecialForm enum works correctly
        let args = vec![
            Expression::arc_atom(Value::boolean(true)),
            Expression::arc_atom(Value::string("yes")),
            Expression::arc_atom(Value::string("no")),
        ];
        let special_form = SpecialForm::from_name("if").unwrap();
        let result = special_form.call(&args, &mut env).unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");

        // Test that unknown special forms return None
        let result = SpecialForm::from_name("unknown-form");
        assert!(result.is_none());
    }

    #[test]
    fn test_let_star_special_form_direct() {
        let mut env = Environment::new();

        // Test let* special form: (let* ((x 5) (y x)) y)
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(5.0)),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("y")),
                Expression::arc_atom(Value::symbol("x")),
            ]),
        ]);
        let body = Expression::arc_atom(Value::symbol("y"));
        let args = vec![bindings, body];

        let result = SpecialForm::LetStar.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0));
    }

    #[test]
    fn test_letrec_star_special_form_direct() {
        let mut env = Environment::new();

        // Test letrec* special form with factorial
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("fact")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("lambda")),
                Expression::arc_list(vec![Expression::arc_atom(Value::symbol("n"))]),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("if")),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("=")),
                        Expression::arc_atom(Value::symbol("n")),
                        Expression::arc_atom(Value::number(0.0)),
                    ]),
                    Expression::arc_atom(Value::number(1.0)),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("*")),
                        Expression::arc_atom(Value::symbol("n")),
                        Expression::arc_list(vec![
                            Expression::arc_atom(Value::symbol("fact")),
                            Expression::arc_list(vec![
                                Expression::arc_atom(Value::symbol("-")),
                                Expression::arc_atom(Value::symbol("n")),
                                Expression::arc_atom(Value::number(1.0)),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ])]);
        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("fact")),
            Expression::arc_atom(Value::number(3.0)),
        ]);
        let args = vec![bindings, body];

        let result = SpecialForm::LetrecStar.call(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(6.0)); // 3! = 6
    }
}
