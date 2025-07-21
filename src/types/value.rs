//! Core Value type for Scheme data representation
//!
//! This module implements the main Value enum that represents all possible
//! Scheme values, along with construction and extraction methods.

use super::{SchemeList, SchemeNumber, SchemeString, SchemeSymbol};

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
    Number(SchemeNumber),

    /// Boolean values (true/false)
    Boolean(bool),

    /// Immutable string values
    ///
    /// Using SchemeString provides proper abstraction and efficient sharing
    /// of string data across multiple Value instances.
    String(SchemeString),

    /// Symbol values (identifiers)
    ///
    /// Symbols are like strings but represent identifiers in Scheme.
    /// Using SchemeSymbol provides proper abstraction for identifier handling.
    Symbol(SchemeSymbol),

    /// List values (compound data)
    ///
    /// Represents Scheme lists using SchemeList wrapper around Vec<Value>.
    /// Lists are the fundamental compound data structure in Scheme.
    List(SchemeList),

    /// The nil/null value
    ///
    /// Represents both the empty list '() and null/undefined values.
    /// This dual nature is common in Lisp-family languages.
    Nil,
}

impl Value {
    /// Create a new number value from f64
    pub fn number(n: f64) -> Self {
        Value::Number(SchemeNumber::new(n))
    }

    /// Create a new number value from SchemeNumber
    pub fn scheme_number(n: SchemeNumber) -> Self {
        Value::Number(n)
    }

    /// Create a new boolean value
    pub fn boolean(b: bool) -> Self {
        Value::Boolean(b)
    }

    /// Create a new string value from a string slice
    pub fn string(s: &str) -> Self {
        Value::String(SchemeString::new(s))
    }

    /// Create a new string value from an owned String
    pub fn string_from_owned(s: String) -> Self {
        Value::String(SchemeString::from_string(s))
    }

    /// Create a new symbol value from a string slice
    pub fn symbol(s: &str) -> Self {
        Value::Symbol(SchemeSymbol::new(s))
    }

    /// Create a new symbol value from an owned String
    pub fn symbol_from_owned(s: String) -> Self {
        Value::Symbol(SchemeSymbol::from_string(s))
    }

    /// Create a new list value from a vector of values
    pub fn list(values: Vec<Value>) -> Self {
        Value::List(SchemeList::from_vec(values))
    }

    /// Create a new list value from a SchemeList
    pub fn scheme_list(list: SchemeList) -> Self {
        Value::List(list)
    }

    /// Create an empty list value
    pub fn empty_list() -> Self {
        Value::List(SchemeList::new())
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
    pub fn as_scheme_number(&self) -> Option<SchemeNumber> {
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
    pub fn as_list(&self) -> Option<&SchemeList> {
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
