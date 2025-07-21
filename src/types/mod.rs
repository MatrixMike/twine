//! Core data types for the Twine Scheme interpreter
//!
//! This module implements the fundamental type system with strict immutability.
//! All data structures are immutable after creation, supporting thread-safe sharing
//! and the functional programming paradigm.

use std::str::FromStr;
use std::sync::Arc;

// Re-export all public types
pub use procedures::Procedure;
pub use value::Value;

pub mod procedures;
pub mod value;

/// Immutable number type for Scheme numeric values
///
/// Wraps f64 to provide a proper abstraction for Scheme numbers with
/// parsing, formatting, and validation capabilities. This simplified
/// implementation uses f64 internally for all numeric operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchemeNumber(f64);

impl SchemeNumber {
    /// Create a new SchemeNumber from an f64 value
    pub fn new(value: f64) -> Self {
        SchemeNumber(value)
    }

    /// Get the inner f64 value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if this number is an integer (no fractional part)
    pub fn is_integer(&self) -> bool {
        self.0.is_finite() && self.0.fract() == 0.0
    }

    /// Check if this number is finite (not infinity or NaN)
    pub fn is_finite(&self) -> bool {
        self.0.is_finite()
    }

    /// Check if this number is infinite
    pub fn is_infinite(&self) -> bool {
        self.0.is_infinite()
    }

    /// Check if this number is NaN
    pub fn is_nan(&self) -> bool {
        self.0.is_nan()
    }

    /// Check if this number is positive infinity
    pub fn is_positive_infinity(&self) -> bool {
        self.0 == f64::INFINITY
    }

    /// Check if this number is negative infinity
    pub fn is_negative_infinity(&self) -> bool {
        self.0 == f64::NEG_INFINITY
    }

    // Common numeric constants
    pub const INFINITY: SchemeNumber = SchemeNumber(f64::INFINITY);
    pub const NEG_INFINITY: SchemeNumber = SchemeNumber(f64::NEG_INFINITY);
    pub const NAN: SchemeNumber = SchemeNumber(f64::NAN);
    pub const ZERO: SchemeNumber = SchemeNumber(0.0);
    pub const ONE: SchemeNumber = SchemeNumber(1.0);
}

impl FromStr for SchemeNumber {
    type Err = std::num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Trim whitespace first
        let trimmed = s.trim();

        // Handle special cases first
        match trimmed {
            "+inf.0" | "+inf" | "inf" => return Ok(SchemeNumber::INFINITY),
            "-inf.0" | "-inf" => return Ok(SchemeNumber::NEG_INFINITY),
            "+nan.0" | "nan" => return Ok(SchemeNumber::NAN),
            _ => {}
        }

        // Parse as regular float
        let value = trimmed.parse::<f64>()?;
        Ok(SchemeNumber::new(value))
    }
}

impl std::fmt::Display for SchemeNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_positive_infinity() {
            write!(f, "+inf.0")
        } else if self.is_negative_infinity() {
            write!(f, "-inf.0")
        } else if self.is_nan() {
            write!(f, "+nan.0")
        } else if self.is_integer() {
            write!(f, "{}", self.0 as i64)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<f64> for SchemeNumber {
    fn from(value: f64) -> Self {
        SchemeNumber::new(value)
    }
}

impl From<i32> for SchemeNumber {
    fn from(value: i32) -> Self {
        SchemeNumber::new(value as f64)
    }
}

impl From<i64> for SchemeNumber {
    fn from(value: i64) -> Self {
        SchemeNumber::new(value as f64)
    }
}

impl From<SchemeNumber> for f64 {
    fn from(num: SchemeNumber) -> Self {
        num.value()
    }
}

/// Immutable string type for Scheme string values
///
/// Wraps Arc<str> to provide efficient sharing of string data with
/// proper abstraction for Scheme strings. Strings are immutable after
/// creation and can be safely shared across threads.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemeString(Arc<str>);

impl SchemeString {
    /// Create a new SchemeString from a string slice
    pub fn new(s: &str) -> Self {
        SchemeString(Arc::from(s))
    }

    /// Create a new SchemeString from an owned String
    pub fn from_string(s: String) -> Self {
        SchemeString(Arc::from(s))
    }

    /// Get the string content as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the length of the string in bytes
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for SchemeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

impl From<&str> for SchemeString {
    fn from(s: &str) -> Self {
        SchemeString::new(s)
    }
}

impl From<String> for SchemeString {
    fn from(s: String) -> Self {
        SchemeString::from_string(s)
    }
}

/// Immutable symbol type for Scheme symbol values
///
/// Wraps Arc<str> to provide efficient sharing of symbol data with
/// proper abstraction for Scheme symbols. Symbols represent identifiers
/// and are immutable after creation. This implementation does not yet
/// include symbol interning for optimization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemeSymbol(Arc<str>);

impl SchemeSymbol {
    /// Create a new SchemeSymbol from a string slice
    pub fn new(s: &str) -> Self {
        SchemeSymbol(Arc::from(s))
    }

    /// Create a new SchemeSymbol from an owned String
    pub fn from_string(s: String) -> Self {
        SchemeSymbol(Arc::from(s))
    }

    /// Get the symbol name as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the length of the symbol name in bytes
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the symbol name is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for SchemeSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SchemeSymbol {
    fn from(s: &str) -> Self {
        SchemeSymbol::new(s)
    }
}

impl From<String> for SchemeSymbol {
    fn from(s: String) -> Self {
        SchemeSymbol::from_string(s)
    }
}

/// Immutable list type for Scheme list values
///
/// Wraps Vec<Value> to provide proper abstraction for Scheme lists.
/// This is a simplified implementation that uses Vec for storage.
/// Future versions will add structural sharing with Arc for efficiency.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemeList(Vec<Value>);

impl SchemeList {
    /// Create a new empty SchemeList
    pub fn new() -> Self {
        SchemeList(Vec::new())
    }

    /// Create a new SchemeList from a vector of values
    pub fn from_vec(values: Vec<Value>) -> Self {
        SchemeList(values)
    }

    /// Create a new SchemeList from an iterator of values
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        SchemeList(iter.into_iter().collect())
    }

    /// Get the length of the list
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a reference to the value at the given index
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Get an iterator over the values in the list
    pub fn iter(&self) -> std::slice::Iter<Value> {
        self.0.iter()
    }

    /// Convert to a vector (consumes the list)
    pub fn into_vec(self) -> Vec<Value> {
        self.0
    }

    /// Get a slice of the underlying values
    pub fn as_slice(&self) -> &[Value] {
        &self.0
    }
}

impl Default for SchemeList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SchemeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, value) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", value)?;
        }
        write!(f, ")")
    }
}

