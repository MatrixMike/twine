//! List manipulation procedures for the Twine Scheme runtime
//!
//! This module implements the fundamental list operations in Scheme:
//! - `car`: Get the first element of a list
//! - `cdr`: Get the rest of a list (all elements except the first)
//! - `cons`: Construct a new list by prepending an element
//! - `list`: Create a new list from multiple arguments
//! - `null?`: Check if a value is the empty list
//! - `length`: Get the number of elements in a list

use crate::error::{Error, Result};
use crate::types::Value;

/// Get the first element of a list (car)
///
/// In Scheme, `car` returns the first element of a non-empty list.
/// It is an error to call `car` on an empty list or a non-list value.
///
/// # Arguments
/// * `args` - Should contain exactly one argument that is a non-empty list
///
/// # Returns
/// * `Ok(Value)` - The first element of the list
/// * `Err(SchemeError)` - If wrong number of arguments, not a list, or empty list
///
/// # Examples
/// ```scheme
/// (car '(1 2 3))     ; => 1
/// (car '((a b) c))   ; => (a b)
/// ```
pub fn car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("car", 1, args.len()));
    }

    let list = args[0]
        .as_list()
        .ok_or_else(|| Error::type_error("car", "list", args[0].type_name(), None))?;

    if list.is_empty() {
        return Err(Error::runtime_error("car: cannot take car of empty list"));
    }

    Ok(list.get(0).unwrap().clone())
}

/// Get the rest of a list (cdr)
///
/// In Scheme, `cdr` returns a new list containing all elements except the first.
/// It is an error to call `cdr` on an empty list or a non-list value.
///
/// # Arguments
/// * `args` - Should contain exactly one argument that is a non-empty list
///
/// # Returns
/// * `Ok(Value)` - A new list with all elements except the first
/// * `Err(SchemeError)` - If wrong number of arguments, not a list, or empty list
///
/// # Examples
/// ```scheme
/// (cdr '(1 2 3))     ; => (2 3)
/// (cdr '(a))         ; => ()
/// ```
pub fn cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("cdr", 1, args.len()));
    }

    let list = args[0]
        .as_list()
        .ok_or_else(|| Error::type_error("cdr", "list", args[0].type_name(), None))?;

    if list.is_empty() {
        return Err(Error::runtime_error("cdr: cannot take cdr of empty list"));
    }

    // Create a new list with all elements except the first
    let rest_values: Vec<Value> = list.iter().skip(1).cloned().collect();
    Ok(Value::list(rest_values))
}

/// Construct a new list by prepending an element (cons)
///
/// In Scheme, `cons` creates a new list with the first argument as the head
/// and the second argument as the tail. The second argument should be a list.
///
/// # Arguments
/// * `args` - Should contain exactly two arguments: element and list
///
/// # Returns
/// * `Ok(Value)` - A new list with the element prepended
/// * `Err(SchemeError)` - If wrong number of arguments or second arg is not a list
///
/// # Examples
/// ```scheme
/// (cons 1 '(2 3))    ; => (1 2 3)
/// (cons 'a '())      ; => (a)
/// ```
pub fn cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Error::arity_error("cons", 2, args.len()));
    }

    let element = &args[0];
    let tail = args[1]
        .as_list()
        .ok_or_else(|| Error::type_error("cons", "list", args[1].type_name(), Some(2)))?;

    // Create a new list with the element prepended
    let mut new_values = Vec::with_capacity(tail.len() + 1);
    new_values.push(element.clone());
    new_values.extend(tail.iter().cloned());

    Ok(Value::list(new_values))
}

/// Create a new list from multiple arguments (list)
///
/// In Scheme, `list` creates a new list containing all its arguments as elements.
/// This is a convenient way to create lists without using quote.
///
/// # Arguments
/// * `args` - Any number of arguments to include in the list
///
/// # Returns
/// * `Ok(Value)` - A new list containing all arguments as elements
///
/// # Examples
/// ```scheme
/// (list 1 2 3)       ; => (1 2 3)
/// (list)             ; => ()
/// (list 'a (+ 1 2))  ; => (a 3)
/// ```
pub fn list(args: &[Value]) -> Result<Value> {
    Ok(Value::list(args.to_vec()))
}

/// Check if a value is the empty list (null?)
///
/// In Scheme, `null?` returns `#t` if the argument is the empty list, `#f` otherwise.
/// Note that in this implementation, we check for empty lists specifically.
///
/// # Arguments
/// * `args` - Should contain exactly one argument
///
/// # Returns
/// * `Ok(Value)` - `#t` if the argument is an empty list, `#f` otherwise
/// * `Err(SchemeError)` - If wrong number of arguments
///
/// # Examples
/// ```scheme
/// (null? '())        ; => #t
/// (null? '(1 2))     ; => #f
/// (null? 42)         ; => #f
/// ```
pub fn null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("null?", 1, args.len()));
    }

    let is_null = match &args[0] {
        Value::List(list) => list.is_empty(),
        _ => false,
    };

    Ok(Value::boolean(is_null))
}

