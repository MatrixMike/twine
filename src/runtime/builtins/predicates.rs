//! Type predicate procedures for Scheme values
//!
//! This module implements the type checking procedures required by R7RS-small.
//! These procedures test the type of their single argument and return a boolean.

use crate::error::{Error, Result};
use crate::types::Value;

/// Implements `number?` - tests if value is a number
///
/// Returns `#t` if the argument is a number, `#f` otherwise.
/// Accepts exactly one argument.
///
/// # Arguments
/// * `args` - Vector containing exactly one value to test
///
/// # Returns
/// * `Ok(Value::Boolean(true))` - if argument is a number
/// * `Ok(Value::Boolean(false))` - if argument is not a number
/// * `Err(Error)` - if wrong number of arguments
///
/// # Examples
/// ```scheme
/// (number? 42) => #t
/// (number? 3.14) => #t
/// (number? "hello") => #f
/// ```
pub fn number_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("number?", 1, args.len()));
    }

    let result = matches!(args[0], Value::Number(_));
    Ok(Value::boolean(result))
}

/// Implements `string?` - tests if value is a string
///
/// Returns `#t` if the argument is a string, `#f` otherwise.
/// Accepts exactly one argument.
///
/// # Arguments
/// * `args` - Vector containing exactly one value to test
///
/// # Returns
/// * `Ok(Value::Boolean(true))` - if argument is a string
/// * `Ok(Value::Boolean(false))` - if argument is not a string
/// * `Err(Error)` - if wrong number of arguments
///
/// # Examples
/// ```scheme
/// (string? "hello") => #t
/// (string? "") => #t
/// (string? 42) => #f
/// ```
pub fn string_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("string?", 1, args.len()));
    }

    let result = matches!(args[0], Value::String(_));
    Ok(Value::boolean(result))
}

/// Implements `boolean?` - tests if value is a boolean
///
/// Returns `#t` if the argument is a boolean, `#f` otherwise.
/// Accepts exactly one argument.
///
/// # Arguments
/// * `args` - Vector containing exactly one value to test
///
/// # Returns
/// * `Ok(Value::Boolean(true))` - if argument is a boolean
/// * `Ok(Value::Boolean(false))` - if argument is not a boolean
/// * `Err(Error)` - if wrong number of arguments
///
/// # Examples
/// ```scheme
/// (boolean? #t) => #t
/// (boolean? #f) => #t
/// (boolean? 42) => #f
/// ```
pub fn boolean_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("boolean?", 1, args.len()));
    }

    let result = matches!(args[0], Value::Boolean(_));
    Ok(Value::boolean(result))
}

/// Implements `symbol?` - tests if value is a symbol
///
/// Returns `#t` if the argument is a symbol, `#f` otherwise.
/// Accepts exactly one argument.
///
/// # Arguments
/// * `args` - Vector containing exactly one value to test
///
/// # Returns
/// * `Ok(Value::Boolean(true))` - if argument is a symbol
/// * `Ok(Value::Boolean(false))` - if argument is not a symbol
/// * `Err(Error)` - if wrong number of arguments
///
/// # Examples
/// ```scheme
/// (symbol? 'foo) => #t
/// (symbol? 'hello-world) => #t
/// (symbol? "string") => #f
/// ```
pub fn symbol_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("symbol?", 1, args.len()));
    }

    let result = matches!(args[0], Value::Symbol(_));
    Ok(Value::boolean(result))
}

/// Implements `list?` - tests if value is a list
///
/// Returns `#t` if the argument is a list (including empty list), `#f` otherwise.
/// Accepts exactly one argument.
///
/// # Arguments
/// * `args` - Vector containing exactly one value to test
///
/// # Returns
/// * `Ok(Value::Boolean(true))` - if argument is a list
/// * `Ok(Value::Boolean(false))` - if argument is not a list
/// * `Err(Error)` - if wrong number of arguments
///
/// # Examples
/// ```scheme
/// (list? '(a b c)) => #t
/// (list? '()) => #t
/// (list? 42) => #f
/// ```
pub fn list_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("list?", 1, args.len()));
    }

    let result = matches!(args[0], Value::List(_) | Value::Nil);
    Ok(Value::boolean(result))
}

/// Implements `procedure?` - tests if value is a procedure
///
/// Returns `#t` if the argument is a procedure (builtin or user-defined), `#f` otherwise.
/// Accepts exactly one argument.
///
/// # Arguments
/// * `args` - Vector containing exactly one value to test
///
/// # Returns
/// * `Ok(Value::Boolean(true))` - if argument is a procedure
/// * `Ok(Value::Boolean(false))` - if argument is not a procedure
/// * `Err(Error)` - if wrong number of arguments
///
/// # Examples
/// ```scheme
/// (procedure? +) => #t
/// (procedure? (lambda (x) x)) => #t
/// (procedure? 42) => #f
/// ```
pub fn procedure_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("procedure?", 1, args.len()));
    }

    let result = matches!(args[0], Value::Procedure(_));
    Ok(Value::boolean(result))
}

