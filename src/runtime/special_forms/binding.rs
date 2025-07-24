//! Binding and definition special forms
//!
//! This module implements binding special forms like `define` and `let`.

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::Value;

/// Evaluate a define special form
///
/// Syntax:
/// - Variable definition: (define <identifier> <expression>)
/// - Function definition: (define (<identifier> <param>...) <body>...)
///
/// For variable definition:
/// - Evaluates <expression> and binds the result to <identifier> in the current environment
/// - Returns Nil
///
/// For function definition (syntactic sugar):
/// - (define (name param1 param2) body1 body2...)
/// - Equivalent to: (define name (lambda (param1 param2) body1 body2...))
pub fn eval_define(args: &[Expression], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::Error::arity_error("define", 1, 0));
    }

    match &args[0] {
        // Variable definition: (define identifier expression)
        Expression::Atom(Value::Symbol(identifier)) => {
            if args.len() != 2 {
                return Err(crate::Error::arity_error("define", 2, args.len()));
            }

            // Evaluate the expression and bind it to the identifier
            let value = eval(&args[1], env)?;
            env.define(identifier.clone(), value);
            Ok(Value::Nil)
        }

        // Function definition: (define (name param...) body...)
        Expression::List(elements) => {
            if elements.is_empty() {
                return Err(crate::Error::runtime_error(
                    "define: function definition requires non-empty parameter list",
                ));
            }

            // Extract function name and parameters
            let function_name = match &elements[0] {
                Expression::Atom(Value::Symbol(name)) => name.clone(),
                _ => {
                    return Err(crate::Error::runtime_error(
                        "define: function name must be a symbol",
                    ));
                }
            };

            // Extract parameters - all must be symbols
            let mut parameters = Vec::new();
            for param_expr in &elements[1..] {
                match param_expr {
                    Expression::Atom(Value::Symbol(param)) => {
                        parameters.push(param.clone());
                    }
                    _ => {
                        return Err(crate::Error::runtime_error(
                            "define: function parameters must be symbols",
                        ));
                    }
                }
            }

            // Function body is everything after the parameter list
            if args.len() < 2 {
                return Err(crate::Error::runtime_error(
                    "define: function definition requires at least one body expression",
                ));
            }

            let body_expressions: Vec<Expression> = args[1..].to_vec();

            // Create a lambda expression: (lambda (param...) body...)
            let mut lambda_args = vec![Expression::List(
                parameters
                    .into_iter()
                    .map(|p| Expression::atom(Value::Symbol(p)))
                    .collect(),
            )];
            lambda_args.extend(body_expressions);

            // For now, we'll store a placeholder since lambda isn't implemented yet
            // This will be updated when T3.1.2 implements lambda
            // TODO: Replace with actual lambda creation when lambda is implemented
            let placeholder_value =
                Value::string(&format!("<function:{}>", function_name.as_str()));

            env.define(function_name, placeholder_value);
            Ok(Value::Nil)
        }

        _ => Err(crate::Error::runtime_error(
            "define: first argument must be a symbol or parameter list",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Expression;
    use crate::runtime::environment::Environment;
    use crate::types::{Symbol, Value};

    #[test]
    fn test_eval_define_variable() {
        let mut env = Environment::new();

        // Test simple variable definition
        let args = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));
    }

    #[test]
    fn test_eval_define_variable_with_expression() {
        let mut env = Environment::new();

        // Define a variable first
        env.define(Symbol::new("y"), Value::number(10.0));

        // Test defining with an expression that references another variable
        let args = vec![
            Expression::atom(Value::symbol("z")),
            Expression::atom(Value::symbol("y")), // Should evaluate to 10.0
        ];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created with the evaluated value
        assert_eq!(env.lookup(&Symbol::new("z")).unwrap(), Value::number(10.0));
    }

    #[test]
    fn test_eval_define_function_syntax() {
        let mut env = Environment::new();

        // Test function definition syntax: (define (square x) (* x x))
        let function_def = Expression::List(vec![
            Expression::atom(Value::symbol("square")),
            Expression::atom(Value::symbol("x")),
        ]);

        let body = Expression::List(vec![
            Expression::atom(Value::symbol("*")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("x")),
        ]);

        let args = vec![function_def, body];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the function binding was created (placeholder for now)
        let binding = env.lookup(&Symbol::new("square")).unwrap();
        assert!(binding.as_string().unwrap().contains("function:square"));
    }

    #[test]
    fn test_eval_define_function_multiple_params() {
        let mut env = Environment::new();

        // Test function with multiple parameters: (define (add x y) (+ x y))
        let function_def = Expression::List(vec![
            Expression::atom(Value::symbol("add")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
        ]);

        let body = Expression::List(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
        ]);

        let args = vec![function_def, body];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the function binding was created
        let binding = env.lookup(&Symbol::new("add")).unwrap();
        assert!(binding.as_string().unwrap().contains("function:add"));
    }

    #[test]
    fn test_eval_define_function_multiple_body_expressions() {
        let mut env = Environment::new();

        // Test function with multiple body expressions
        let function_def = Expression::List(vec![Expression::atom(Value::symbol("test-fn"))]);

        let body1 = Expression::atom(Value::number(1.0));
        let body2 = Expression::atom(Value::number(2.0));

        let args = vec![function_def, body1, body2];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the function binding was created
        let binding = env.lookup(&Symbol::new("test-fn")).unwrap();
        assert!(binding.as_string().unwrap().contains("function:test-fn"));
    }

    #[test]
    fn test_eval_define_errors() {
        let mut env = Environment::new();

        // Test no arguments
        let result = eval_define(&[], &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("define: expected 1 argument, got 0")
        );

        // Test variable definition with wrong arity
        let args = vec![Expression::atom(Value::symbol("x"))];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("define: expected 2 arguments")
        );

        // Test function definition with non-symbol name
        let function_def = Expression::List(vec![
            Expression::atom(Value::number(42.0)), // Not a symbol
        ]);
        let args = vec![function_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("function name must be a symbol")
        );

        // Test function definition with non-symbol parameter
        let function_def = Expression::List(vec![
            Expression::atom(Value::symbol("fn")),
            Expression::atom(Value::number(42.0)), // Parameter must be symbol
        ]);
        let args = vec![function_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("function parameters must be symbols")
        );

        // Test function definition with empty parameter list
        let function_def = Expression::List(vec![]);
        let args = vec![function_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires non-empty parameter list")
        );

        // Test function definition without body
        let function_def = Expression::List(vec![Expression::atom(Value::symbol("fn"))]);
        let args = vec![function_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("function definition requires at least one body expression")
        );

        // Test invalid first argument
        let args = vec![Expression::atom(Value::number(42.0))];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("first argument must be a symbol or parameter list")
        );
    }

    #[test]
    fn test_eval_define_variable_shadowing() {
        let mut env = Environment::new();

        // Define a variable
        let args1 = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ];
        eval_define(&args1, &mut env).unwrap();
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));

        // Redefine the same variable (should shadow)
        let args2 = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::string("hello")),
        ];
        eval_define(&args2, &mut env).unwrap();
        assert_eq!(
            env.lookup(&Symbol::new("x")).unwrap(),
            Value::string("hello")
        );
    }
}
