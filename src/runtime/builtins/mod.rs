//! Builtin procedures for the Twine Scheme runtime
//!
//! This module contains all built-in procedures organized by category.
//! These procedures are automatically available in the global environment.

use crate::error::Result;
use crate::types::Value;

/// Enumeration of all built-in procedures
///
/// Each variant represents a specific built-in procedure, eliminating the need
/// to store both function pointers and names. This provides type safety and
/// eliminates redundancy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Builtin {
    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,

    // Comparison operations
    Equal,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,

    // List operations
    Car,
    Cdr,
    Cons,
    List,
    NullP,

    // I/O operations
    Display,
    Newline,
}

impl Builtin {
    /// Get the display name for this builtin procedure
    pub fn name(self) -> &'static str {
        match self {
            Builtin::Add => "+",
            Builtin::Subtract => "-",
            Builtin::Multiply => "*",
            Builtin::Divide => "/",
            Builtin::Equal => "=",
            Builtin::LessThan => "<",
            Builtin::GreaterThan => ">",
            Builtin::LessThanOrEqual => "<=",
            Builtin::GreaterThanOrEqual => ">=",
            Builtin::Car => "car",
            Builtin::Cdr => "cdr",
            Builtin::Cons => "cons",
            Builtin::List => "list",
            Builtin::NullP => "null?",
            Builtin::Display => "display",
            Builtin::Newline => "newline",
        }
    }

    /// Execute this builtin procedure with the given arguments
    pub fn call(self, args: &[Value]) -> Result<Value> {
        match self {
            Builtin::Add => add(args),
            Builtin::Subtract => subtract(args),
            Builtin::Multiply => multiply(args),
            Builtin::Divide => divide(args),
            Builtin::Equal => equal(args),
            Builtin::LessThan => less_than(args),
            Builtin::GreaterThan => greater_than(args),
            Builtin::LessThanOrEqual => less_than_or_equal(args),
            Builtin::GreaterThanOrEqual => greater_than_or_equal(args),
            Builtin::Car => car(args),
            Builtin::Cdr => cdr(args),
            Builtin::Cons => cons(args),
            Builtin::List => list(args),
            Builtin::NullP => null_p(args),
            Builtin::Display => display(args),
            Builtin::Newline => newline(args),
        }
    }

    /// Parse a builtin procedure name into its corresponding Builtin
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "+" => Some(Builtin::Add),
            "-" => Some(Builtin::Subtract),
            "*" => Some(Builtin::Multiply),
            "/" => Some(Builtin::Divide),
            "=" => Some(Builtin::Equal),
            "<" => Some(Builtin::LessThan),
            ">" => Some(Builtin::GreaterThan),
            "<=" => Some(Builtin::LessThanOrEqual),
            ">=" => Some(Builtin::GreaterThanOrEqual),
            "car" => Some(Builtin::Car),
            "cdr" => Some(Builtin::Cdr),
            "cons" => Some(Builtin::Cons),
            "list" => Some(Builtin::List),
            "null?" => Some(Builtin::NullP),
            "display" => Some(Builtin::Display),
            "newline" => Some(Builtin::Newline),
            _ => None,
        }
    }
}

pub mod arithmetic;
pub mod comparison;
pub mod io;
pub mod list;

// Re-export arithmetic functions for convenience
pub use arithmetic::{add, divide, multiply, subtract};

// Re-export comparison functions for convenience
pub use comparison::{equal, greater_than, greater_than_or_equal, less_than, less_than_or_equal};

// Re-export list functions for convenience
pub use list::{car, cdr, cons, list, null_p};

