//! Lambda special form implementation
//!
//! This module implements the `lambda` special form for creating user-defined
//! procedures with lexical closure support. Lambda expressions create procedures
//! that capture their defining environment and can be called with arguments.

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::runtime::Environment;
use crate::types::{Procedure, Symbol, Value};
use std::sync::Arc;

/// Evaluate a lambda expression
///
/// Lambda syntax: `(lambda (param1 param2 ...) body1 body2 ... bodyn)`
///
/// Creates a new procedure with the specified parameters and body expressions.
/// The procedure captures the current environment as a closure, implementing
/// lexical scoping as required by FR-13. Multiple body expressions are evaluated
/// in sequence, with only the last expression in tail position.
///
/// # Arguments
/// * `args` - The lambda arguments: parameter list followed by one or more body expressions
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
/// // (lambda (x) (display x) (+ x 1))  ; Multi-expression body
/// ```
pub fn eval_lambda(args: &[Arc<Expression>], env: &Environment) -> Result<Value> {
    // Lambda requires at least 2 arguments: parameter list and one or more body expressions
    if args.len() < 2 {
        return Err(Error::arity_error("lambda", 2, args.len()));
    }

    // Extract and validate parameters
    let params_expr = Arc::clone(&args[0]);
    let param_elements = match params_expr.as_ref() {
        Expression::List(elements) => elements,
        Expression::Atom(Value::Symbol(_)) => {
            return Err(Error::parse_error(
                "lambda: parameters must be enclosed in parentheses",
            ));
        }
        Expression::Atom(other) => {
            return Err(Error::parse_error(&format!(
                "lambda: parameter list must be a list, got {}",
                other.type_name()
            )));
        }
        Expression::Quote(_) => {
            return Err(Error::parse_error(
                "lambda: parameter list must be a list, got quote",
            ));
        }
    };
    let params = parse_parameters(param_elements)?;
    validate_parameters(&params)?;

    // Collect all body expressions (everything after the parameter list)
    let body_exprs = args[1..].iter().map(Arc::clone).collect();

    // Create lambda procedure using shared logic
    Ok(create_lambda_procedure(params, body_exprs, env))
}

/// Create a lambda procedure from validated parameters and body expressions
///
/// This shared function handles the common logic of creating lambda procedures
/// used by both `eval_lambda` and procedure definitions in `define`.
///
/// # Arguments
/// * `params` - Already validated parameter symbols
/// * `body_exprs` - The body expressions for the lambda (one or more)
/// * `env` - Environment to capture for closure
///
/// # Returns
/// A new lambda procedure value
pub fn create_lambda_procedure(
    params: Vec<Symbol>,
    body_exprs: Vec<Arc<Expression>>,
    env: &Environment,
) -> Value {
    let lambda_proc = Procedure::lambda(params, body_exprs, env.flatten());
    Value::Procedure(lambda_proc)
}

