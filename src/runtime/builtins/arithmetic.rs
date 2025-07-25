//! Arithmetic builtin procedures for the Twine Scheme runtime
//!
//! This module implements the core arithmetic operations as builtin procedures:
//! +, -, *, /
//!
//! All operations include proper arity checking and type validation.

use crate::error::{Error, Result};
use crate::types::Value;

/// Add two or more numbers
///
/// Scheme: (+ number1 number2 ...)
/// Returns the sum of all arguments. With no arguments, returns 0.
pub fn add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::number(0.0));
    }

    let mut sum = 0.0;
    for arg in args {
        match arg {
            Value::Number(n) => sum += n.value(),
            _ => {
                return Err(Error::runtime_error(&format!(
                    "'+' requires numeric arguments, got {}",
                    arg.type_name()
                )));
            }
        }
    }
    Ok(Value::number(sum))
}

/// Subtract numbers
///
/// Scheme: (- number1 number2 ...)
/// With one argument, returns its negation.
/// With multiple arguments, subtracts all subsequent arguments from the first.
pub fn subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Error::arity_error("-", 1, 0));
    }

    // Check all arguments are numbers first
    for (i, arg) in args.iter().enumerate() {
        if !arg.is_number() {
            return Err(Error::type_error(
                "-",
                "number",
                arg.type_name(),
                Some(i + 1),
            ));
        }
    }

    let first = args[0].as_number().unwrap();

    if args.len() == 1 {
        // Unary minus - negate the number
        Ok(Value::number(-first))
    } else {
        // Subtract all subsequent arguments from the first
        let mut result = first;
        for arg in &args[1..] {
            result -= arg.as_number().unwrap();
        }
        Ok(Value::number(result))
    }
}

/// Multiply numbers
///
/// Scheme: (* number1 number2 ...)
/// Returns the product of all arguments. With no arguments, returns 1.
pub fn multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::number(1.0));
    }

    let mut product = 1.0;
    for arg in args {
        match arg {
            Value::Number(n) => product *= n.value(),
            _ => {
                return Err(Error::runtime_error(&format!(
                    "'*' requires numeric arguments, got {}",
                    arg.type_name()
                )));
            }
        }
    }
    Ok(Value::number(product))
}

/// Divide numbers
///
/// Scheme: (/ number1 number2 ...)
/// With one argument, returns its reciprocal (1/number).
/// With multiple arguments, divides the first by all subsequent arguments.
pub fn divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Error::arity_error("/", 1, 0));
    }

    // Check all arguments are numbers first
    for (i, arg) in args.iter().enumerate() {
        if !arg.is_number() {
            return Err(Error::type_error(
                "/",
                "number",
                arg.type_name(),
                Some(i + 1),
            ));
        }
    }

    let first = args[0].as_number().unwrap();

    if args.len() == 1 {
        // Unary division - reciprocal
        if first == 0.0 {
            return Err(Error::runtime_error("Division by zero"));
        }
        Ok(Value::number(1.0 / first))
    } else {
        // Divide first by all subsequent arguments
        let mut result = first;
        for arg in &args[1..] {
            let divisor = arg.as_number().unwrap();
            if divisor == 0.0 {
                return Err(Error::runtime_error("Division by zero"));
            }
            result /= divisor;
        }
        Ok(Value::number(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        // No arguments - should return 0
        let result = add(&[]).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);

        // Single argument
        let result = add(&[Value::number(5.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);

        // Multiple arguments
        let result = add(&[Value::number(1.0), Value::number(2.0), Value::number(3.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 6.0);

        // Non-numeric argument should error
        let result = add(&[Value::number(1.0), Value::string("hello")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_subtract() {
        // No arguments should error
        let result = subtract(&[]);
        assert!(result.is_err());

        // Single argument - unary minus
        let result = subtract(&[Value::number(5.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), -5.0);

        // Multiple arguments
        let result =
            subtract(&[Value::number(10.0), Value::number(3.0), Value::number(2.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);

        // Non-numeric argument should error
        let result = subtract(&[Value::boolean(true)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiply() {
        // No arguments - should return 1
        let result = multiply(&[]).unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);

        // Single argument
        let result = multiply(&[Value::number(5.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);

        // Multiple arguments
        let result =
            multiply(&[Value::number(2.0), Value::number(3.0), Value::number(4.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 24.0);

        // Non-numeric argument should error
        let result = multiply(&[Value::number(2.0), Value::symbol("x")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_divide() {
        // No arguments should error
        let result = divide(&[]);
        assert!(result.is_err());

        // Single argument - reciprocal
        let result = divide(&[Value::number(4.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.25);

        // Multiple arguments
        let result =
            divide(&[Value::number(24.0), Value::number(3.0), Value::number(2.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 4.0);

        // Division by zero should error
        let result = divide(&[Value::number(5.0), Value::number(0.0)]);
        assert!(result.is_err());

        // Reciprocal of zero should error
        let result = divide(&[Value::number(0.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_checking() {
        // All operations should reject non-numeric types
        let string_arg = Value::string("not a number");
        let bool_arg = Value::boolean(true);
        let symbol_arg = Value::symbol("x");

        assert!(add(&[Value::number(1.0), string_arg.clone()]).is_err());
        assert!(subtract(&[bool_arg.clone()]).is_err());
        assert!(multiply(&[Value::number(1.0), symbol_arg.clone()]).is_err());
        assert!(divide(&[string_arg.clone()]).is_err());
    }

    #[test]
    fn test_edge_cases() {
        // Test with zero
        let result = add(&[Value::number(0.0), Value::number(5.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), 5.0);

        // Test with negative numbers
        let result = multiply(&[Value::number(-2.0), Value::number(3.0)]).unwrap();
        assert_eq!(result.as_number().unwrap(), -6.0);

        // Test floating point precision
        let result = divide(&[Value::number(1.0), Value::number(3.0)]).unwrap();
        let expected = 1.0 / 3.0;
        assert!((result.as_number().unwrap() - expected).abs() < f64::EPSILON);
    }
}