impl From<Vec<Value>> for SchemeList {
    fn from(values: Vec<Value>) -> Self {
        SchemeList::from_vec(values)
    }
}

impl FromIterator<Value> for SchemeList {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        SchemeList::from_iter(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_number_parsing() {
        // Test basic integer parsing
        assert_eq!(
            "42".parse::<SchemeNumber>().unwrap(),
            SchemeNumber::new(42.0)
        );
        assert_eq!(
            "-17".parse::<SchemeNumber>().unwrap(),
            SchemeNumber::new(-17.0)
        );

        // Test floating point parsing
        assert_eq!(
            "3.14".parse::<SchemeNumber>().unwrap(),
            SchemeNumber::new(3.14)
        );
        assert_eq!(
            "-2.5".parse::<SchemeNumber>().unwrap(),
            SchemeNumber::new(-2.5)
        );

        // Test scientific notation
        assert_eq!(
            "1e3".parse::<SchemeNumber>().unwrap(),
            SchemeNumber::new(1000.0)
        );
        assert_eq!(
            "2.5e-2".parse::<SchemeNumber>().unwrap(),
            SchemeNumber::new(0.025)
        );

        // Test special values
        assert!(
            "+inf.0"
                .parse::<SchemeNumber>()
                .unwrap()
                .is_positive_infinity()
        );
        assert!(
            "-inf.0"
                .parse::<SchemeNumber>()
                .unwrap()
                .is_negative_infinity()
        );
        assert!(
            "+inf"
                .parse::<SchemeNumber>()
                .unwrap()
                .is_positive_infinity()
        );
        assert!(
            "-inf"
                .parse::<SchemeNumber>()
                .unwrap()
                .is_negative_infinity()
        );
        assert!("+nan.0".parse::<SchemeNumber>().unwrap().is_nan());
        assert!("nan".parse::<SchemeNumber>().unwrap().is_nan());

        // Test whitespace handling
        assert_eq!(
            "  42  ".parse::<SchemeNumber>().unwrap(),
            SchemeNumber::new(42.0)
        );

        // Test invalid formats
        assert!("abc".parse::<SchemeNumber>().is_err());
        assert!("12.34.56".parse::<SchemeNumber>().is_err());
        assert!("".parse::<SchemeNumber>().is_err());
    }

    #[test]
    fn test_number_formatting() {
        // Test integer formatting
        assert_eq!(SchemeNumber::new(42.0).to_string(), "42");
        assert_eq!(SchemeNumber::new(-17.0).to_string(), "-17");
        assert_eq!(SchemeNumber::new(0.0).to_string(), "0");

        // Test floating point formatting
        assert_eq!(SchemeNumber::new(3.14).to_string(), "3.14");
        assert_eq!(SchemeNumber::new(-2.5).to_string(), "-2.5");

        // Test special value formatting
        assert_eq!(SchemeNumber::INFINITY.to_string(), "+inf.0");
        assert_eq!(SchemeNumber::NEG_INFINITY.to_string(), "-inf.0");
        assert_eq!(SchemeNumber::NAN.to_string(), "+nan.0");

        // Test constants
        assert_eq!(SchemeNumber::ZERO.to_string(), "0");
        assert_eq!(SchemeNumber::ONE.to_string(), "1");
    }

    #[test]
    fn test_number_equality() {
        // Test basic equality
        assert_eq!(SchemeNumber::new(42.0), SchemeNumber::new(42.0));
        assert_ne!(SchemeNumber::new(42.0), SchemeNumber::new(43.0));

        // Test floating point equality
        assert_eq!(SchemeNumber::new(3.14), SchemeNumber::new(3.14));
        assert_ne!(SchemeNumber::new(3.14), SchemeNumber::new(3.15));

        // Test special values
        assert_eq!(SchemeNumber::INFINITY, SchemeNumber::INFINITY);
        assert_eq!(SchemeNumber::NEG_INFINITY, SchemeNumber::NEG_INFINITY);
        assert_ne!(SchemeNumber::INFINITY, SchemeNumber::NEG_INFINITY);

        // Note: NaN != NaN in IEEE 754, so we test this behavior
        assert_ne!(SchemeNumber::NAN, SchemeNumber::NAN);

        // Test constants
        assert_eq!(SchemeNumber::ZERO, SchemeNumber::new(0.0));
        assert_eq!(SchemeNumber::ONE, SchemeNumber::new(1.0));
    }

    #[test]
    fn test_number_edge_cases() {
        // Test infinity
        let pos_inf = SchemeNumber::INFINITY;
        assert!(pos_inf.is_infinite());
        assert!(pos_inf.is_positive_infinity());
        assert!(!pos_inf.is_negative_infinity());
        assert!(!pos_inf.is_finite());
        assert!(!pos_inf.is_nan());

        let neg_inf = SchemeNumber::NEG_INFINITY;
        assert!(neg_inf.is_infinite());
        assert!(neg_inf.is_negative_infinity());
        assert!(!neg_inf.is_positive_infinity());
        assert!(!neg_inf.is_finite());
        assert!(!neg_inf.is_nan());

        // Test NaN
        let nan = SchemeNumber::NAN;
        assert!(nan.is_nan());
        assert!(!nan.is_infinite());
        assert!(!nan.is_finite());
        assert!(!nan.is_positive_infinity());
        assert!(!nan.is_negative_infinity());

        // Test finite numbers
        let normal = SchemeNumber::new(42.5);
        assert!(normal.is_finite());
        assert!(!normal.is_infinite());
        assert!(!normal.is_nan());

        // Test integer detection
        assert!(SchemeNumber::new(42.0).is_integer());
        assert!(!SchemeNumber::new(42.5).is_integer());
        assert!(SchemeNumber::new(-17.0).is_integer());
        assert!(!SchemeNumber::new(-17.3).is_integer());

        // Test zero handling
        assert!(SchemeNumber::ZERO.is_integer());
        assert!(SchemeNumber::ZERO.is_finite());
        assert_eq!(SchemeNumber::ZERO.value(), 0.0);
    }

    #[test]
    fn test_scheme_number_conversions() {
        // Test conversions from various types
        let from_f64: SchemeNumber = 42.5.into();
        assert_eq!(from_f64.value(), 42.5);

        let from_i32: SchemeNumber = 42i32.into();
        assert_eq!(from_i32.value(), 42.0);

        let from_i64: SchemeNumber = 42i64.into();
        assert_eq!(from_i64.value(), 42.0);

        // Test conversion to f64
        let num = SchemeNumber::new(3.14);
        let as_f64: f64 = num.into();
        assert_eq!(as_f64, 3.14);

        // Test Value creation with SchemeNumber
        let value = Value::scheme_number(SchemeNumber::new(42.0));
        assert_eq!(value.as_number(), Some(42.0));
        assert_eq!(value.as_scheme_number(), Some(SchemeNumber::new(42.0)));
    }

    #[test]
    fn test_string_creation() {
        // Test SchemeString creation from &str
        let s1 = SchemeString::new("hello");
        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s1.len(), 5);
        assert!(!s1.is_empty());

        // Test SchemeString creation from String
        let s2 = SchemeString::from_string("world".to_string());
        assert_eq!(s2.as_str(), "world");
        assert_eq!(s2.len(), 5);

        // Test empty string
        let empty = SchemeString::new("");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        // Test Value::String creation
        let val1 = Value::string("test");
        assert!(val1.is_string());
        assert_eq!(val1.as_string(), Some("test"));
        assert_eq!(val1.type_name(), "string");

        let val2 = Value::string_from_owned("owned".to_string());
        assert!(val2.is_string());
        assert_eq!(val2.as_string(), Some("owned"));
    }

