//! Symbol type implementation for Scheme
//!
//! This module implements the Symbol type that represents identifiers and
//! symbol values in Scheme. Symbols are like strings but represent identifiers.

use std::rc::Rc;

/// Symbol type for Scheme identifiers
///
/// Wraps a reference-counted string to enable efficient sharing while
/// maintaining immutability guarantees. Symbols are used for identifiers,
/// variable names, and symbolic data in Scheme.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(Rc<std::string::String>);

impl Symbol {
    /// Create a new Symbol from a string slice
    pub fn new(s: &str) -> Self {
        Symbol(Rc::new(s.to_string()))
    }

    /// Create a new Symbol from an owned String
    pub fn from_string(s: std::string::String) -> Self {
        Symbol(Rc::new(s))
    }

    /// Get a string slice view of the symbol name
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

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Symbol::new(s)
    }
}

impl From<std::string::String> for Symbol {
    fn from(s: std::string::String) -> Self {
        Symbol::from_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        // Test creation from &str
        let s1 = Symbol::new("variable");
        assert_eq!(s1.as_str(), "variable");
        assert_eq!(s1.len(), 8);
        assert!(!s1.is_empty());

        // Test creation from owned String
        let owned = std::string::String::from("function-name");
        let s2 = Symbol::from_string(owned);
        assert_eq!(s2.as_str(), "function-name");
        assert_eq!(s2.len(), 13);

        // Test empty symbol
        let empty = Symbol::new("");
        assert_eq!(empty.as_str(), "");
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());

        // Test symbol with special characters commonly used in Scheme
        let special = Symbol::new("list->vector");
        assert_eq!(special.as_str(), "list->vector");

        let with_question = Symbol::new("null?");
        assert_eq!(with_question.as_str(), "null?");

        let with_bang = Symbol::new("set!");
        assert_eq!(with_bang.as_str(), "set!");
    }

    #[test]
    fn test_symbol_equality() {
        let s1 = Symbol::new("symbol");
        let s2 = Symbol::new("symbol");
        let s3 = Symbol::new("different");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        // Test equality with different creation methods
        let s4 = Symbol::from_string(std::string::String::from("symbol"));
        assert_eq!(s1, s4);
    }

    #[test]
    fn test_symbol_display() {
        let s = Symbol::new("my-variable");
        assert_eq!(format!("{}", s), "my-variable");

        let empty = Symbol::new("");
        assert_eq!(format!("{}", empty), "");

        let with_special = Symbol::new("string->number");
        assert_eq!(format!("{}", with_special), "string->number");
    }

    #[test]
    fn test_symbol_cloning() {
        let s1 = Symbol::new("test-symbol");
        let s2 = s1.clone();

        assert_eq!(s1, s2);
        assert_eq!(s1.as_str(), s2.as_str());

        // Both should reference the same underlying data
        assert!(Rc::ptr_eq(&s1.0, &s2.0));
    }

    #[test]
    fn test_symbol_conversion_traits() {
        // From &str
        let s1: Symbol = "hello".into();
        assert_eq!(s1.as_str(), "hello");

        // From owned String
        let owned = std::string::String::from("world");
        let s2: Symbol = owned.into();
        assert_eq!(s2.as_str(), "world");
    }

    #[test]
    fn test_symbol_edge_cases() {
        // Symbol with numbers
        let with_numbers = Symbol::new("var123");
        assert_eq!(with_numbers.as_str(), "var123");

        // Symbol with hyphens (common in Scheme)
        let with_hyphens = Symbol::new("multi-word-symbol");
        assert_eq!(with_hyphens.as_str(), "multi-word-symbol");

        // Symbol with special Scheme characters
        let scheme_chars = Symbol::new("<=>");
        assert_eq!(scheme_chars.as_str(), "<=>");

        // Very long symbol
        let long_symbol = Symbol::new(&"a".repeat(1000));
        assert_eq!(long_symbol.len(), 1000);
    }

    #[test]
    fn test_memory_efficiency() {
        let original = Symbol::new("shared-symbol");
        let cloned = original.clone();

        // Should share the same underlying memory
        assert!(Rc::ptr_eq(&original.0, &cloned.0));

        // Reference count should be 2
        assert_eq!(Rc::strong_count(&original.0), 2);
    }

    #[test]
    fn test_scheme_identifier_patterns() {
        // Test common Scheme identifier patterns
        let predicate = Symbol::new("number?");
        assert_eq!(predicate.as_str(), "number?");

        let mutator = Symbol::new("set-car!");
        assert_eq!(mutator.as_str(), "set-car!");

        let converter = Symbol::new("string->list");
        assert_eq!(converter.as_str(), "string->list");

        let arithmetic = Symbol::new("+");
        assert_eq!(arithmetic.as_str(), "+");

        let comparison = Symbol::new("<=");
        assert_eq!(comparison.as_str(), "<=");
    }
}
