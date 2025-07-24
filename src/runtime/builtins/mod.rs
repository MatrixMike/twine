//! Builtin procedures for the Twine Scheme runtime
//!
//! This module contains all built-in procedures organized by category.
//! These procedures are automatically available in the global environment.

use crate::error::Result;
use crate::types::{Symbol, Value};

pub mod arithmetic;
pub mod list;

// Re-export arithmetic functions for convenience
pub use arithmetic::{
    add, divide, equal, greater_than, greater_than_or_equal, less_than, less_than_or_equal,
    multiply, subtract,
};

// Re-export list functions for convenience
pub use list::{car, cdr, cons, list, null_p};

/// Dispatch a builtin procedure call
///
/// This function serves as the central dispatch point for all builtin procedures.
/// It returns `Some(result)` if the symbol corresponds to a builtin procedure,
/// or `None` if the symbol is not a builtin.
///
/// # Arguments
/// * `identifier` - The procedure name as a Symbol
/// * `args` - The evaluated arguments to pass to the procedure
///
/// # Returns
/// * `Option<Result<Value>>` - Some(result) for builtins, None for unknown identifiers
pub fn dispatch(identifier: &Symbol, args: &[Value]) -> Option<Result<Value>> {
    match identifier.as_str() {
        // Arithmetic operations
        "+" => Some(add(args)),
        "-" => Some(subtract(args)),
        "*" => Some(multiply(args)),
        "/" => Some(divide(args)),

        // Comparison operations
        "=" => Some(equal(args)),
        "<" => Some(less_than(args)),
        ">" => Some(greater_than(args)),
        "<=" => Some(less_than_or_equal(args)),
        ">=" => Some(greater_than_or_equal(args)),

        // List operations
        "car" => Some(car(args)),
        "cdr" => Some(cdr(args)),
        "cons" => Some(cons(args)),
        "list" => Some(list(args)),
        "null?" => Some(null_p(args)),

        // Return None for unknown identifiers - not a builtin procedure
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_arithmetic_operations() {
        // Test addition
        let result = dispatch(&Symbol::new("+"), &[Value::number(1.0), Value::number(2.0)])
            .unwrap()
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        // Test subtraction
        let result = dispatch(&Symbol::new("-"), &[Value::number(5.0), Value::number(3.0)])
            .unwrap()
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);

        // Test multiplication
        let result = dispatch(&Symbol::new("*"), &[Value::number(3.0), Value::number(4.0)])
            .unwrap()
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 12.0);

        // Test division
        let result = dispatch(&Symbol::new("/"), &[Value::number(8.0), Value::number(2.0)])
            .unwrap()
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 4.0);
    }

    #[test]
    fn test_dispatch_comparison_operations() {
        // Test equality
        let result = dispatch(&Symbol::new("="), &[Value::number(5.0), Value::number(5.0)])
            .unwrap()
            .unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test less than
        let result = dispatch(&Symbol::new("<"), &[Value::number(3.0), Value::number(5.0)])
            .unwrap()
            .unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test greater than
        let result = dispatch(&Symbol::new(">"), &[Value::number(5.0), Value::number(3.0)])
            .unwrap()
            .unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test less than or equal
        let result = dispatch(
            &Symbol::new("<="),
            &[Value::number(3.0), Value::number(3.0)],
        )
        .unwrap()
        .unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        // Test greater than or equal
        let result = dispatch(
            &Symbol::new(">="),
            &[Value::number(5.0), Value::number(3.0)],
        )
        .unwrap()
        .unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_dispatch_unknown_procedure() {
        // Unknown procedure should return None
        let result = dispatch(&Symbol::new("unknown-proc"), &[Value::number(1.0)]);
        assert!(result.is_none());

        // Test with unknown builtin
        let result = dispatch(&Symbol::new("unknown-builtin"), &[Value::number(1.0)]);
        assert!(result.is_none());
    }

    #[test]
    fn test_dispatch_error_propagation() {
        // Test that errors from builtin functions are properly propagated
        let result = dispatch(
            &Symbol::new("+"),
            &[Value::number(1.0), Value::string("not a number")],
        )
        .unwrap();
        assert!(result.is_err());

        // Test arity errors
        let result = dispatch(&Symbol::new("="), &[Value::number(1.0)]).unwrap();
        assert!(result.is_err());

        // Test division by zero
        let result =
            dispatch(&Symbol::new("/"), &[Value::number(1.0), Value::number(0.0)]).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_dispatch_list_operations() {
        // Test car
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let result = dispatch(&Symbol::new("car"), &[list]).unwrap().unwrap();
        assert_eq!(result, Value::number(1.0));

        // Test cdr
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let result = dispatch(&Symbol::new("cdr"), &[list]).unwrap().unwrap();
        assert_eq!(result, Value::list(vec![Value::number(2.0)]));

        // Test cons
        let tail = Value::list(vec![Value::number(2.0)]);
        let result = dispatch(&Symbol::new("cons"), &[Value::number(1.0), tail])
            .unwrap()
            .unwrap();
        assert_eq!(
            result,
            Value::list(vec![Value::number(1.0), Value::number(2.0)])
        );

        // Test list
        let result = dispatch(
            &Symbol::new("list"),
            &[Value::number(1.0), Value::string("hello")],
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            result,
            Value::list(vec![Value::number(1.0), Value::string("hello")])
        );

        // Test null?
        let empty = Value::empty_list();
        let result = dispatch(&Symbol::new("null?"), &[empty]).unwrap().unwrap();
        assert_eq!(result, Value::boolean(true));

        let non_empty = Value::list(vec![Value::number(1.0)]);
        let result = dispatch(&Symbol::new("null?"), &[non_empty])
            .unwrap()
            .unwrap();
        assert_eq!(result, Value::boolean(false));
    }
}
