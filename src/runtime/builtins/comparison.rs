//! Comparison builtin procedures for the Twine Scheme runtime
//!
//! This module implements the core comparison operations as builtin procedures:
//! =, <, >, <=, >=
//!
//! All operations include proper arity checking and type validation.

use crate::error::{Error, Result};
use crate::types::Value;

/// Numeric equality
///
/// Scheme: (= number1 number2 ...)
/// Returns #t if all arguments are numerically equal, #f otherwise.
/// Requires at least two arguments.
pub fn equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Error::arity_error("=", 2, args.len()));
    }

    // Check all arguments are numbers first
    for (i, arg) in args.iter().enumerate() {
        if !arg.is_number() {
            return Err(Error::type_error(
                "=",
                "number",
                arg.type_name(),
                Some(i + 1),
            ));
        }
    }

    let first = args[0].as_number().unwrap();
    for arg in &args[1..] {
        if arg.as_number().unwrap() != first {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Numeric less-than
///
/// Scheme: (< number1 number2 ...)
/// Returns #t if arguments are in strictly increasing order, #f otherwise.
/// Requires at least two arguments.
pub fn less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Error::runtime_error("'<' requires at least two arguments"));
    }

    // Check all arguments are numbers first
    for arg in args {
        if !arg.is_number() {
            return Err(Error::runtime_error(&format!(
                "'<' requires numeric arguments, got {}",
                arg.type_name()
            )));
        }
    }

    for i in 1..args.len() {
        let prev = args[i - 1].as_number().unwrap();
        let curr = args[i].as_number().unwrap();
        if prev >= curr {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Numeric greater-than
///
/// Scheme: (> number1 number2 ...)
/// Returns #t if arguments are in strictly decreasing order, #f otherwise.
/// Requires at least two arguments.
pub fn greater_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Error::runtime_error("'>' requires at least two arguments"));
    }

    // Check all arguments are numbers first
    for arg in args {
        if !arg.is_number() {
            return Err(Error::runtime_error(&format!(
                "'>' requires numeric arguments, got {}",
                arg.type_name()
            )));
        }
    }

    for i in 1..args.len() {
        let prev = args[i - 1].as_number().unwrap();
        let curr = args[i].as_number().unwrap();
        if prev <= curr {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Numeric less-than-or-equal
///
/// Scheme: (<= number1 number2 ...)
/// Returns #t if arguments are in non-decreasing order, #f otherwise.
/// Requires at least two arguments.
pub fn less_than_or_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Error::runtime_error("'<=' requires at least two arguments"));
    }

    // Check all arguments are numbers first
    for arg in args {
        if !arg.is_number() {
            return Err(Error::runtime_error(&format!(
                "'<=' requires numeric arguments, got {}",
                arg.type_name()
            )));
        }
    }

    for i in 1..args.len() {
        let prev = args[i - 1].as_number().unwrap();
        let curr = args[i].as_number().unwrap();
        if prev > curr {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// Numeric greater-than-or-equal
///
/// Scheme: (>= number1 number2 ...)
/// Returns #t if arguments are in non-increasing order, #f otherwise.
/// Requires at least two arguments.
pub fn greater_than_or_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Error::runtime_error("'>=' requires at least two arguments"));
    }

    // Check all arguments are numbers first
    for arg in args {
        if !arg.is_number() {
            return Err(Error::runtime_error(&format!(
                "'>=' requires numeric arguments, got {}",
                arg.type_name()
            )));
        }
    }

    for i in 1..args.len() {
        let prev = args[i - 1].as_number().unwrap();
        let curr = args[i].as_number().unwrap();
        if prev < curr {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        // Less than two arguments should error
        let result = equal(&[Value::number(1.0)]);
        assert!(result.is_err());

        // Equal numbers
        let result = equal(&[Value::number(5.0), Value::number(5.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        // Unequal numbers
        let result = equal(&[Value::number(5.0), Value::number(6.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Multiple equal numbers
        let result = equal(&[Value::number(3.0), Value::number(3.0), Value::number(3.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        // Multiple numbers with one different
        let result = equal(&[Value::number(3.0), Value::number(3.0), Value::number(4.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Non-numeric argument should error
        let result = equal(&[Value::number(1.0), Value::string("1")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_less_than() {
        // Less than two arguments should error
        let result = less_than(&[Value::number(1.0)]);
        assert!(result.is_err());

        // Strictly increasing
        let result =
            less_than(&[Value::number(1.0), Value::number(2.0), Value::number(3.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        // Not strictly increasing
        let result =
            less_than(&[Value::number(1.0), Value::number(2.0), Value::number(2.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Decreasing
        let result = less_than(&[Value::number(3.0), Value::number(2.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_greater_than() {
        // Less than two arguments should error
        let result = greater_than(&[Value::number(1.0)]);
        assert!(result.is_err());

        // Strictly decreasing
        let result =
            greater_than(&[Value::number(3.0), Value::number(2.0), Value::number(1.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        // Not strictly decreasing
        let result =
            greater_than(&[Value::number(3.0), Value::number(2.0), Value::number(2.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Increasing
        let result = greater_than(&[Value::number(1.0), Value::number(2.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_less_than_or_equal() {
        // Non-decreasing (allows equal)
        let result = less_than_or_equal(&[
            Value::number(1.0),
            Value::number(2.0),
            Value::number(2.0),
            Value::number(3.0),
        ])
        .unwrap();
        assert!(result.as_boolean().unwrap());

        // Decreasing should fail
        let result = less_than_or_equal(&[Value::number(3.0), Value::number(2.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_greater_than_or_equal() {
        // Non-increasing (allows equal)
        let result = greater_than_or_equal(&[
            Value::number(3.0),
            Value::number(2.0),
            Value::number(2.0),
            Value::number(1.0),
        ])
        .unwrap();
        assert!(result.as_boolean().unwrap());

        // Increasing should fail
        let result = greater_than_or_equal(&[Value::number(1.0), Value::number(2.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());
    }

    #[test]
    fn test_type_checking() {
        // All operations should reject non-numeric types
        let string_arg = Value::string("not a number");
        let bool_arg = Value::boolean(true);
        let symbol_arg = Value::symbol("x");

        assert!(equal(&[Value::number(1.0), string_arg.clone()]).is_err());
        assert!(less_than(&[Value::number(1.0), symbol_arg.clone()]).is_err());
        assert!(greater_than(&[Value::number(1.0), string_arg.clone()]).is_err());
        assert!(less_than_or_equal(&[bool_arg.clone(), Value::number(1.0)]).is_err());
        assert!(greater_than_or_equal(&[symbol_arg, Value::number(1.0)]).is_err());
    }

    #[test]
    fn test_edge_cases() {
        // Test with zero
        let result = equal(&[Value::number(0.0), Value::number(0.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        // Test with negative numbers
        let result = less_than(&[Value::number(-2.0), Value::number(-1.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        // Test floating point precision
        let result = equal(&[Value::number(1.0 / 3.0), Value::number(1.0 / 3.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        // Test equal values with <=, >=
        let result = less_than_or_equal(&[Value::number(5.0), Value::number(5.0)]).unwrap();
        assert!(result.as_boolean().unwrap());

        let result = greater_than_or_equal(&[Value::number(5.0), Value::number(5.0)]).unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_arity_errors() {
        // All comparison operations require at least 2 arguments
        assert!(equal(&[]).is_err());
        assert!(equal(&[Value::number(1.0)]).is_err());

        assert!(less_than(&[]).is_err());
        assert!(less_than(&[Value::number(1.0)]).is_err());

        assert!(greater_than(&[]).is_err());
        assert!(greater_than(&[Value::number(1.0)]).is_err());

        assert!(less_than_or_equal(&[]).is_err());
        assert!(less_than_or_equal(&[Value::number(1.0)]).is_err());

        assert!(greater_than_or_equal(&[]).is_err());
        assert!(greater_than_or_equal(&[Value::number(1.0)]).is_err());
    }

    #[test]
    fn test_multi_argument_comparisons() {
        // Test multiple arguments for all comparison operations

        // Equal with multiple arguments
        let result = equal(&[
            Value::number(2.0),
            Value::number(2.0),
            Value::number(2.0),
            Value::number(2.0),
        ])
        .unwrap();
        assert!(result.as_boolean().unwrap());

        let result = equal(&[Value::number(2.0), Value::number(2.0), Value::number(3.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Less than with multiple arguments (strictly increasing)
        let result = less_than(&[
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
            Value::number(4.0),
        ])
        .unwrap();
        assert!(result.as_boolean().unwrap());

        let result =
            less_than(&[Value::number(1.0), Value::number(3.0), Value::number(2.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Greater than with multiple arguments (strictly decreasing)
        let result = greater_than(&[
            Value::number(4.0),
            Value::number(3.0),
            Value::number(2.0),
            Value::number(1.0),
        ])
        .unwrap();
        assert!(result.as_boolean().unwrap());

        let result =
            greater_than(&[Value::number(4.0), Value::number(2.0), Value::number(3.0)]).unwrap();
        assert!(!result.as_boolean().unwrap());

        // Less than or equal with multiple arguments (non-decreasing)
        let result = less_than_or_equal(&[
            Value::number(1.0),
            Value::number(2.0),
            Value::number(2.0),
            Value::number(3.0),
        ])
        .unwrap();
        assert!(result.as_boolean().unwrap());

        let result =
            less_than_or_equal(&[Value::number(1.0), Value::number(3.0), Value::number(2.0)])
                .unwrap();
        assert!(!result.as_boolean().unwrap());

        // Greater than or equal with multiple arguments (non-increasing)
        let result = greater_than_or_equal(&[
            Value::number(3.0),
            Value::number(2.0),
            Value::number(2.0),
            Value::number(1.0),
        ])
        .unwrap();
        assert!(result.as_boolean().unwrap());

        let result =
            greater_than_or_equal(&[Value::number(3.0), Value::number(1.0), Value::number(2.0)])
                .unwrap();
        assert!(!result.as_boolean().unwrap());
    }
}