/// Parse parameters from a slice of expressions
///
/// Takes a slice of parameter expressions and extracts individual parameter symbols.
/// Validates that all parameters are symbols and returns them as a vector.
///
/// # Arguments
/// * `param_elements` - Slice of expressions representing the parameters
///
/// # Returns
/// Vector of parameter symbols if valid, Error if malformed
///
/// # Errors
/// Returns error if any parameter is not a symbol
pub fn parse_parameters(param_elements: &[Arc<Expression>]) -> Result<Vec<Symbol>> {
    let mut params = Vec::with_capacity(param_elements.len());

    for element in param_elements {
        match element.as_ref() {
            Expression::Atom(Value::Symbol(symbol)) => {
                params.push(symbol.clone());
            }
            Expression::Atom(other) => {
                return Err(Error::parse_error(&format!(
                    "lambda: parameter must be a symbol, got {}",
                    other.type_name()
                )));
            }
            Expression::List(_) => {
                return Err(Error::parse_error(
                    "lambda: parameter must be a symbol, not list",
                ));
            }
            Expression::Quote(_) => {
                return Err(Error::parse_error(
                    "lambda: parameter must be a symbol, not quoted expression",
                ));
            }
        }
    }
    Ok(params)
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
pub fn validate_parameters(params: &[Symbol]) -> Result<()> {
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
        let params = Expression::arc_list(vec![]);
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, Arc::clone(&body)];

        let result = eval_lambda(&args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(0));
            assert_eq!(proc.params().unwrap().len(), 0);
            assert_eq!(proc.body().unwrap(), &[body]);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_single_parameter() {
        let env = Environment::new();

        // (lambda (x) x)
        let params = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]);
        let body = Expression::arc_atom(Value::symbol("x"));
        let args = vec![params, Arc::clone(&body)];

        let result = eval_lambda(&args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(1));
            let param_list = proc.params().unwrap();
            assert_eq!(param_list.len(), 1);
            assert_eq!(param_list[0], Symbol::new("x"));
            assert_eq!(proc.body().unwrap(), &[body]);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_multiple_parameters() {
        let env = Environment::new();

        // (lambda (x y z) (+ x y z))
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
            Expression::arc_atom(Value::symbol("z")),
        ]);
        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
            Expression::arc_atom(Value::symbol("z")),
        ]);
        let args = vec![params, Arc::clone(&body)];

        let result = eval_lambda(&args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(3));
            let param_list = proc.params().unwrap();
            assert_eq!(param_list.len(), 3);
            assert_eq!(param_list[0], Symbol::new("x"));
            assert_eq!(param_list[1], Symbol::new("y"));
            assert_eq!(param_list[2], Symbol::new("z"));
            assert_eq!(proc.body().unwrap(), &[body]);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_environment_capture() {
        let mut env = Environment::new();
        env.define(Symbol::new("outer"), Value::number(100.0));

        // (lambda (x) (+ x outer))
        let params = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]);
        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("outer")),
        ]);
        let args = vec![params, body];

        let result = eval_lambda(&args, &env).unwrap();

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
    fn test_lambda_arity_error() {
        let env = Environment::new();

        // Test too few arguments
        let args = vec![Expression::arc_atom(Value::symbol("x"))];
        let result = eval_lambda(&args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected 2 arguments")
        );

        // Test that multiple body expressions are now valid (no longer an error)
        let args = vec![
            Expression::arc_list(vec![]),
            Expression::arc_atom(Value::number(42.0)),
            Expression::arc_atom(Value::number(43.0)),
        ];
        let result = eval_lambda(&args, &env);
        assert!(result.is_ok()); // This should now succeed with multiple body expressions
    }

    #[test]
    fn test_lambda_parameter_validation_errors() {
        let env = Environment::new();

        // Parameter list is not a list
        let params = Expression::arc_atom(Value::symbol("x"));
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(&args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameters must be enclosed in parentheses")
        );

        // Parameter list contains non-symbol
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)), // Not a symbol
        ]);
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(&args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameter must be a symbol")
        );

        // Parameter list contains list
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_list(vec![]), // Invalid parameter
        ]);
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(&args, &env);
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
        // Duplicate parameters
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("x")), // Duplicate
        ]);
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(&args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate parameter 'x'")
        );

        // (lambda (a b a) 42) - duplicate parameter
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("a")),
            Expression::arc_atom(Value::symbol("b")),
            Expression::arc_atom(Value::symbol("a")), // Duplicate
        ]);
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(&args, &env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate parameter 'a'")
        );
    }

    #[test]
    fn test_lambda_parameter_parsing() {
        // Test parse_parameters function directly

        // Empty list
        let params = vec![];
        let result = parse_parameters(&params).unwrap();
        assert_eq!(result.len(), 0);

        // Single parameter
        let params = vec![Expression::arc_atom(Value::symbol("x"))];
        let result = parse_parameters(&params).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Symbol::new("x"));

        // Multiple parameters
        let params = vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ];
        let result = parse_parameters(&params).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Symbol::new("x"));
        assert_eq!(result[1], Symbol::new("y"));

        // Invalid parameter (number instead of symbol)
        let params = vec![Expression::arc_atom(Value::number(42.0))];
        let result = parse_parameters(&params);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameter must be a symbol")
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
        let params = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]);
        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol(">")),
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(0.0)),
            ]),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(0.0)),
        ]);
        let args = vec![params, Arc::clone(&body)];

        let result = eval_lambda(&args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.body().unwrap(), &[body]);
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_procedure_display() {
        let env = Environment::new();

        // (lambda (x y) body)
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ]);
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(&args, &env).unwrap();

        // Test that the procedure displays correctly
        assert_eq!(format!("{result}"), "#<lambda:x y>");
    }

    #[test]
    fn test_lambda_with_keyword_parameter_names() {
        let env = Environment::new();

        // Lambda parameters can use any valid symbol names, including those
        // that might look like keywords but are just symbols
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("if")),
            Expression::arc_atom(Value::symbol("define")),
            Expression::arc_atom(Value::symbol("lambda")),
        ]);
        let body = Expression::arc_atom(Value::number(42.0));
        let args = vec![params, body];

        let result = eval_lambda(&args, &env).unwrap();

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

    #[test]
    fn test_lambda_multiple_body_expressions() {
        let env = Environment::new();

        // Test lambda with two body expressions
        let params = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]);
        let body1 = Expression::arc_atom(Value::number(1.0));
        let body2 = Expression::arc_atom(Value::symbol("x"));
        let args = vec![params, body1, body2];

        let result = eval_lambda(&args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(1));
            let body_exprs = proc.body().unwrap();
            assert_eq!(body_exprs.len(), 2);
            assert_eq!(body_exprs[0], Expression::arc_atom(Value::number(1.0)));
            assert_eq!(body_exprs[1], Expression::arc_atom(Value::symbol("x")));
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_three_body_expressions() {
        let env = Environment::new();

        // Test lambda with three body expressions
        let params = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ]);
        let body1 = Expression::arc_atom(Value::number(1.0));
        let body2 = Expression::arc_atom(Value::number(2.0));
        let body3 = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ]);
        let args = vec![params, body1, body2, body3];

        let result = eval_lambda(&args, &env).unwrap();

        if let Value::Procedure(proc) = result {
            assert!(proc.is_lambda());
            assert_eq!(proc.arity(), Some(2));
            let body_exprs = proc.body().unwrap();
            assert_eq!(body_exprs.len(), 3);
            assert_eq!(body_exprs[0], Expression::arc_atom(Value::number(1.0)));
            assert_eq!(body_exprs[1], Expression::arc_atom(Value::number(2.0)));
            // The third expression should be the addition
            if let Expression::List(elements) = body_exprs[2].as_ref() {
                assert_eq!(elements.len(), 3);
            } else {
                panic!("Expected list expression");
            }
        } else {
            panic!("Expected lambda procedure");
        }
    }

    #[test]
    fn test_lambda_minimum_body_requirement() {
        let env = Environment::new();

        // Test that lambda still requires at least one body expression
        let params = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]);
        let args = vec![params]; // Only parameters, no body

        let result = eval_lambda(&args, &env);
        assert!(result.is_err());
        // Check that the error is about requiring at least 2 arguments (params + body)
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("arity") || error_msg.contains("2"));
    }
}
