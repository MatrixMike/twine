//! Runtime utilities for shared evaluation patterns
//!
//! This module provides common utilities used across multiple runtime components
//! to reduce code duplication and maintain consistency.

use crate::error::{Error, Result};
use crate::parser::Expression;
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::{Symbol, Value};
use std::collections::HashSet;
use std::sync::Arc;

/// Evaluate a sequence of expressions and return the value of the last one
///
/// This is the common pattern used by `begin`, `let`, `letrec`, and other forms
/// that need to evaluate multiple expressions sequentially and return the final result.
///
/// # Arguments
/// * `exprs` - Slice of expressions to evaluate sequentially
/// * `env` - Environment to evaluate expressions in
///
/// # Returns
/// The value of the last expression, or `Value::Nil` if no expressions provided
///
/// # Errors
/// Returns the first error encountered during evaluation
pub fn eval_sequence(exprs: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if exprs.is_empty() {
        return Ok(Value::Nil);
    }

    let mut result = Value::Nil;
    for expr in exprs {
        result = eval(Arc::clone(expr), env)?;
    }
    Ok(result)
}

/// Check for duplicate identifiers in a collection of symbols
///
/// Uses efficient HashSet-based checking to detect duplicates in O(n) time.
/// This is used for parameter validation, binding validation, and other contexts
/// where unique identifiers are required.
///
/// # Arguments
/// * `identifiers` - Slice of symbols to check for duplicates
/// * `form_name` - Name of the form (for error messages)
/// * `error_fn` - Function to create the appropriate error for duplicates
///
/// # Returns
/// `Ok(())` if all identifiers are unique, error if duplicates found
///
/// # Type Parameters
/// * `F` - Error creation function type
/// * `E` - Error type returned by the function
pub fn validate_unique_identifiers<F, E>(
    identifiers: &[Symbol],
    form_name: &str,
    error_fn: F,
) -> Result<()>
where
    F: Fn(&str, &str) -> E,
    E: Into<crate::Error>,
{
    let mut seen = HashSet::new();

    for identifier in identifiers {
        if !seen.insert(identifier) {
            return Err(error_fn(form_name, identifier.as_str()).into());
        }
    }

    Ok(())
}

/// Specialized version for parameter validation
///
/// Provides a convenient interface for the common case of validating parameters
/// with the standard duplicate parameter error.
///
/// # Arguments
/// * `params` - Parameter symbols to validate
/// * `form_name` - Name of the form (for error messages)
///
/// # Returns
/// `Ok(())` if all parameters are unique, duplicate parameter error otherwise
pub fn validate_unique_parameters(params: &[Symbol], form_name: &str) -> Result<()> {
    validate_unique_identifiers(params, form_name, crate::Error::duplicate_parameter_error)
}

/// Specialized version for identifier validation in bindings
///
/// Provides a convenient interface for validating identifiers in binding forms
/// with the standard duplicate identifier error.
///
/// # Arguments
/// * `identifiers` - Identifier symbols to validate
/// * `form_name` - Name of the form (for error messages)
///
/// # Returns
/// `Ok(())` if all identifiers are unique, duplicate identifier error otherwise
pub fn validate_unique_binding_identifiers(identifiers: &[Symbol], form_name: &str) -> Result<()> {
    validate_unique_identifiers(
        identifiers,
        form_name,
        crate::Error::duplicate_identifier_error,
    )
}

/// Parse parameter list from expressions
///
/// Takes a slice of parameter expressions and extracts individual parameter symbols.
/// Validates that all parameters are symbols and returns them as a vector.
///
/// # Arguments
/// * `param_elements` - Slice of expressions representing the parameters
/// * `form_name` - Name of the special form (for error messages)
///
/// # Returns
/// Vector of parameter symbols if valid, Error if malformed
///
/// # Errors
/// Returns error if any parameter is not a symbol, with context-specific error messages
pub fn parse_parameters(
    param_elements: &[Arc<Expression>],
    form_name: &str,
) -> Result<Vec<Symbol>> {
    let mut params = Vec::with_capacity(param_elements.len());

    for element in param_elements {
        match element.as_ref() {
            Expression::Atom(Value::Symbol(symbol)) => {
                params.push(symbol.clone());
            }
            other => {
                return Err(Error::parameter_must_be_symbol_error(
                    form_name,
                    other.type_name(),
                ));
            }
        }
    }
    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Symbol;

    #[test]
    fn test_eval_sequence_empty() {
        let mut env = Environment::new();
        let result = eval_sequence(&[], &mut env).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_sequence_single() {
        let mut env = Environment::new();
        let exprs = vec![Expression::arc_atom(Value::number(42.0))];
        let result = eval_sequence(&exprs, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_sequence_multiple() {
        let mut env = Environment::new();
        let exprs = vec![
            Expression::arc_atom(Value::number(1.0)),
            Expression::arc_atom(Value::number(2.0)),
            Expression::arc_atom(Value::number(3.0)),
        ];
        let result = eval_sequence(&exprs, &mut env).unwrap();
        assert_eq!(result, Value::number(3.0)); // Should return last value
    }

    #[test]
    fn test_validate_unique_parameters_success() {
        let params = vec![Symbol::new("x"), Symbol::new("y"), Symbol::new("z")];
        assert!(validate_unique_parameters(&params, "lambda").is_ok());
    }

    #[test]
    fn test_validate_unique_parameters_duplicate() {
        let params = vec![Symbol::new("x"), Symbol::new("y"), Symbol::new("x")];
        let result = validate_unique_parameters(&params, "lambda");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate parameter 'x'")
        );
    }

    #[test]
    fn test_validate_unique_binding_identifiers_success() {
        let identifiers = vec![Symbol::new("a"), Symbol::new("b"), Symbol::new("c")];
        assert!(validate_unique_binding_identifiers(&identifiers, "letrec").is_ok());
    }

    #[test]
    fn test_validate_unique_binding_identifiers_duplicate() {
        let identifiers = vec![Symbol::new("a"), Symbol::new("b"), Symbol::new("a")];
        let result = validate_unique_binding_identifiers(&identifiers, "letrec");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("duplicate identifier 'a'")
        );
    }

    #[test]
    fn test_parse_parameters_empty() {
        let params = vec![];
        let result = parse_parameters(&params, "lambda").unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_parameters_single() {
        let params = vec![Expression::arc_atom(Value::symbol("x"))];
        let result = parse_parameters(&params, "lambda").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Symbol::new("x"));
    }

    #[test]
    fn test_parse_parameters_multiple() {
        let params = vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ];
        let result = parse_parameters(&params, "lambda").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Symbol::new("x"));
        assert_eq!(result[1], Symbol::new("y"));
    }

    #[test]
    fn test_parse_parameters_invalid_parameter() {
        let params = vec![Expression::arc_atom(Value::number(42.0))];
        let result = parse_parameters(&params, "lambda");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameter must be a symbol")
        );
    }
}