/// Implements `eq?` - tests if two values are identical
///
/// Returns `#t` if the two arguments are identical, `#f` otherwise.
/// This is similar to `=` but tests identity rather than equality.
/// For symbols and other immutable values, this behaves the same as `=`.
/// Accepts exactly two arguments.
///
/// # Arguments
/// * `args` - Vector containing exactly two values to compare
///
/// # Returns
/// * `Ok(Value::Boolean(true))` - if arguments are identical
/// * `Ok(Value::Boolean(false))` - if arguments are not identical
/// * `Err(Error)` - if wrong number of arguments
///
/// # Examples
/// ```scheme
/// (eq? 'foo 'foo) => #t
/// (eq? 42 42) => #t
/// (eq? "hello" "hello") => #t
/// (eq? 'foo 'bar) => #f
/// ```
pub fn eq_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Error::arity_error("eq?", 2, args.len()));
    }

    // For our implementation, eq? behaves the same as equality comparison
    // since all our values are immutable and we use structural equality
    let result = args[0] == args[1];
    Ok(Value::boolean(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::builtins::Builtin;
    use crate::types::Procedure;

    #[test]
    fn test_number_p() {
        // Test positive cases
        assert_eq!(
            number_p(&[Value::number(42.0)]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            number_p(&[Value::number(0.0)]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            number_p(&[Value::number(-3.14)]).unwrap(),
            Value::boolean(true)
        );

        // Test negative cases
        assert_eq!(
            number_p(&[Value::string("42")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            number_p(&[Value::symbol("number")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            number_p(&[Value::boolean(true)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(number_p(&[Value::Nil]).unwrap(), Value::boolean(false));

        // Test arity error
        assert!(number_p(&[]).is_err());
        assert!(number_p(&[Value::number(1.0), Value::number(2.0)]).is_err());
    }

    #[test]
    fn test_string_p() {
        // Test positive cases
        assert_eq!(
            string_p(&[Value::string("hello")]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            string_p(&[Value::string("")]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            string_p(&[Value::string("with\nescapes")]).unwrap(),
            Value::boolean(true)
        );

        // Test negative cases
        assert_eq!(
            string_p(&[Value::number(42.0)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            string_p(&[Value::symbol("string")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            string_p(&[Value::boolean(false)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(string_p(&[Value::Nil]).unwrap(), Value::boolean(false));

        // Test arity error
        assert!(string_p(&[]).is_err());
        assert!(string_p(&[Value::string("a"), Value::string("b")]).is_err());
    }

    #[test]
    fn test_boolean_p() {
        // Test positive cases
        assert_eq!(
            boolean_p(&[Value::boolean(true)]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            boolean_p(&[Value::boolean(false)]).unwrap(),
            Value::boolean(true)
        );

        // Test negative cases
        assert_eq!(
            boolean_p(&[Value::number(1.0)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            boolean_p(&[Value::string("true")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            boolean_p(&[Value::symbol("boolean")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(boolean_p(&[Value::Nil]).unwrap(), Value::boolean(false));

        // Test arity error
        assert!(boolean_p(&[]).is_err());
        assert!(boolean_p(&[Value::boolean(true), Value::boolean(false)]).is_err());
    }

    #[test]
    fn test_symbol_p() {
        // Test positive cases
        assert_eq!(
            symbol_p(&[Value::symbol("foo")]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            symbol_p(&[Value::symbol("hello-world")]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            symbol_p(&[Value::symbol("x")]).unwrap(),
            Value::boolean(true)
        );

        // Test negative cases
        assert_eq!(
            symbol_p(&[Value::string("foo")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            symbol_p(&[Value::number(42.0)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            symbol_p(&[Value::boolean(true)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(symbol_p(&[Value::Nil]).unwrap(), Value::boolean(false));

        // Test arity error
        assert!(symbol_p(&[]).is_err());
        assert!(symbol_p(&[Value::symbol("a"), Value::symbol("b")]).is_err());
    }

    #[test]
    fn test_list_p() {
        // Test positive cases
        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        assert_eq!(list_p(&[list]).unwrap(), Value::boolean(true));
        assert_eq!(list_p(&[Value::Nil]).unwrap(), Value::boolean(true));

        // Test negative cases
        assert_eq!(
            list_p(&[Value::number(42.0)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            list_p(&[Value::string("list")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            list_p(&[Value::symbol("list")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            list_p(&[Value::boolean(true)]).unwrap(),
            Value::boolean(false)
        );

        // Test arity error
        assert!(list_p(&[]).is_err());
        assert!(list_p(&[Value::Nil, Value::Nil]).is_err());
    }

    #[test]
    fn test_procedure_p() {
        // Test positive case with builtin procedure
        let builtin_proc = Value::procedure(Procedure::builtin(Builtin::Add));
        assert_eq!(procedure_p(&[builtin_proc]).unwrap(), Value::boolean(true));

        // Test negative cases
        assert_eq!(
            procedure_p(&[Value::number(42.0)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            procedure_p(&[Value::string("procedure")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            procedure_p(&[Value::symbol("proc")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            procedure_p(&[Value::boolean(true)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(procedure_p(&[Value::Nil]).unwrap(), Value::boolean(false));

        // Test arity error
        assert!(procedure_p(&[]).is_err());
        let proc1 = Value::procedure(Procedure::builtin(Builtin::Add));
        let proc2 = Value::procedure(Procedure::builtin(Builtin::Subtract));
        assert!(procedure_p(&[proc1, proc2]).is_err());
    }

    #[test]
    fn test_eq_p() {
        // Test positive cases - identical values
        assert_eq!(
            eq_p(&[Value::symbol("foo"), Value::symbol("foo")]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            eq_p(&[Value::number(42.0), Value::number(42.0)]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            eq_p(&[Value::string("hello"), Value::string("hello")]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            eq_p(&[Value::boolean(true), Value::boolean(true)]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            eq_p(&[Value::Nil, Value::Nil]).unwrap(),
            Value::boolean(true)
        );

        // Test negative cases - different values
        assert_eq!(
            eq_p(&[Value::symbol("foo"), Value::symbol("bar")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            eq_p(&[Value::number(42.0), Value::number(43.0)]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            eq_p(&[Value::string("hello"), Value::string("world")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            eq_p(&[Value::boolean(true), Value::boolean(false)]).unwrap(),
            Value::boolean(false)
        );

        // Test different types
        assert_eq!(
            eq_p(&[Value::number(42.0), Value::string("42")]).unwrap(),
            Value::boolean(false)
        );
        assert_eq!(
            eq_p(&[Value::symbol("true"), Value::boolean(true)]).unwrap(),
            Value::boolean(false)
        );

        // Test arity errors
        assert!(eq_p(&[]).is_err());
        assert!(eq_p(&[Value::symbol("foo")]).is_err());
        assert!(
            eq_p(&[
                Value::symbol("foo"),
                Value::symbol("bar"),
                Value::symbol("baz")
            ])
            .is_err()
        );
    }

    #[test]
    fn test_type_checking_comprehensive() {
        // Test that each predicate returns false for all other types
        let test_values = vec![
            Value::number(42.0),
            Value::string("test"),
            Value::boolean(true),
            Value::symbol("test"),
            Value::list(vec![Value::number(1.0)]),
            Value::Nil,
            Value::procedure(Procedure::builtin(Builtin::Add)),
        ];

        for (i, value) in test_values.iter().enumerate() {
            // Each predicate should return true only for its own type
            assert_eq!(
                number_p(&[value.clone()]).unwrap().as_boolean().unwrap(),
                i == 0
            );
            assert_eq!(
                string_p(&[value.clone()]).unwrap().as_boolean().unwrap(),
                i == 1
            );
            assert_eq!(
                boolean_p(&[value.clone()]).unwrap().as_boolean().unwrap(),
                i == 2
            );
            assert_eq!(
                symbol_p(&[value.clone()]).unwrap().as_boolean().unwrap(),
                i == 3
            );
            // list? returns true for both lists and nil
            assert_eq!(
                list_p(&[value.clone()]).unwrap().as_boolean().unwrap(),
                i == 4 || i == 5
            );
            assert_eq!(
                procedure_p(&[value.clone()]).unwrap().as_boolean().unwrap(),
                i == 6
            );
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test with special numbers
        assert_eq!(
            number_p(&[Value::number(f64::INFINITY)]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            number_p(&[Value::number(f64::NEG_INFINITY)]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            number_p(&[Value::number(f64::NAN)]).unwrap(),
            Value::boolean(true)
        );

        // Test with empty string and symbol
        assert_eq!(
            string_p(&[Value::string("")]).unwrap(),
            Value::boolean(true)
        );

        // Test with complex symbol names
        assert_eq!(
            symbol_p(&[Value::symbol("complex-symbol-name!")]).unwrap(),
            Value::boolean(true)
        );
        assert_eq!(
            symbol_p(&[Value::symbol("?")]).unwrap(),
            Value::boolean(true)
        );

        // Test with nested lists
        let nested_list = Value::list(vec![
            Value::list(vec![Value::number(1.0)]),
            Value::number(2.0),
        ]);
        assert_eq!(list_p(&[nested_list]).unwrap(), Value::boolean(true));
    }
}
