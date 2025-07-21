//! Symbol type implementation for Scheme
//!
//! This module implements the Symbol type that represents identifiers and
//! symbol values in Scheme. Symbols are like strings but represent identifiers.

use smol_str::SmolStr;

/// Symbol type for Scheme identifiers
///
/// Wraps SmolStr to enable efficient storage and sharing of symbol names.
/// Most symbols (≤23 bytes) are stack-allocated with O(1) clone operations.
/// Symbols are used for identifiers, variable names, and symbolic data in Scheme.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(SmolStr);

impl Symbol {
    /// Create a new Symbol from a string slice
    pub fn new(s: &str) -> Self {
        Symbol(SmolStr::new(s))
    }

    /// Create a new Symbol from an owned String
    pub fn from_string(s: std::string::String) -> Self {
        Symbol(SmolStr::from(s))
    }

    /// Get a string slice view of the symbol name
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the length of the symbol name in bytes
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the symbol name is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if this symbol is heap-allocated
    /// Returns true for symbols longer than 23 bytes
    pub fn is_heap_allocated(&self) -> bool {
        self.0.is_heap_allocated()
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

impl From<SmolStr> for Symbol {
    fn from(s: SmolStr) -> Self {
        Symbol(s)
    }
}

impl From<Symbol> for SmolStr {
    fn from(s: Symbol) -> Self {
        s.0
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

        // Clone should be very efficient with SmolStr
        assert_eq!(s1.len(), s2.len());
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

        // From SmolStr
        let smol = SmolStr::new("test");
        let s3: Symbol = smol.into();
        assert_eq!(s3.as_str(), "test");

        // To SmolStr
        let s4 = Symbol::new("convert");
        let smol2: SmolStr = s4.into();
        assert_eq!(smol2.as_str(), "convert");
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

        // Very long symbol (should be heap-allocated)
        let long_symbol = Symbol::new(&"a".repeat(1000));
        assert_eq!(long_symbol.len(), 1000);
        assert!(long_symbol.is_heap_allocated());
    }

    #[test]
    fn test_stack_vs_heap_allocation() {
        // Short symbols should be stack-allocated
        let short = Symbol::new("short");
        assert!(!short.is_heap_allocated());

        // Medium symbols (≤23 bytes) should still be stack-allocated
        let medium = Symbol::new("medium-length-symbol");
        assert_eq!(medium.len(), 20);
        assert!(!medium.is_heap_allocated());

        // Exactly 23 bytes should be stack-allocated
        let exactly23 = Symbol::new("exactly-twenty-three-b");
        assert_eq!(exactly23.len(), 22); // Actually 22, let me fix this
        let exactly23 = Symbol::new("exactly-twenty-three-by");
        assert_eq!(exactly23.len(), 23);
        assert!(!exactly23.is_heap_allocated());

        // Longer than 23 bytes should be heap-allocated
        let long = Symbol::new("this-symbol-is-definitely-longer-than-twenty-three-bytes");
        assert!(long.len() > 23);
        assert!(long.is_heap_allocated());
    }

    #[test]
    fn test_scheme_identifier_patterns() {
        // Test common Scheme identifier patterns
        let predicate = Symbol::new("number?");
        assert_eq!(predicate.as_str(), "number?");
        assert!(!predicate.is_heap_allocated());

        let mutator = Symbol::new("set-car!");
        assert_eq!(mutator.as_str(), "set-car!");
        assert!(!mutator.is_heap_allocated());

        let converter = Symbol::new("string->list");
        assert_eq!(converter.as_str(), "string->list");
        assert!(!converter.is_heap_allocated());

        let arithmetic = Symbol::new("+");
        assert_eq!(arithmetic.as_str(), "+");
        assert!(!arithmetic.is_heap_allocated());

        let comparison = Symbol::new("<=");
        assert_eq!(comparison.as_str(), "<=");
        assert!(!comparison.is_heap_allocated());
    }

    #[test]
    fn test_symbol_performance_characteristics() {
        // Test that common Scheme symbols are efficiently stored
        let common_symbols = [
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=", "car", "cdr", "cons", "list", "append",
            "length", "null?", "pair?", "number?", "string?", "symbol?", "if", "cond", "let",
            "define", "lambda", "quote", "set!", "begin", "and", "or", "not",
        ];

        for symbol_str in &common_symbols {
            let symbol = Symbol::new(symbol_str);
            assert_eq!(symbol.as_str(), *symbol_str);
            // All common symbols should be stack-allocated
            assert!(
                !symbol.is_heap_allocated(),
                "Symbol '{}' should be stack-allocated",
                symbol_str
            );
        }
    }

    #[test]
    fn test_thread_safe_sharing() {
        use std::sync::Arc;
        use std::thread;

        // Test that symbols can be shared across threads
        let short_symbol = Symbol::new("short");
        let long_symbol = Symbol::new(
            &"very-long-symbol-that-exceeds-twenty-three-bytes-and-should-be-heap-allocated",
        );

        // Verify allocation types
        assert!(!short_symbol.is_heap_allocated());
        assert!(long_symbol.is_heap_allocated());

        // Test thread-safe sharing
        let symbols = Arc::new((short_symbol, long_symbol));
        let mut handles = vec![];

        for i in 0..4 {
            let symbols_clone = Arc::clone(&symbols);
            let handle = thread::spawn(move || {
                let (short, long) = &*symbols_clone;

                // Both types should be accessible from different threads
                assert_eq!(short.as_str(), "short");
                assert_eq!(
                    long.as_str(),
                    "very-long-symbol-that-exceeds-twenty-three-bytes-and-should-be-heap-allocated"
                );

                // Clone operations should work in threads
                let short_clone = short.clone();
                let long_clone = long.clone();

                assert_eq!(short_clone.as_str(), "short");
                assert_eq!(
                    long_clone.as_str(),
                    "very-long-symbol-that-exceeds-twenty-three-bytes-and-should-be-heap-allocated"
                );

                i
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