    #[test]
    fn test_symbol_creation() {
        // Test SchemeSymbol creation from &str
        let s1 = SchemeSymbol::new("foo");
        assert_eq!(s1.as_str(), "foo");
        assert_eq!(s1.len(), 3);
        assert!(!s1.is_empty());

        // Test SchemeSymbol creation from String
        let s2 = SchemeSymbol::from_string("bar".to_string());
        assert_eq!(s2.as_str(), "bar");
        assert_eq!(s2.len(), 3);

        // Test empty symbol (though not typical in practice)
        let empty = SchemeSymbol::new("");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        // Test Value::Symbol creation
        let val1 = Value::symbol("identifier");
        assert!(val1.is_symbol());
        assert_eq!(val1.as_symbol(), Some("identifier"));
        assert_eq!(val1.type_name(), "symbol");

        let val2 = Value::symbol_from_owned("var-name".to_string());
        assert!(val2.is_symbol());
        assert_eq!(val2.as_symbol(), Some("var-name"));
    }

    #[test]
    fn test_string_symbol_equality() {
        // Test SchemeString equality
        let str1 = SchemeString::new("hello");
        let str2 = SchemeString::new("hello");
        let str3 = SchemeString::new("world");
        assert_eq!(str1, str2);
        assert_ne!(str1, str3);

        // Test SchemeSymbol equality
        let sym1 = SchemeSymbol::new("foo");
        let sym2 = SchemeSymbol::new("foo");
        let sym3 = SchemeSymbol::new("bar");
        assert_eq!(sym1, sym2);
        assert_ne!(sym1, sym3);

        // Test Value equality for strings
        let val_str1 = Value::string("test");
        let val_str2 = Value::string("test");
        let val_str3 = Value::string("other");
        assert_eq!(val_str1, val_str2);
        assert_ne!(val_str1, val_str3);

        // Test Value equality for symbols
        let val_sym1 = Value::symbol("var");
        let val_sym2 = Value::symbol("var");
        let val_sym3 = Value::symbol("func");
        assert_eq!(val_sym1, val_sym2);
        assert_ne!(val_sym1, val_sym3);

        // Test that strings and symbols with same content are not equal
        let string_val = Value::string("same");
        let symbol_val = Value::symbol("same");
        assert_ne!(string_val, symbol_val);
    }

