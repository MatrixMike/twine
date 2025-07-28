//! Lambda special form implementation
//!
//! This module implements the `lambda` special form for creating user-defined
//! procedures with lexical closure support. Lambda expressions create procedures
//! that capture their defining environment and can be called with arguments.

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::runtime::Environment;
use crate::types::{Procedure, Symbol, Value};

/// Evaluate a lambda expression
///
/// Lambda syntax: `(lambda (param1 param2 ...) body)`
///
/// Creates a new procedure with the specified parameters and body expression.
/// The procedure captures the current environment as a closure, implementing
/// lexical scoping as required by FR-13.
///
/// # Arguments
/// * `args` - The lambda arguments: parameter list and body expression
/// * `env` - Current environment for closure capture
///
/// # Returns
/// A new `Value::Procedure` containing the lambda with captured environment
///
/// # Examples
/// ```
/// // (lambda (x) (* x x))
/// // (lambda (x y) (+ x y))
/// // (lambda () 42)
/// ```
pub fn eval_lambda(mut args: Vec<Expression>, env: &Environment) -> Result<Value> {
    // Lambda requires exactly 2 arguments: parameter list and body
    if args.len() != 2 {
        return Err(Error::arity_error("lambda", 2, args.len()));
    }

    let body_expr = args.pop().unwrap();
    let params_expr = args.pop().unwrap();

    // Parse parameter list - must be a list of symbols
    let params = parse_parameter_list(params_expr)?;

    // Validate that all parameters are unique identifiers
    validate_parameters(&params)?;

    // Create lambda procedure with captured environment (closure)
    // The environment is captured at lambda creation time (lexical scoping)
    // Flatten the environment to remove lifetime constraints
    //
    // TODO: Remove `env.flatten()` and instead create a minimal ebv with
    // only the bindings that are captured inside the lambda.
    let lambda_proc = Procedure::lambda(params, body_expr, env.flatten());

    Ok(Value::Procedure(lambda_proc))
}

/// Parse the parameter list from a lambda expression
///
/// Parameter list can be:
/// - Empty list: `()`
/// - List of symbols: `(x y z)`
///
/// # Arguments
/// * `params_expr` - Expression representing the parameter list
///
/// # Returns
/// Vector of parameter symbols
///
/// # Errors
/// Returns error if parameter list is malformed or contains non-symbols
fn parse_parameter_list(params_expr: Expression) -> Result<Vec<Symbol>> {
    match params_expr {
        Expression::List(elements) => {
            let mut params = Vec::with_capacity(elements.len());

            for element in elements {
                match element {
                    Expression::Atom(Value::Symbol(symbol)) => {
                        params.push(symbol);
                    }
                    Expression::Atom(other) => {
                        return Err(Error::parse_error(&format!(
                            "lambda: parameter must be a symbol, got {}",
                            other.type_name()
                        )));
                    }
                    Expression::List(_) => {
                        return Err(Error::parse_error(
                            "lambda: parameter must be a symbol, got list",
                        ));
                    }
                    Expression::Quote(_) => {
                        return Err(Error::parse_error(
                            "lambda: parameter must be a symbol, got quote",
                        ));
                    }
                }
            }

            Ok(params)
        }
        Expression::Atom(Value::Symbol(_)) => {
            // Single parameter (not in a list) - this is not standard Scheme
            Err(Error::parse_error(
                "lambda: parameters must be enclosed in parentheses",
            ))
        }
        Expression::Atom(other) => Err(Error::parse_error(&format!(
            "lambda: parameter list must be a list, got {}",
            other.type_name()
        ))),
        Expression::Quote(_) => Err(Error::parse_error(
            "lambda: parameter list must be a list, got quote",
        )),
    }
}

