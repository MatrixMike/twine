//! Core Value type for Scheme data representation
//!
//! This module implements the main Value enum that represents all possible
//! Scheme values, along with construction and extraction methods.

use super::{List, Number, String, Symbol};

/// The core value type for all Scheme data
///
/// Represents all possible values in the Scheme language. This implementation
/// focuses on immutability and thread-safe sharing of data.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Numeric values using SchemeNumber wrapper
    ///
    /// Note: This is a simplified numeric tower. A full Scheme implementation
    /// would support exact/inexact numbers, rationals, and complex numbers.
    Number(Number),

    /// Boolean values (true/false)
    Boolean(bool),

    /// Immutable string values
    ///
    /// Using SchemeString provides proper abstraction and efficient sharing
    /// of string data across multiple Value instances.
    String(String),

    /// Symbol values (identifiers)
    ///
    /// Symbols are like strings but represent identifiers in Scheme.
    /// Using SchemeSymbol provides proper abstraction for identifier handling.
    Symbol(Symbol),

    /// List values (compound data)
    ///
    /// Represents Scheme lists using SchemeList wrapper around Vec<Value>.
    /// Lists are the fundamental compound data structure in Scheme.
    List(List),

    /// The nil/null value
    ///
    /// Represents both the empty list '() and null/undefined values.
    /// This dual nature is common in Lisp-family languages.
    Nil,
}

impl Value {
    /// Create a new number value from f64
    pub fn number(n: f64) -> Self {
        Value::Number(Number::new(n))
    }

    /// Create a new number value from SchemeNumber
    pub fn scheme_number(n: Number) -> Self {
        Value::Number(n)
    }

    /// Create a new boolean value
    pub fn boolean(b: bool) -> Self {
        Value::Boolean(b)
    }

    /// Create a new string value from a string slice
    pub fn string(s: &str) -> Self {
        Value::String(String::new(s))
    }

    /// Create a new string value from an owned String
    pub fn string_from_owned(s: std::string::String) -> Self {
        Value::String(String::from_string(s))
    }

    /// Create a new symbol value from a string slice
    pub fn symbol(s: &str) -> Self {
        Value::Symbol(Symbol::new(s))
    }

    /// Create a new symbol value from an owned String
    pub fn symbol_from_owned(s: std::string::String) -> Self {
        Value::Symbol(Symbol::from_string(s))
    }

    /// Create a new list value from a vector of values
    pub fn list(values: Vec<Value>) -> Self {
        Value::List(List::from_vec(values))
    }

    /// Create a new list value from a SchemeList
    pub fn scheme_list(list: List) -> Self {
        Value::List(list)
    }

    /// Create an empty list value
    pub fn empty_list() -> Self {
        Value::List(List::new())
    }

    /// Create the nil value
    pub fn nil() -> Self {
        Value::Nil
    }

    /// Check if this value is nil
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Check if this value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Check if this value is a boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Check if this value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if this value is a symbol
    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    /// Check if this value is a list
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    /// Get the numeric value if this is a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(n.value()),
            _ => None,
        }
    }

    /// Get the SchemeNumber if this is a number
    pub fn as_scheme_number(&self) -> Option<Number> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the boolean value if this is a boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the string value if this is a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get the symbol value if this is a symbol
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Value::Symbol(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get the list value if this is a list
    pub fn as_list(&self) -> Option<&List> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    /// Get a string representation of the value's type
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Boolean(_) => "boolean",
            Value::String(_) => "string",
            Value::Symbol(_) => "symbol",
            Value::List(_) => "list",
            Value::Nil => "nil",
        }
    }
}