    #[test]
    fn test_string_symbol_hashing() {
        use std::collections::HashMap;

        // Test SchemeString hashing
        let mut string_map = HashMap::new();
        let str1 = SchemeString::new("key1");
        let str2 = SchemeString::new("key1");
        let str3 = SchemeString::new("key2");

        string_map.insert(str1.clone(), "value1");
        assert_eq!(string_map.get(&str2), Some(&"value1")); // Same content should hash equally
        assert_eq!(string_map.get(&str3), None); // Different content should not match

        // Test SchemeSymbol hashing
        let mut symbol_map = HashMap::new();
        let sym1 = SchemeSymbol::new("symbol1");
        let sym2 = SchemeSymbol::new("symbol1");
        let sym3 = SchemeSymbol::new("symbol2");

        symbol_map.insert(sym1.clone(), "value1");
        assert_eq!(symbol_map.get(&sym2), Some(&"value1")); // Same content should hash equally
        assert_eq!(symbol_map.get(&sym3), None); // Different content should not match

        // Test that equal values have equal hashes
        let str_a = SchemeString::new("test");
        let str_b = SchemeString::new("test");
        let sym_a = SchemeSymbol::new("test");
        let sym_b = SchemeSymbol::new("test");

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        str_a.hash(&mut hasher1);
        str_b.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());