/// Validate that all parameters are unique identifiers
///
/// Checks for duplicate parameter names, which would create ambiguous bindings.
/// This follows Scheme conventions where parameter names must be unique.
///
/// # Arguments
/// * `params` - Vector of parameter symbols to validate
///
/// # Returns
/// Ok(()) if all parameters are unique, Error if duplicates found
fn validate_parameters(params: &[Symbol]) -> Result<()> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();

    for param in params {
        if !seen.insert(param) {
            return Err(Error::parse_error(&format!(
                "lambda: duplicate parameter '{param}'"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_lambda_no_parameters() {
        let env = Environment::new();

        // (lambda () 42)
        let params = Expression::List(vec![]);
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body.clone()];

        let result = eval_lambda(args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(0));
            assert_eq!(proc.params().unwrap().len(), 0);
            assert_eq!(proc.body().unwrap(), &body);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_single_parameter() {
        let env = Environment::new();

        // (lambda (x) x)
        let params = Expression::List(vec![Expression::atom(Value::symbol("x"))]);
        let body = Expression::atom(Value::symbol("x"));
        let args = vec![params, body.clone()];

        let result = eval_lambda(args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(1));
            let param_list = proc.params().unwrap();
            assert_eq!(param_list.len(), 1);
            assert_eq!(param_list[0], Symbol::new("x"));
            assert_eq!(proc.body().unwrap(), &body);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_multiple_parameters() {
        let env = Environment::new();

        // (lambda (x y z) (+ x y z))
        let params = Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
            Expression::atom(Value::symbol("z")),
        ]);
        let body = Expression::List(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
            Expression::atom(Value::symbol("z")),
        ]);
        let args = vec![params, body.clone()];

        let result = eval_lambda(args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(3));
            let param_list = proc.params().unwrap();
            assert_eq!(param_list.len(), 3);
            assert_eq!(param_list[0], Symbol::new("x"));
            assert_eq!(param_list[1], Symbol::new("y"));
            assert_eq!(param_list[2], Symbol::new("z"));
            assert_eq!(proc.body().unwrap(), &body);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_environment_capture() {
        let mut env = Environment::new();
        env.define(Symbol::new("outer"), Value::number(100.0));

        // (lambda (x) (+ x outer))
        let params = Expression::List(vec![Expression::atom(Value::symbol("x"))]);
        let body = Expression::List(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("outer")),
        ]);
        let args = vec![params, body.clone()];

        let result = eval_lambda(args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            let captured_env = proc.env().unwrap();

            // Verify environment was captured
            assert_eq!(
                captured_env.lookup(&Symbol::new("outer")).unwrap(),
                Value::number(100.0)
            );
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_arity_errors() {
        let env = Environment::new();

        // Too few arguments
        let args = vec![Expression::List(vec![])];
        let result = eval_lambda(args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected 2 arguments")
        );

        // Too many arguments
        let args = vec![
            Expression::List(vec![]),
            Expression::atom(Value::number(42.0)),
            Expression::atom(Value::number(43.0)),
        ];
        let result = eval_lambda(args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected 2 arguments")
        );
    }

    #[test]
    fn test_lambda_parameter_validation_errors() {
        let env = Environment::new();

        // Parameter list is not a list
        let params = Expression::atom(Value::symbol("x"));
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameters must be enclosed in parentheses")
        );

        // Parameter list contains non-symbol
        let params = Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)), // Invalid parameter
        ]);
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameter must be a symbol")
        );

        // Parameter list contains list
        let params = Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::List(vec![]), // Invalid parameter
        ]);
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameter must be a symbol")
        );
    }

    #[test]
    fn test_lambda_duplicate_parameters() {
        let env = Environment::new();

        // (lambda (x x) 42) - duplicate parameter
        let params = Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("x")), // Duplicate
        ]);
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate parameter 'x'")
        );

        // (lambda (a b a) 42) - duplicate parameter
        let params = Expression::List(vec![
            Expression::atom(Value::symbol("a")),
            Expression::atom(Value::symbol("b")),
            Expression::atom(Value::symbol("a")), // Duplicate
        ]);
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate parameter 'a'")
        );
    }

    #[test]
    fn test_lambda_parameter_list_parsing() {
        // Test parse_parameter_list function directly

        // Empty list
        let params_expr = Expression::List(vec![]);
        let result = parse_parameter_list(params_expr).unwrap();
        assert_eq!(result.len(), 0);

        // Single parameter
        let params_expr = Expression::List(vec![Expression::atom(Value::symbol("x"))]);
        let result = parse_parameter_list(params_expr).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Symbol::new("x"));

        // Multiple parameters
        let params_expr = Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
        ]);
        let result = parse_parameter_list(params_expr).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Symbol::new("x"));
        assert_eq!(result[1], Symbol::new("y"));

        // Non-list parameter expression
        let params_expr = Expression::atom(Value::symbol("x"));
        let result = parse_parameter_list(params_expr);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameters must be enclosed in parentheses")
        );
    }

    #[test]
    fn test_validate_parameters() {
        // Valid parameters (unique)
        let params = vec![Symbol::new("x"), Symbol::new("y"), Symbol::new("z")];
        assert!(validate_parameters(&params).is_ok());

        // Empty parameters (valid)
        let params = vec![];
        assert!(validate_parameters(&params).is_ok());

        // Single parameter (valid)
        let params = vec![Symbol::new("x")];
        assert!(validate_parameters(&params).is_ok());

        // Duplicate parameters (invalid)
        let params = vec![Symbol::new("x"), Symbol::new("y"), Symbol::new("x")];
        let result = validate_parameters(&params);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate parameter 'x'")
        );
    }

    #[test]
    fn test_lambda_edge_cases() {
        let env = Environment::new();

        // Lambda with complex body expression
        let params = Expression::List(vec![Expression::atom(Value::symbol("x"))]);
        let body = Expression::List(vec![
            Expression::atom(Value::symbol("if")),
            Expression::List(vec![
                Expression::atom(Value::symbol(">")),
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::number(0.0)),
            ]),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(0.0)),
        ]);
        let args = vec![params, body.clone()];

        let result = eval_lambda(args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.body().unwrap(), &body);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_procedure_display() {
        let env = Environment::new();

        // (lambda (x y) body)
        let params = Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
        ]);
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(args, &env).unwrap();

        // Test that the procedure displays correctly
        assert_eq!(format!("{result}"), "#<lambda:x y>");
    }

    #[test]
    fn test_lambda_with_keyword_parameter_names() {
        let env = Environment::new();

        // Lambda parameters can use any valid symbol names, including those
        // that might look like keywords but are just symbols
        let params = Expression::List(vec![
            Expression::atom(Value::symbol("if")),
            Expression::atom(Value::symbol("define")),
            Expression::atom(Value::symbol("lambda")),
        ]);
        let body = Expression::atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            let param_list = proc.params().unwrap();
            assert_eq!(param_list.len(), 3);
            assert_eq!(param_list[0], Symbol::new("if"));
            assert_eq!(param_list[1], Symbol::new("define"));
            assert_eq!(param_list[2], Symbol::new("lambda"));
        } else {
            panic!("Expected lambda procedure");
        }
    }
}