// Re-export I/O functions for convenience
pub use io::{display, newline};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_arithmetic_operations() {
        // Test addition
        let builtin = Builtin::from_name("+").unwrap();
        let result = builtin
            .call(&[Value::number(1.0), Value::number(2.0)])
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        // Test subtraction
        let builtin = Builtin::from_name("-").unwrap();
        let result = builtin
            .call(&[Value::number(5.0), Value::number(3.0)])
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);

        // Test multiplication
        let builtin = Builtin::from_name("*").unwrap();
        let result = builtin
            .call(&[Value::number(3.0), Value::number(4.0)])
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 12.0);

        // Test division
        let builtin = Builtin::from_name("/").unwrap();
        let result = builtin
            .call(&[Value::number(8.0), Value::number(2.0)])
            .unwrap();
        assert_eq!(result.as_number().unwrap(), 4.0);
    }

    #[test]
    fn test_builtin_comparison_operations() {
        // Test equality
        let builtin = Builtin::from_name("=").unwrap();
        let result = builtin
            .call(&[Value::number(5.0), Value::number(5.0)])
            .unwrap();
        assert!(result.as_boolean().unwrap());

        // Test less than
        let builtin = Builtin::from_name("<").unwrap();
        let result = builtin
            .call(&[Value::number(3.0), Value::number(5.0)])
            .unwrap();
        assert!(result.as_boolean().unwrap());

        // Test greater than
        let builtin = Builtin::from_name(">").unwrap();
        let result = builtin
            .call(&[Value::number(5.0), Value::number(3.0)])
            .unwrap();
        assert!(result.as_boolean().unwrap());

        // Test less than or equal
        let builtin = Builtin::from_name("<=").unwrap();
        let result = builtin
            .call(&[Value::number(3.0), Value::number(3.0)])
            .unwrap();
        assert!(result.as_boolean().unwrap());

        // Test greater than or equal
        let builtin = Builtin::from_name(">=").unwrap();
        let result = builtin
            .call(&[Value::number(5.0), Value::number(3.0)])
            .unwrap();
        assert!(result.as_boolean().unwrap());
    }

    #[test]
    fn test_unknown_builtin_procedure() {
        // Unknown procedure should return None
        let result = Builtin::from_name("unknown-proc");
        assert!(result.is_none());

        // Test with unknown builtin
        let result = Builtin::from_name("unknown-builtin");
        assert!(result.is_none());
    }

    #[test]
    fn test_builtin_error_propagation() {
        // Test that errors from builtin functions are properly propagated
        let builtin = Builtin::from_name("+").unwrap();
        let result = builtin.call(&[Value::number(1.0), Value::string("not a number")]);
        assert!(result.is_err());

        // Test arity errors
        let builtin = Builtin::from_name("=").unwrap();
        let result = builtin.call(&[Value::number(1.0)]);
        assert!(result.is_err());

        // Test division by zero
        let builtin = Builtin::from_name("/").unwrap();
        let result = builtin.call(&[Value::number(1.0), Value::number(0.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_list_operations() {
        // Test car
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let builtin = Builtin::from_name("car").unwrap();
        let result = builtin.call(&[list]).unwrap();
        assert_eq!(result, Value::number(1.0));

        // Test cdr
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let builtin = Builtin::from_name("cdr").unwrap();
        let result = builtin.call(&[list]).unwrap();
        assert_eq!(result, Value::list(vec![Value::number(2.0)]));

        // Test cons
        let tail = Value::list(vec![Value::number(2.0)]);
        let builtin = Builtin::from_name("cons").unwrap();
        let result = builtin.call(&[Value::number(1.0), tail]).unwrap();
        assert_eq!(
            result,
            Value::list(vec![Value::number(1.0), Value::number(2.0)])
        );

        // Test list
        let builtin = Builtin::from_name("list").unwrap();
        let result = builtin
            .call(&[Value::number(1.0), Value::string("hello")])
            .unwrap();
        assert_eq!(
            result,
            Value::list(vec![Value::number(1.0), Value::string("hello")])
        );

        // Test null?
        let empty = Value::empty_list();
        let builtin = Builtin::from_name("null?").unwrap();
        let result = builtin.call(&[empty]).unwrap();
        assert_eq!(result, Value::boolean(true));

        let non_empty = Value::list(vec![Value::number(1.0)]);
        let result = builtin.call(&[non_empty]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_builtin_name() {
        assert_eq!(Builtin::Add.name(), "+");
        assert_eq!(Builtin::Subtract.name(), "-");
        assert_eq!(Builtin::Multiply.name(), "*");
        assert_eq!(Builtin::Divide.name(), "/");
        assert_eq!(Builtin::Equal.name(), "=");
        assert_eq!(Builtin::LessThan.name(), "<");
        assert_eq!(Builtin::GreaterThan.name(), ">");
        assert_eq!(Builtin::LessThanOrEqual.name(), "<=");
        assert_eq!(Builtin::GreaterThanOrEqual.name(), ">=");
        assert_eq!(Builtin::Car.name(), "car");
        assert_eq!(Builtin::Cdr.name(), "cdr");
        assert_eq!(Builtin::Cons.name(), "cons");
        assert_eq!(Builtin::List.name(), "list");
        assert_eq!(Builtin::NullP.name(), "null?");
        assert_eq!(Builtin::Display.name(), "display");
        assert_eq!(Builtin::Newline.name(), "newline");
    }

    #[test]
    fn test_builtin_from_name() {
        assert_eq!(Builtin::from_name("+"), Some(Builtin::Add));
        assert_eq!(Builtin::from_name("-"), Some(Builtin::Subtract));
        assert_eq!(Builtin::from_name("*"), Some(Builtin::Multiply));
        assert_eq!(Builtin::from_name("/"), Some(Builtin::Divide));
        assert_eq!(Builtin::from_name("="), Some(Builtin::Equal));
        assert_eq!(Builtin::from_name("<"), Some(Builtin::LessThan));
        assert_eq!(Builtin::from_name(">"), Some(Builtin::GreaterThan));
        assert_eq!(Builtin::from_name("<="), Some(Builtin::LessThanOrEqual));
        assert_eq!(Builtin::from_name(">="), Some(Builtin::GreaterThanOrEqual));
        assert_eq!(Builtin::from_name("car"), Some(Builtin::Car));
        assert_eq!(Builtin::from_name("cdr"), Some(Builtin::Cdr));
        assert_eq!(Builtin::from_name("cons"), Some(Builtin::Cons));
        assert_eq!(Builtin::from_name("list"), Some(Builtin::List));
        assert_eq!(Builtin::from_name("null?"), Some(Builtin::NullP));
        assert_eq!(Builtin::from_name("display"), Some(Builtin::Display));
        assert_eq!(Builtin::from_name("newline"), Some(Builtin::Newline));

        // Test unknown names
        assert_eq!(Builtin::from_name("unknown"), None);
        assert_eq!(Builtin::from_name(""), None);
        assert_eq!(Builtin::from_name("foo"), None);
    }

    #[test]
    fn test_builtin_call() {
        // Test arithmetic operations
        let result = Builtin::Add
            .call(&[Value::number(1.0), Value::number(2.0)])
            .unwrap();
        assert_eq!(result, Value::number(3.0));

        let result = Builtin::Subtract
            .call(&[Value::number(5.0), Value::number(3.0)])
            .unwrap();
        assert_eq!(result, Value::number(2.0));

        let result = Builtin::Multiply
            .call(&[Value::number(3.0), Value::number(4.0)])
            .unwrap();
        assert_eq!(result, Value::number(12.0));

        let result = Builtin::Divide
            .call(&[Value::number(8.0), Value::number(2.0)])
            .unwrap();
        assert_eq!(result, Value::number(4.0));

        // Test comparison operations
        let result = Builtin::Equal
            .call(&[Value::number(5.0), Value::number(5.0)])
            .unwrap();
        assert_eq!(result, Value::boolean(true));

        let result = Builtin::LessThan
            .call(&[Value::number(3.0), Value::number(5.0)])
            .unwrap();
        assert_eq!(result, Value::boolean(true));

        let result = Builtin::GreaterThan
            .call(&[Value::number(5.0), Value::number(3.0)])
            .unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test list operations
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let result = Builtin::Car.call(&[list]).unwrap();
        assert_eq!(result, Value::number(1.0));

        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let result = Builtin::Cdr.call(&[list]).unwrap();
        assert_eq!(result, Value::list(vec![Value::number(2.0)]));

        let tail = Value::list(vec![Value::number(2.0)]);
        let result = Builtin::Cons.call(&[Value::number(1.0), tail]).unwrap();
        assert_eq!(
            result,
            Value::list(vec![Value::number(1.0), Value::number(2.0)])
        );

        let result = Builtin::List
            .call(&[Value::number(1.0), Value::string("hello")])
            .unwrap();
        assert_eq!(
            result,
            Value::list(vec![Value::number(1.0), Value::string("hello")])
        );

        let empty = Value::empty_list();
        let result = Builtin::NullP.call(&[empty]).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test I/O operations
        let result = Builtin::Display.call(&[Value::string("test")]).unwrap();
        assert_eq!(result, Value::Nil);

        let result = Builtin::Newline.call(&[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_builtin_call_errors() {
        // Test error propagation for invalid arguments
        let result = Builtin::Add.call(&[Value::number(1.0), Value::string("not a number")]);
        assert!(result.is_err());

        let result = Builtin::Equal.call(&[Value::number(1.0)]);
        assert!(result.is_err());

        let result = Builtin::Divide.call(&[Value::number(1.0), Value::number(0.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_builtin_equality_and_hash() {
        use std::collections::HashSet;

        // Test equality
        assert_eq!(Builtin::Add, Builtin::Add);
        assert_ne!(Builtin::Add, Builtin::Subtract);

        // Test that they can be used in HashSet (implements Hash + Eq)
        let mut set = HashSet::new();
        set.insert(Builtin::Add);
        set.insert(Builtin::Subtract);
        set.insert(Builtin::Add); // Duplicate should not increase size

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Builtin::Add));
        assert!(set.contains(&Builtin::Subtract));
        assert!(!set.contains(&Builtin::Multiply));
    }

    #[test]
    fn test_builtin_debug() {
        let debug_output = format!("{:?}", Builtin::Add);
        assert_eq!(debug_output, "Add");

        let debug_output = format!("{:?}", Builtin::NullP);
        assert_eq!(debug_output, "NullP");
    }

    #[test]
    fn test_builtin_copy_clone() {
        let original = Builtin::Add;
        let copied = original; // Copy trait
        let cloned = original; // Clone trait

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
        assert_eq!(copied, cloned);
    }

    #[test]
    fn test_builtin_enum_usage() {
        // Test that Builtin enum works correctly
        let builtin = Builtin::from_name("+").unwrap();
        let result = builtin
            .call(&[Value::number(1.0), Value::number(2.0)])
            .unwrap();
        assert_eq!(result, Value::number(3.0));

        // Test that unknown procedures return None
        let result = Builtin::from_name("unknown-builtin");
        assert!(result.is_none());
    }
}