/// Get the number of elements in a list (length)
///
/// In Scheme, `length` returns the number of elements in a list.
/// It only accepts lists as arguments.
///
/// # Arguments
/// * `args` - Should contain exactly one argument that is a list
///
/// # Returns
/// * `Ok(Value)` - A number representing the length of the list
/// * `Err(SchemeError)` - If wrong number of arguments or not a list
///
/// # Examples
/// ```scheme
/// (length '())       ; => 0
/// (length '(1 2 3))  ; => 3
/// (length '(a))      ; => 1
/// ```
pub fn length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Error::arity_error("length", 1, args.len()));
    }

    let list = args[0]
        .as_list()
        .ok_or_else(|| Error::type_error("length", "list", args[0].type_name(), None))?;

    Ok(Value::number(list.len() as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_car() {
        // Test normal operation
        let list = Value::list(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        let result = car(&[list]).unwrap();
        assert_eq!(result, Value::number(1.0));

        // Test with different types
        let mixed_list = Value::list(vec![
            Value::string("hello"),
            Value::number(42.0),
            Value::boolean(true),
        ]);
        let result = car(&[mixed_list]).unwrap();
        assert_eq!(result, Value::string("hello"));

        // Test with nested list
        let nested = Value::list(vec![
            Value::list(vec![Value::number(1.0), Value::number(2.0)]),
            Value::number(3.0),
        ]);
        let result = car(&[nested]).unwrap();
        assert_eq!(
            result,
            Value::list(vec![Value::number(1.0), Value::number(2.0)])
        );
    }

    #[test]
    fn test_car_errors() {
        // Test wrong arity
        let result = car(&[]);
        assert!(result.is_err());

        let result = car(&[Value::number(1.0), Value::number(2.0)]);
        assert!(result.is_err());

        // Test non-list argument
        let result = car(&[Value::number(42.0)]);
        assert!(result.is_err());

        let result = car(&[Value::string("not a list")]);
        assert!(result.is_err());

        // Test empty list
        let empty_list = Value::empty_list();
        let result = car(&[empty_list]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cdr() {
        // Test normal operation
        let list = Value::list(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        let result = cdr(&[list]).unwrap();
        let expected = Value::list(vec![Value::number(2.0), Value::number(3.0)]);
        assert_eq!(result, expected);

        // Test single element list
        let single = Value::list(vec![Value::string("only")]);
        let result = cdr(&[single]).unwrap();
        assert_eq!(result, Value::empty_list());

        // Test with mixed types
        let mixed_list = Value::list(vec![
            Value::boolean(false),
            Value::string("world"),
            Value::number(3.14),
        ]);
        let result = cdr(&[mixed_list]).unwrap();
        let expected = Value::list(vec![Value::string("world"), Value::number(3.14)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cdr_errors() {
        // Test wrong arity
        let result = cdr(&[]);
        assert!(result.is_err());

        let result = cdr(&[Value::number(1.0), Value::number(2.0)]);
        assert!(result.is_err());

        // Test non-list argument
        let result = cdr(&[Value::boolean(true)]);
        assert!(result.is_err());

        // Test empty list
        let empty_list = Value::empty_list();
        let result = cdr(&[empty_list]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cons() {
        // Test normal operation
        let tail = Value::list(vec![Value::number(2.0), Value::number(3.0)]);
        let result = cons(&[Value::number(1.0), tail]).unwrap();
        let expected = Value::list(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        assert_eq!(result, expected);

        // Test cons with empty list
        let empty = Value::empty_list();
        let result = cons(&[Value::string("first"), empty]).unwrap();
        let expected = Value::list(vec![Value::string("first")]);
        assert_eq!(result, expected);

        // Test cons with mixed types
        let tail = Value::list(vec![Value::boolean(false), Value::number(42.0)]);
        let result = cons(&[Value::symbol("start"), tail]).unwrap();
        let expected = Value::list(vec![
            Value::symbol("start"),
            Value::boolean(false),
            Value::number(42.0),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_cons_errors() {
        // Test wrong arity
        let result = cons(&[Value::number(1.0)]);
        assert!(result.is_err());

        let result = cons(&[Value::number(1.0), Value::number(2.0), Value::number(3.0)]);
        assert!(result.is_err());

        // Test non-list second argument
        let result = cons(&[Value::number(1.0), Value::number(2.0)]);
        assert!(result.is_err());

        let result = cons(&[Value::string("a"), Value::string("b")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list() {
        // Test empty list creation
        let result = list(&[]).unwrap();
        assert_eq!(result, Value::empty_list());

        // Test single element
        let result = list(&[Value::number(42.0)]).unwrap();
        let expected = Value::list(vec![Value::number(42.0)]);
        assert_eq!(result, expected);

        // Test multiple elements
        let args = vec![
            Value::number(1.0),
            Value::string("hello"),
            Value::boolean(true),
            Value::symbol("world"),
        ];
        let result = list(&args).unwrap();
        let expected = Value::list(args);
        assert_eq!(result, expected);

        // Test with nested lists
        let inner_list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let args = vec![inner_list.clone(), Value::number(3.0)];
        let result = list(&args).unwrap();
        let expected = Value::list(vec![inner_list, Value::number(3.0)]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_null_p() {
        // Test empty list
        let empty = Value::empty_list();
        let result = null_p(&[empty]).unwrap();
        assert_eq!(result, Value::boolean(true));

        // Test non-empty list
        let non_empty = Value::list(vec![Value::number(1.0)]);
        let result = null_p(&[non_empty]).unwrap();
        assert_eq!(result, Value::boolean(false));

        // Test non-list values
        let result = null_p(&[Value::number(42.0)]).unwrap();
        assert_eq!(result, Value::boolean(false));

        let result = null_p(&[Value::string("hello")]).unwrap();
        assert_eq!(result, Value::boolean(false));

        let result = null_p(&[Value::boolean(true)]).unwrap();
        assert_eq!(result, Value::boolean(false));

        let result = null_p(&[Value::symbol("test")]).unwrap();
        assert_eq!(result, Value::boolean(false));

        let result = null_p(&[Value::nil()]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_null_p_errors() {
        // Test wrong arity
        let result = null_p(&[]);
        assert!(result.is_err());

        let result = null_p(&[Value::empty_list(), Value::number(1.0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_operations_integration() {
        // Test car and cdr working together
        let original = Value::list(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        let first = car(&[original.clone()]).unwrap();
        let rest = cdr(&[original]).unwrap();

        assert_eq!(first, Value::number(1.0));
        assert_eq!(
            rest,
            Value::list(vec![Value::number(2.0), Value::number(3.0)])
        );

        // Test cons reconstructing a list
        let reconstructed = cons(&[first, rest]).unwrap();
        let expected = Value::list(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);
        assert_eq!(reconstructed, expected);
    }

    #[test]
    fn test_edge_cases() {
        // Test single element list operations
        let single = Value::list(vec![Value::string("only")]);
        let first = car(&[single.clone()]).unwrap();
        let rest = cdr(&[single]).unwrap();

        assert_eq!(first, Value::string("only"));
        assert_eq!(rest, Value::empty_list());
        assert_eq!(null_p(&[rest]).unwrap(), Value::boolean(true));

        // Test cons creating from nothing
        let from_cons = cons(&[Value::number(1.0), Value::empty_list()]).unwrap();
        let expected = Value::list(vec![Value::number(1.0)]);
        assert_eq!(from_cons, expected);
    }

    #[test]
    fn test_length() {
        // Test empty list
        let empty = Value::empty_list();
        let result = length(&[empty]).unwrap();
        assert_eq!(result, Value::number(0.0));

        // Test single element
        let single = Value::list(vec![Value::number(42.0)]);
        let result = length(&[single]).unwrap();
        assert_eq!(result, Value::number(1.0));

        // Test multiple elements
        let multiple = Value::list(vec![
            Value::number(1.0),
            Value::string("hello"),
            Value::boolean(true),
        ]);
        let result = length(&[multiple]).unwrap();
        assert_eq!(result, Value::number(3.0));

        // Test nested lists (they still count as single elements)
        let nested = Value::list(vec![
            Value::list(vec![Value::number(1.0), Value::number(2.0)]),
            Value::number(3.0),
        ]);
        let result = length(&[nested]).unwrap();
        assert_eq!(result, Value::number(2.0));
    }

    #[test]
    fn test_length_errors() {
        // Test wrong arity
        let result = length(&[]);
        assert!(result.is_err());

        let result = length(&[Value::empty_list(), Value::number(1.0)]);
        assert!(result.is_err());

        // Test non-list argument
        let result = length(&[Value::number(42.0)]);
        assert!(result.is_err());

        let result = length(&[Value::string("not a list")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_checking() {
        // Verify that type errors give meaningful messages
        let not_list = Value::number(42.0);

        let car_err = car(&[not_list.clone()]).unwrap_err();
        assert!(car_err.to_string().contains("expected list"));

        let cdr_err = cdr(&[not_list.clone()]).unwrap_err();
        assert!(cdr_err.to_string().contains("expected list"));

        let cons_err = cons(&[Value::number(1.0), not_list.clone()]).unwrap_err();
        assert!(cons_err.to_string().contains("expected list"));

        let length_err = length(&[not_list]).unwrap_err();
        assert!(length_err.to_string().contains("expected list"));
    }
}