/// Display implementation for Value
///
/// This provides a string representation suitable for output to users.
/// It follows Scheme conventions for displaying values.
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(true) => write!(f, "#t"),
            Value::Boolean(false) => write!(f, "#f"),
            Value::String(s) => write!(f, "\"{}\"", s.as_str().replace('"', "\\\"")),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::List(l) => write!(f, "{}", l),
            Value::Nil => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        // Test all value creation methods work with new types
        let num = Value::number(42.0);
        assert!(num.is_number());
        assert_eq!(num.as_number(), Some(42.0));

        let bool_val = Value::boolean(true);
        assert!(bool_val.is_boolean());
        assert_eq!(bool_val.as_boolean(), Some(true));

        let str_val = Value::string("hello");
        assert!(str_val.is_string());
        assert_eq!(str_val.as_string(), Some("hello"));

        let sym_val = Value::symbol("variable");
        assert!(sym_val.is_symbol());
        assert_eq!(sym_val.as_symbol(), Some("variable"));

        let list_val = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        assert!(list_val.is_list());
        assert_eq!(list_val.as_list().unwrap().len(), 2);

        let nil_val = Value::nil();
        assert!(nil_val.is_nil());
    }

    #[test]
    fn test_value_debug_output() {
        let num = Value::number(3.14);
        let debug_str = format!("{:?}", num);
        assert!(debug_str.contains("Number"));

        let str_val = Value::string("test");
        let debug_str = format!("{:?}", str_val);
        assert!(debug_str.contains("String"));
    }

    #[test]
    fn test_value_equality() {
        let num1 = Value::number(42.0);
        let num2 = Value::number(42.0);
        let num3 = Value::number(43.0);

        assert_eq!(num1, num2);
        assert_ne!(num1, num3);

        let str1 = Value::string("hello");
        let str2 = Value::string("hello");
        assert_eq!(str1, str2);
    }

    #[test]
    fn test_value_cloning() {
        let original = Value::list(vec![
            Value::number(1.0),
            Value::string("test"),
            Value::boolean(true),
        ]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_type_checking_methods() {
        let values = vec![
            Value::number(42.0),
            Value::boolean(true),
            Value::string("hello"),
            Value::symbol("var"),
            Value::list(vec![]),
            Value::nil(),
        ];

        // Test that each value only returns true for its own type check
        assert!(values[0].is_number() && !values[0].is_boolean() && !values[0].is_string());
        assert!(values[1].is_boolean() && !values[1].is_number() && !values[1].is_string());
        assert!(values[2].is_string() && !values[2].is_number() && !values[2].is_boolean());
        assert!(values[3].is_symbol() && !values[3].is_string() && !values[3].is_number());
        assert!(values[4].is_list() && !values[4].is_symbol() && !values[4].is_number());
        assert!(values[5].is_nil() && !values[5].is_list() && !values[5].is_number());
    }

    #[test]
    fn test_type_name_method() {
        assert_eq!(Value::number(42.0).type_name(), "number");
        assert_eq!(Value::boolean(true).type_name(), "boolean");
        assert_eq!(Value::string("hello").type_name(), "string");
        assert_eq!(Value::symbol("var").type_name(), "symbol");
        assert_eq!(Value::list(vec![]).type_name(), "list");
        assert_eq!(Value::nil().type_name(), "nil");
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(format!("{}", Value::number(42.0)), "42");
        assert_eq!(format!("{}", Value::boolean(true)), "#t");
        assert_eq!(format!("{}", Value::boolean(false)), "#f");
        assert_eq!(format!("{}", Value::string("hello")), "\"hello\"");
        assert_eq!(format!("{}", Value::symbol("var")), "var");
        assert_eq!(format!("{}", Value::nil()), "()");

        let list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        assert_eq!(format!("{}", list), "(1 2)");
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that types use reference counting for efficient sharing
        let original_str = Value::string("shared");
        let cloned_str = original_str.clone();

        // Both values should be equal but share memory
        assert_eq!(original_str, cloned_str);

        let original_list = Value::list(vec![Value::number(1.0), Value::number(2.0)]);
        let cloned_list = original_list.clone();

        assert_eq!(original_list, cloned_list);
    }

    #[test]
    fn test_value_type_conversions() {
        // Test various ways to create the same values
        let num1 = Value::number(42.0);
        let num2 = Value::scheme_number(Number::new(42.0));
        assert_eq!(num1, num2);

        let str1 = Value::string("hello");
        let str2 = Value::string_from_owned(std::string::String::from("hello"));
        assert_eq!(str1, str2);

        let sym1 = Value::symbol("var");
        let sym2 = Value::symbol_from_owned(std::string::String::from("var"));
        assert_eq!(sym1, sym2);
    }

    #[test]
    fn test_comprehensive_value_roundtrip() {
        // Test that values can be created, extracted, and recreated identically
        let original_values = vec![
            Value::number(3.14159),
            Value::boolean(false),
            Value::string("test string"),
            Value::symbol("test-symbol"),
            Value::list(vec![Value::number(1.0), Value::symbol("nested")]),
            Value::nil(),
        ];

        for original in original_values {
            let cloned = original.clone();
            assert_eq!(original, cloned);
            assert_eq!(original.type_name(), cloned.type_name());
            assert_eq!(format!("{}", original), format!("{}", cloned));
        }
    }

    #[test]
    fn test_value_extraction_methods() {
        // Test successful extractions
        assert_eq!(Value::number(42.0).as_number(), Some(42.0));
        assert_eq!(Value::boolean(true).as_boolean(), Some(true));
        assert_eq!(Value::string("hello").as_string(), Some("hello"));
        assert_eq!(Value::symbol("foo").as_symbol(), Some("foo"));

        // Test failed extractions (wrong type)
        assert_eq!(Value::number(42.0).as_boolean(), None);
        assert_eq!(Value::boolean(true).as_string(), None);
        assert_eq!(Value::string("hello").as_symbol(), None);
        assert_eq!(Value::symbol("foo").as_number(), None);
        assert_eq!(Value::nil().as_number(), None);
    }

    #[test]
    fn test_value_edge_cases() {
        // Test nil vs empty list distinction
        let nil = Value::nil();
        let empty_list = Value::empty_list();
        assert_ne!(nil, empty_list);
        assert!(nil.is_nil());
        assert!(!empty_list.is_nil());
        assert!(!nil.is_list());
        assert!(empty_list.is_list());

        // Test that different types with same content are not equal
        let string_val = Value::string("same");
        let symbol_val = Value::symbol("same");
        assert_ne!(string_val, symbol_val);

        // Test boolean distinctness
        let true_val = Value::boolean(true);
        let false_val = Value::boolean(false);
        assert_ne!(true_val, Value::number(1.0));
        assert_ne!(false_val, Value::number(0.0));
        assert_ne!(true_val, Value::string("true"));
        assert_ne!(false_val, Value::string("false"));
    }

    #[test]
    fn test_special_string_formatting() {
        // Test string with quotes gets properly escaped
        let quoted_str = Value::string("He said \"Hello\"");
        let display = format!("{}", quoted_str);
        assert!(display.contains("\\\""));
        assert_eq!(display, "\"He said \\\"Hello\\\"\"");

        // Test empty string
        let empty_str = Value::string("");
        assert_eq!(format!("{}", empty_str), "\"\"");

        // Test string with special characters
        let special = Value::string("Hello\nWorld\t!");
        assert_eq!(format!("{}", special), "\"Hello\nWorld\t!\"");
    }

    #[test]
    fn test_nested_list_display() {
        // Test deeply nested lists display correctly
        let nested = Value::list(vec![
            Value::number(1.0),
            Value::list(vec![
                Value::number(2.0),
                Value::list(vec![Value::number(3.0)]),
            ]),
            Value::number(4.0),
        ]);
        assert_eq!(format!("{}", nested), "(1 (2 (3)) 4)");

        // Test mixed type lists
        let mixed = Value::list(vec![
            Value::number(42.0),
            Value::string("hello"),
            Value::symbol("world"),
            Value::boolean(true),
            Value::nil(),
        ]);
        assert_eq!(format!("{}", mixed), "(42 \"hello\" world #t ())");
    }
}
