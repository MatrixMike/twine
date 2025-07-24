//! Twine Scheme Interpreter
//!
//! A minimalist Scheme interpreter written in Rust that implements a functional
//! subset of R7RS-small Scheme with fiber-based concurrency and strict immutability.

pub mod error;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod types;

// Re-export error types for convenience
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    // Helper function for end-to-end evaluation testing
    fn eval_source(
        source: &str,
        env: &crate::runtime::Environment,
    ) -> crate::Result<crate::types::Value> {
        use crate::parser::Parser;
        use crate::runtime::eval::eval;

        let mut parser = Parser::new(source.to_string())?;
        let expr = parser.parse_expression()?.expr;
        eval(&expr, env)
    }

    #[test]
    fn test_integration_self_evaluating_atoms() {
        use crate::runtime::environment::Environment;

        let env = Environment::new();

        // Numbers
        let result = eval_source("42", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        let result = eval_source("-17.5", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), -17.5);

        // Booleans
        let result = eval_source("#t", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = eval_source("#f", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);

        // Strings
        let result = eval_source("\"hello world\"", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "hello world");

        let result = eval_source("\"\"", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "");
    }

    #[test]
    fn test_integration_symbol_lookup() {
        use crate::runtime::environment::Environment;
        use crate::types::Value;

        let mut env = Environment::new();
        env.define_str("x", Value::number(42.0));
        env.define_str("name", Value::string("Scheme"));
        env.define_str("flag", Value::boolean(true));

        // Symbol lookup
        let result = eval_source("x", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        let result = eval_source("name", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "Scheme");

        let result = eval_source("flag", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Unbound symbol should error
        let result = eval_source("undefined", &env);
        assert!(result.is_err());
    }

    #[test]
    fn test_integration_arithmetic_operations() {
        use crate::runtime::environment::Environment;
        use crate::types::Value;

        let mut env = Environment::new();
        env.define_str("x", Value::number(10.0));
        env.define_str("y", Value::number(3.0));

        // Basic arithmetic
        let result = eval_source("(+ 1 2 3)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 6.0);

        let result = eval_source("(- 10 3)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 7.0);

        let result = eval_source("(* 4 5)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 20.0);

        let result = eval_source("(/ 15 3)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);

        // With variables
        let result = eval_source("(+ x y)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 13.0);

        let result = eval_source("(* x y)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 30.0);

        // Nested arithmetic
        let result = eval_source("(+ (* 2 3) (- 10 5))", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 11.0);
    }

    #[test]
    fn test_integration_comparison_operations() {
        use crate::runtime::environment::Environment;
        use crate::types::Value;

        let mut env = Environment::new();
        env.define_str("a", Value::number(5.0));
        env.define_str("b", Value::number(3.0));

        // Equality
        let result = eval_source("(= 5 5)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = eval_source("(= a 5)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = eval_source("(= a b)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);

        // Less than
        let result = eval_source("(< 3 5)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = eval_source("(< b a)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Greater than
        let result = eval_source("(> 5 3)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = eval_source("(> a b)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Less than or equal
        let result = eval_source("(<= 3 3)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = eval_source("(<= b a)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Greater than or equal
        let result = eval_source("(>= 5 5)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let result = eval_source("(>= a b)", &env).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_integration_conditional_expressions() {
        use crate::runtime::environment::Environment;
        use crate::types::Value;

        let mut env = Environment::new();
        env.define_str("x", Value::number(5.0));
        env.define_str("y", Value::number(-3.0));

        // Basic conditionals
        let result = eval_source("(if #t \"yes\" \"no\")", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "yes");

        let result = eval_source("(if #f \"yes\" \"no\")", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "no");

        // Conditionals with expressions
        let result = eval_source("(if (> x 0) \"positive\" \"non-positive\")", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "positive");

        let result = eval_source("(if (> y 0) \"positive\" \"non-positive\")", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "non-positive");

        // Scheme truthiness (only #f is false)
        let result = eval_source("(if 0 \"truthy\" \"falsy\")", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "truthy");

        let result = eval_source("(if \"\" \"truthy\" \"falsy\")", &env).unwrap();
        assert_eq!(result.as_string().unwrap(), "truthy");

        // Nested conditionals
        let result = eval_source("(if #t (if #f 1 2) 3)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);

        let result = eval_source("(if #f (if #t 1 2) 3)", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);
    }

    #[test]
    fn test_integration_quoted_expressions() {
        use crate::runtime::environment::Environment;

        let env = Environment::new();

        // Quoted atoms
        let result = eval_source("'x", &env).unwrap();
        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "x");

        let result = eval_source("'42", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 42.0);

        // Quoted lists (should not be evaluated)
        let result = eval_source("'(+ 1 2)", &env).unwrap();
        assert!(result.is_list());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "+");
        assert_eq!(list.get(1).unwrap().as_number().unwrap(), 1.0);
        assert_eq!(list.get(2).unwrap().as_number().unwrap(), 2.0);

        // Nested quotes
        let result = eval_source("''x", &env).unwrap();
        assert!(result.is_symbol());
        assert_eq!(result.as_symbol().unwrap(), "x");
    }

    #[test]
    fn test_integration_complex_expressions() {
        use crate::runtime::environment::Environment;
        use crate::types::Value;

        let mut env = Environment::new();
        env.define_str("a", Value::number(10.0));
        env.define_str("b", Value::number(5.0));
        env.define_str("c", Value::number(2.0));

        // Complex arithmetic with conditionals
        let result = eval_source("(if (> a b) (+ a (* b c)) (- a b))", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 20.0); // 10 + (5 * 2)

        // Nested comparisons
        let result = eval_source("(if (and #t (> a 0)) \"positive\" \"not positive\")", &env);
        // This should fail because we haven't implemented 'and' yet
        assert!(result.is_err());

        // But this should work
        let result = eval_source(
            "(if (> a 0) (if (> b 0) \"both positive\" \"mixed\") \"negative\")",
            &env,
        )
        .unwrap();
        assert_eq!(result.as_string().unwrap(), "both positive");

        // Complex nested expression
        let result = eval_source("(+ (* (if (> a b) a b) c) (- a c))", &env).unwrap();
        assert_eq!(result.as_number().unwrap(), 28.0); // (10 * 2) + (10 - 2)
    }

    #[test]
    fn test_integration_error_cases() {
        use crate::runtime::environment::Environment;

        let env = Environment::new();

        // Unbound symbol
        let result = eval_source("undefined-symbol", &env);
        assert!(result.is_err());

        // Type error in arithmetic
        let result = eval_source("(+ 1 \"not a number\")", &env);
        assert!(result.is_err());

        // Wrong arity for comparison operations (need exactly 2 args)
        let result = eval_source("(= 1)", &env);
        assert!(result.is_err());

        let result = eval_source("(if #t)", &env);
        assert!(result.is_err());

        // Division by zero
        let result = eval_source("(/ 1 0)", &env);
        assert!(result.is_err());

        // Unknown procedure
        let result = eval_source("(unknown-proc 1 2)", &env);
        assert!(result.is_err());
    }
}