        let mut hasher3 = DefaultHasher::new();
        let mut hasher4 = DefaultHasher::new();
        sym_a.hash(&mut hasher3);
        sym_b.hash(&mut hasher4);
        assert_eq!(hasher3.finish(), hasher4.finish());
    }

    #[test]
    fn test_list_creation() {
        // Test empty SchemeList creation
        let empty = SchemeList::new();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.get(0), None);

        // Test SchemeList creation from vector
        let values = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let list = SchemeList::from_vec(values.clone());
        assert!(!list.is_empty());
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(&Value::number(1.0)));
        assert_eq!(list.get(1), Some(&Value::number(2.0)));
        assert_eq!(list.get(2), Some(&Value::number(3.0)));
        assert_eq!(list.get(3), None);

        // Test iteration
        let collected: Vec<&Value> = list.iter().collect();
        assert_eq!(collected.len(), 3);

        // Test Value::List creation
        let val_empty = Value::empty_list();
        assert!(val_empty.is_list());
        assert_eq!(val_empty.type_name(), "list");

        let val_list = Value::list(vec![Value::string("hello"), Value::symbol("world")]);
        assert!(val_list.is_list());
        if let Some(scheme_list) = val_list.as_list() {
            assert_eq!(scheme_list.len(), 2);
            assert_eq!(scheme_list.get(0), Some(&Value::string("hello")));
            assert_eq!(scheme_list.get(1), Some(&Value::symbol("world")));
        } else {
            panic!("Expected list value");
        }
    }

    #[test]
    fn test_list_display() {
        // Test empty list display
        let empty = SchemeList::new();
        assert_eq!(format!("{}", empty), "()");

        let empty_val = Value::empty_list();
        assert_eq!(format!("{}", empty_val), "()");

        // Test single element list
        let single = SchemeList::from_vec(vec![Value::number(42.0)]);
        assert_eq!(format!("{}", single), "(42)");

        // Test multiple elements
        let multi = SchemeList::from_vec(vec![
            Value::number(1.0),
            Value::string("hello"),
            Value::symbol("test"),
            Value::boolean(true),
        ]);
        assert_eq!(format!("{}", multi), "(1 \"hello\" test #t)");

        // Test nested lists
        let nested = Value::list(vec![
            Value::number(1.0),
            Value::list(vec![Value::number(2.0), Value::number(3.0)]),
            Value::number(4.0),
        ]);
        assert_eq!(format!("{}", nested), "(1 (2 3) 4)");
    }

    #[test]
    fn test_list_equality() {
        // Test empty list equality
        let empty1 = SchemeList::new();
        let empty2 = SchemeList::new();
        assert_eq!(empty1, empty2);

        // Test same content equality
        let list1 = SchemeList::from_vec(vec![Value::number(1.0), Value::string("test")]);
        let list2 = SchemeList::from_vec(vec![Value::number(1.0), Value::string("test")]);
        assert_eq!(list1, list2);

        // Test different content inequality
        let list3 = SchemeList::from_vec(vec![Value::number(2.0), Value::string("test")]);
        assert_ne!(list1, list3);

        // Test different length inequality
        let list4 = SchemeList::from_vec(vec![Value::number(1.0)]);
        assert_ne!(list1, list4);

        // Test Value::List equality
        let val1 = Value::list(vec![Value::number(1.0), Value::boolean(true)]);
        let val2 = Value::list(vec![Value::number(1.0), Value::boolean(true)]);
        let val3 = Value::list(vec![Value::number(1.0), Value::boolean(false)]);

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);

        // Test that lists and nil are different
        let list_val = Value::empty_list();
        let nil_val = Value::nil();
        assert_ne!(list_val, nil_val);
    }

    #[test]
    fn test_empty_list() {
        // Test SchemeList empty behavior
        let empty = SchemeList::new();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.as_slice(), &[]);
        assert_eq!(empty.iter().count(), 0);

        // Test Default trait
        let default_list: SchemeList = Default::default();
        assert!(default_list.is_empty());
        assert_eq!(empty, default_list);

        // Test Value::empty_list()
        let empty_val = Value::empty_list();
        assert!(empty_val.is_list());
        if let Some(list) = empty_val.as_list() {
            assert!(list.is_empty());
        } else {
            panic!("Expected list value");
        }

        // Test conversion from empty Vec
        let from_vec = SchemeList::from_vec(Vec::new());
        assert!(from_vec.is_empty());
        assert_eq!(from_vec, empty);

        // Test FromIterator with empty iterator
        let from_iter: SchemeList = std::iter::empty().collect();
        assert!(from_iter.is_empty());
        assert_eq!(from_iter, empty);

        // Test into_vec on empty list
        let vec = empty.into_vec();
        assert!(vec.is_empty());
    }

    #[test]
    fn test_value_creation() {
        // Test number creation
        let num = Value::number(42.0);
        assert!(matches!(num, Value::Number(_)));
        assert_eq!(num.as_number(), Some(42.0));

        // Test boolean creation
        let bool_true = Value::boolean(true);
        let bool_false = Value::boolean(false);
        assert!(matches!(bool_true, Value::Boolean(true)));
        assert!(matches!(bool_false, Value::Boolean(false)));

        // Test string creation
        let string = Value::string("hello");
        assert!(matches!(string, Value::String(_)));
        assert_eq!(string.as_string(), Some("hello"));

        // Test symbol creation
        let symbol = Value::symbol("foo");
        assert!(matches!(symbol, Value::Symbol(_)));
        assert_eq!(symbol.as_symbol(), Some("foo"));

        // Test list creation
        let empty_list = Value::empty_list();
        assert!(matches!(empty_list, Value::List(_)));
        assert_eq!(empty_list.as_list().unwrap().len(), 0);

        let list = Value::list(vec![Value::number(1.0), Value::string("test")]);
        assert!(matches!(list, Value::List(_)));
        assert_eq!(list.as_list().unwrap().len(), 2);

        // Test nil creation
        let nil = Value::nil();
        assert!(matches!(nil, Value::Nil));
    }

    #[test]
    fn test_value_debug_output() {
        let values = vec![
            Value::number(3.14),
            Value::boolean(true),
            Value::string("test"),
            Value::symbol("var"),
            Value::list(vec![Value::number(1.0), Value::number(2.0)]),
            Value::nil(),
        ];

        for value in values {
            let debug_str = format!("{:?}", value);
            assert!(!debug_str.is_empty());
            // Each variant should appear in its debug representation
            match value {
                Value::Number(_) => assert!(debug_str.contains("Number")),
                Value::Boolean(_) => assert!(debug_str.contains("Boolean")),
                Value::String(_) => assert!(debug_str.contains("String")),
                Value::Symbol(_) => assert!(debug_str.contains("Symbol")),
                Value::List(_) => assert!(debug_str.contains("List")),
                Value::Nil => assert!(debug_str.contains("Nil")),
            }
        }
    }

    #[test]
    fn test_value_equality() {
        // Numbers
        assert_eq!(Value::number(42.0), Value::number(42.0));
        assert_ne!(Value::number(42.0), Value::number(43.0));

        // Booleans
        assert_eq!(Value::boolean(true), Value::boolean(true));
        assert_eq!(Value::boolean(false), Value::boolean(false));
        assert_ne!(Value::boolean(true), Value::boolean(false));

        // Strings
        assert_eq!(Value::string("hello"), Value::string("hello"));
        assert_ne!(Value::string("hello"), Value::string("world"));

        // Symbols
        assert_eq!(Value::symbol("foo"), Value::symbol("foo"));
        assert_ne!(Value::symbol("foo"), Value::symbol("bar"));

        // Nil
        assert_eq!(Value::nil(), Value::nil());

        // Cross-type inequality
        assert_ne!(Value::number(0.0), Value::boolean(false));
        assert_ne!(Value::string("42"), Value::number(42.0));
        assert_ne!(Value::symbol("nil"), Value::nil());
    }

    #[test]
    fn test_value_cloning() {
        let original = Value::string("shared data");
        let cloned = original.clone();

        assert_eq!(original, cloned);

        // Verify that string data is actually shared (same Arc)
        if let (Value::String(s1), Value::String(s2)) = (&original, &cloned) {
            assert!(Arc::ptr_eq(&s1.0, &s2.0));
        } else {
            panic!("Expected string values");
        }
    }

    #[test]
    fn test_type_checking_methods() {
        let number = Value::number(42.0);
        assert!(number.is_number());
        assert!(!number.is_boolean());
        assert!(!number.is_string());
        assert!(!number.is_symbol());
        assert!(!number.is_list());
        assert!(!number.is_nil());

        let boolean = Value::boolean(true);
        assert!(!boolean.is_number());
        assert!(boolean.is_boolean());
        assert!(!boolean.is_string());
        assert!(!boolean.is_symbol());
        assert!(!boolean.is_list());
        assert!(!boolean.is_nil());

        let string = Value::string("test");
        assert!(!string.is_number());
        assert!(!string.is_boolean());
        assert!(string.is_string());
        assert!(!string.is_symbol());
        assert!(!string.is_list());
        assert!(!string.is_nil());

        let symbol = Value::symbol("var");
        assert!(!symbol.is_number());
        assert!(!symbol.is_boolean());
        assert!(!symbol.is_string());
        assert!(symbol.is_symbol());
        assert!(!symbol.is_list());
        assert!(!symbol.is_nil());

        let list = Value::empty_list();
        assert!(!list.is_number());
        assert!(!list.is_boolean());
        assert!(!list.is_string());
        assert!(!list.is_symbol());
        assert!(list.is_list());
        assert!(!list.is_nil());

        let nil = Value::nil();
        assert!(!nil.is_number());
        assert!(!nil.is_boolean());
        assert!(!nil.is_string());
        assert!(!nil.is_symbol());
        assert!(!nil.is_list());
        assert!(nil.is_nil());
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
    fn test_type_name_method() {
        assert_eq!(Value::number(42.0).type_name(), "number");
        assert_eq!(Value::boolean(true).type_name(), "boolean");
        assert_eq!(Value::string("test").type_name(), "string");
        assert_eq!(Value::symbol("var").type_name(), "symbol");
        assert_eq!(Value::empty_list().type_name(), "list");
        assert_eq!(Value::nil().type_name(), "nil");
    }

    #[test]
    fn test_display_formatting() {
        // Numbers
        assert_eq!(Value::number(42.0).to_string(), "42");
        assert_eq!(Value::number(3.14).to_string(), "3.14");
        assert_eq!(Value::number(-1.0).to_string(), "-1");

        // Special number values
        assert_eq!(
            Value::scheme_number(SchemeNumber::INFINITY).to_string(),
            "+inf.0"
        );
        assert_eq!(
            Value::scheme_number(SchemeNumber::NEG_INFINITY).to_string(),
            "-inf.0"
        );
        assert_eq!(
            Value::scheme_number(SchemeNumber::NAN).to_string(),
            "+nan.0"
        );

        // Booleans
        assert_eq!(Value::boolean(true).to_string(), "#t");
        assert_eq!(Value::boolean(false).to_string(), "#f");

        // Strings (should be quoted and escaped)
        assert_eq!(Value::string("hello").to_string(), "\"hello\"");
        assert_eq!(
            Value::string("say \"hi\"").to_string(),
            "\"say \\\"hi\\\"\""
        );

        // Symbols (no quotes)
        assert_eq!(Value::symbol("variable").to_string(), "variable");
        assert_eq!(Value::symbol("+").to_string(), "+");

        // Lists
        assert_eq!(Value::empty_list().to_string(), "()");

        // Nil
        assert_eq!(Value::nil().to_string(), "()");
    }

    #[test]
    fn test_string_creation_variants() {
        // Test creation from &str
        let from_str = Value::string("hello");

        // Test creation from owned String
        let owned_string = String::from("hello");
        let from_owned = Value::string_from_owned(owned_string);

        assert_eq!(from_str, from_owned);
    }

    #[test]
    fn test_symbol_creation_variants() {
        // Test creation from &str
        let from_str = Value::symbol("foo");

        // Test creation from owned String
        let owned_string = String::from("foo");
        let from_owned = Value::symbol_from_owned(owned_string);

        assert_eq!(from_str, from_owned);
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that string data is shared efficiently
        let s1 = Value::string("shared");
        let s2 = Value::string("shared");

        // While they are equal...
        assert_eq!(s1, s2);

        // ...they don't necessarily share the same Arc instance
        // (this would require string interning for true efficiency)
        // This test documents current behavior
    }

    // Comprehensive value system tests for T1.2.5

    #[test]
    fn test_value_type_conversions() {
        // Test various type conversions and edge cases
        let int_val = Value::number(42.0);
        let float_val = Value::number(3.14159);
        let zero_val = Value::number(0.0);
        let neg_val = Value::number(-123.45);

        // Number conversions
        assert_eq!(int_val.as_number(), Some(42.0));
        assert_eq!(float_val.as_number(), Some(3.14159));
        assert_eq!(zero_val.as_number(), Some(0.0));
        assert_eq!(neg_val.as_number(), Some(-123.45));

        // Type checking consistency
        assert!(int_val.is_number());
        assert!(!int_val.is_string());
        assert!(!int_val.is_symbol());
        assert!(!int_val.is_list());
        assert!(!int_val.is_boolean());
        assert!(!int_val.is_nil());
    }

    #[test]
    fn test_value_special_numbers() {
        // Test special numeric values
        let inf_val = Value::number(f64::INFINITY);
        let neg_inf_val = Value::number(f64::NEG_INFINITY);
        let nan_val = Value::number(f64::NAN);

        assert!(inf_val.is_number());
        assert!(neg_inf_val.is_number());
        assert!(nan_val.is_number());

        if let Some(n) = inf_val.as_number() {
            assert!(n.is_infinite() && n.is_sign_positive());
        }
        if let Some(n) = neg_inf_val.as_number() {
            assert!(n.is_infinite() && n.is_sign_negative());
        }
        if let Some(n) = nan_val.as_number() {
            assert!(n.is_nan());
        }
    }

    #[test]
    fn test_string_edge_cases() {
        // Test empty string
        let empty_str = Value::string("");
        assert!(empty_str.is_string());
        assert_eq!(empty_str.as_string(), Some(""));

        // Test strings with special characters
        let special = Value::string("Hello\nWorld\t!");
        assert_eq!(special.as_string(), Some("Hello\nWorld\t!"));

        // Test Unicode strings
        let unicode = Value::string("🦀 Rust 中文");
        assert_eq!(unicode.as_string(), Some("🦀 Rust 中文"));

        // Test very long string
        let long_str = "x".repeat(10000);
        let long_val = Value::string(&long_str);
        assert_eq!(long_val.as_string(), Some(long_str.as_str()));
    }

    #[test]
    fn test_symbol_edge_cases() {
        // Test empty symbol (unusual but valid)
        let empty_sym = Value::symbol("");
        assert!(empty_sym.is_symbol());
        assert_eq!(empty_sym.as_symbol(), Some(""));

        // Test symbols with special characters
        let special_sym = Value::symbol("+-*/");
        assert_eq!(special_sym.as_symbol(), Some("+-*/"));

        // Test very long symbol
        let long_sym = "identifier".repeat(1000);
        let long_val = Value::symbol(&long_sym);
        assert_eq!(long_val.as_symbol(), Some(long_sym.as_str()));

        // Test symbols vs strings distinction
        let str_val = Value::string("test");
        let sym_val = Value::symbol("test");
        assert_ne!(str_val, sym_val);
        assert_ne!(str_val.type_name(), sym_val.type_name());
    }

    #[test]
    fn test_list_edge_cases() {
        // Test deeply nested lists
        let mut nested = Value::empty_list();
        for i in 0..100 {
            nested = Value::list(vec![Value::number(i as f64), nested]);
        }
        assert!(nested.is_list());

        // Test large lists
        let large_vec: Vec<Value> = (0..10000).map(|i| Value::number(i as f64)).collect();
        let large_list = Value::list(large_vec);
        if let Some(list) = large_list.as_list() {
            assert_eq!(list.len(), 10000);
        }

        // Test mixed type lists
        let mixed = Value::list(vec![
            Value::number(42.0),
            Value::string("hello"),
            Value::symbol("world"),
            Value::boolean(true),
            Value::empty_list(),
            Value::nil(),
        ]);
        if let Some(list) = mixed.as_list() {
            assert_eq!(list.len(), 6);
            assert!(list.get(0).unwrap().is_number());
            assert!(list.get(1).unwrap().is_string());
            assert!(list.get(2).unwrap().is_symbol());
            assert!(list.get(3).unwrap().is_boolean());
            assert!(list.get(4).unwrap().is_list());
            assert!(list.get(5).unwrap().is_nil());
        }
    }

    #[test]
    fn test_boolean_edge_cases() {
        let true_val = Value::boolean(true);
        let false_val = Value::boolean(false);

        // Test boolean specific behavior
        assert_eq!(true_val.as_boolean(), Some(true));
        assert_eq!(false_val.as_boolean(), Some(false));

        // Test that booleans are distinct from other types
        assert_ne!(true_val, Value::number(1.0));
        assert_ne!(false_val, Value::number(0.0));
        assert_ne!(true_val, Value::string("true"));
        assert_ne!(false_val, Value::string("false"));
    }

    #[test]
    fn test_nil_edge_cases() {
        let nil = Value::nil();
        let empty_list = Value::empty_list();

        // Nil should be distinct from empty list
        assert_ne!(nil, empty_list);
        assert!(nil.is_nil());
        assert!(!empty_list.is_nil());
        assert!(!nil.is_list());
        assert!(empty_list.is_list());

        // Nil should have correct type name
        assert_eq!(nil.type_name(), "nil");
        assert_eq!(empty_list.type_name(), "list");
    }

    #[test]
    fn test_value_display_edge_cases() {
        // Test display of special values
        let inf_val = Value::number(f64::INFINITY);
        let neg_inf_val = Value::number(f64::NEG_INFINITY);
        let nan_val = Value::number(f64::NAN);

        let inf_display = format!("{}", inf_val);
        let neg_inf_display = format!("{}", neg_inf_val);
        let nan_display = format!("{}", nan_val);

        // Check for infinity representations (case insensitive)
        assert!(inf_display.to_lowercase().contains("inf"));
        assert!(neg_inf_display.to_lowercase().contains("inf"));
        // NaN might be displayed as "NaN" or "nan" depending on platform
        assert!(nan_display.to_lowercase().contains("nan"));

        // Test string with quotes
        let quoted_str = Value::string("He said \"Hello\"");
        let display = format!("{}", quoted_str);
        assert!(display.contains("\\\""));

        // Test empty containers
        let empty_list = Value::empty_list();
        assert_eq!(format!("{}", empty_list), "()");
    }

    #[test]
    fn test_value_equality_comprehensive() {
        // Test reflexivity (a == a)
        let values = vec![
            Value::number(42.0),
            Value::boolean(true),
            Value::string("test"),
            Value::symbol("test"),
            Value::list(vec![Value::number(1.0)]),
            Value::nil(),
        ];

        for value in &values {
            assert_eq!(value, value);
        }

        // Test symmetry (a == b implies b == a)
        let a = Value::number(42.0);
        let b = Value::number(42.0);
        assert_eq!(a, b);
        assert_eq!(b, a);

        // Test transitivity (a == b && b == c implies a == c)
        let c = Value::number(42.0);
        assert_eq!(a, c);

        // Test inequality cases
        assert_ne!(Value::number(1.0), Value::number(2.0));
        assert_ne!(Value::string("a"), Value::string("b"));
        assert_ne!(Value::symbol("a"), Value::symbol("b"));
        assert_ne!(Value::boolean(true), Value::boolean(false));
    }

    #[test]
    fn test_value_cloning_comprehensive() {
        // Test that cloning preserves all properties
        let original_values = vec![
            Value::number(3.14159),
            Value::boolean(false),
            Value::string("clone test"),
            Value::symbol("cloned-sym"),
            Value::list(vec![Value::number(1.0), Value::string("nested")]),
            Value::nil(),
        ];

        for original in original_values {
            let cloned = original.clone();

            // Cloned value should be equal
            assert_eq!(original, cloned);

            // Should have same type
            assert_eq!(original.type_name(), cloned.type_name());

            // Should have same extracted values
            assert_eq!(original.as_number(), cloned.as_number());
            assert_eq!(original.as_boolean(), cloned.as_boolean());
            assert_eq!(original.as_string(), cloned.as_string());
            assert_eq!(original.as_symbol(), cloned.as_symbol());
        }
    }

    #[test]
    fn test_scheme_types_memory_sharing() {
        // Test that Arc sharing works for strings
        let str1 = SchemeString::new("shared");
        let str2 = str1.clone();
        assert!(Arc::ptr_eq(&str1.0, &str2.0));

        // Test that Arc sharing works for symbols
        let sym1 = SchemeSymbol::new("shared");
        let sym2 = sym1.clone();
        assert!(Arc::ptr_eq(&sym1.0, &sym2.0));

        // Test Value-level sharing
        let val1 = Value::string("shared");
        let val2 = val1.clone();
        if let (Value::String(s1), Value::String(s2)) = (&val1, &val2) {
            assert!(Arc::ptr_eq(&s1.0, &s2.0));
        }
    }

    #[test]
    fn test_scheme_list_operations() {
        // Test various SchemeList operations
        let empty = SchemeList::new();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
        assert_eq!(empty.get(0), None);

        // Test from_iter
        let from_iter: SchemeList = vec![Value::number(1.0), Value::number(2.0)]
            .into_iter()
            .collect();
        assert_eq!(from_iter.len(), 2);

        // Test as_slice
        let list = SchemeList::from_vec(vec![Value::number(42.0)]);
        let slice = list.as_slice();
        assert_eq!(slice.len(), 1);
        assert_eq!(slice[0], Value::number(42.0));

        // Test into_vec
        let vec = list.into_vec();
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], Value::number(42.0));
    }

    #[test]
    fn test_type_extraction_errors() {
        // Test that wrong type extractions return None
        let num = Value::number(42.0);
        assert!(num.as_boolean().is_none());
        assert!(num.as_string().is_none());
        assert!(num.as_symbol().is_none());
        assert!(num.as_list().is_none());

        let str_val = Value::string("test");
        assert!(str_val.as_number().is_none());
        assert!(str_val.as_boolean().is_none());
        assert!(str_val.as_symbol().is_none());
        assert!(str_val.as_list().is_none());

        let list_val = Value::empty_list();
        assert!(list_val.as_number().is_none());
        assert!(list_val.as_boolean().is_none());
        assert!(list_val.as_string().is_none());
        assert!(list_val.as_symbol().is_none());
    }

    #[test]
    fn test_value_size_and_efficiency() {
        // Test that our Value enum isn't too large
        use std::mem;
        let size = mem::size_of::<Value>();

        // Should be reasonable size (this is informational)
        // Arc<str> is typically 16 bytes on 64-bit systems
        // Vec<Value> is 24 bytes, so our enum should be around 32 bytes or less
        assert!(
            size <= 64,
            "Value enum size is {} bytes, might be too large",
            size
        );

        // Test that different variants have consistent behavior
        let values = vec![
            Value::number(1.0),
            Value::boolean(true),
            Value::string("test"),
            Value::symbol("test"),
            Value::empty_list(),
            Value::nil(),
        ];

        for value in values {
            // All values should be cloneable
            let _cloned = value.clone();

            // All values should have debug representation
            let _debug = format!("{:?}", value);

            // All values should have display representation
            let _display = format!("{}", value);

            // All values should have type names
            let _type_name = value.type_name();
        }
    }

    #[test]
    fn test_comprehensive_value_roundtrip() {
        // Test that values can be created, extracted, and recreated consistently
        let test_cases = vec![
            (Value::number(42.5), "number"),
            (Value::boolean(true), "boolean"),
            (Value::boolean(false), "boolean"),
            (Value::string("hello world"), "string"),
            (Value::symbol("test-symbol"), "symbol"),
            (
                Value::list(vec![Value::number(1.0), Value::number(2.0)]),
                "list",
            ),
            (Value::empty_list(), "list"),
            (Value::nil(), "nil"),
        ];

        for (original, expected_type) in test_cases {
            // Check type name
            assert_eq!(original.type_name(), expected_type);

            // Clone and verify equality
            let cloned = original.clone();
            assert_eq!(original, cloned);

            // Test specific extractions based on type
            match expected_type {
                "number" => {
                    assert!(original.is_number());
                    assert!(original.as_number().is_some());
                }
                "boolean" => {
                    assert!(original.is_boolean());
                    assert!(original.as_boolean().is_some());
                }
                "string" => {
                    assert!(original.is_string());
                    assert!(original.as_string().is_some());
                }
                "symbol" => {
                    assert!(original.is_symbol());
                    assert!(original.as_symbol().is_some());
                }
                "list" => {
                    assert!(original.is_list());
                    assert!(original.as_list().is_some());
                }
                "nil" => {
                    assert!(original.is_nil());
                }
                _ => panic!("Unexpected type: {}", expected_type),
            }
        }
    }
}
