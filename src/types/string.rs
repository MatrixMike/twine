//! String type implementation for Scheme
//!
//! This module implements the String type that represents immutable string values
//! in Scheme. Provides efficient string handling with proper abstraction.

use std::sync::Arc;

/// Immutable string type for Scheme
///
/// Wraps a thread-safe reference-counted string to enable efficient sharing
/// across multiple threads while maintaining immutability guarantees required
/// by Scheme semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArcString(Arc<String>);

impl ArcString {
    /// Create a new ArcString from a string slice
    pub fn new(s: &str) -> Self {
        ArcString(Arc::new(s.to_string()))
    }

    /// Create a new ArcString from an owned String
    pub fn from_string(s: String) -> Self {
        ArcString(Arc::new(s))
    }

    /// Get a string slice view of the contents
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

impl std::fmt::Display for ArcString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ArcString {
    fn from(s: &str) -> Self {
        ArcString::new(s)
    }
}

impl From<String> for ArcString {
    fn from(s: String) -> Self {
        ArcString::from_string(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_creation() {
        // Test creation from &str
        let s1 = ArcString::new("hello");
        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s1.len(), 5);
        assert!(!s1.is_empty());

        // Test creation from owned String
        let owned = String::from("world");
        let s2 = ArcString::from_string(owned);
        assert_eq!(s2.as_str(), "world");
        assert_eq!(s2.len(), 5);

        // Test empty string
        let empty = ArcString::new("");
        assert_eq!(empty.as_str(), "");
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());

        // Test unicode string
        let unicode = ArcString::new("Hello, 世界!");
        assert_eq!(unicode.as_str(), "Hello, 世界!");
        assert!(unicode.len() > 9); // Unicode characters take more bytes
    }

    #[test]
    fn test_string_equality() {
        let s1 = ArcString::new("hello");
        let s2 = ArcString::new("hello");
        let s3 = ArcString::new("world");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        // Test equality with different creation methods
        let s4 = ArcString::from_string(String::from("hello"));
        assert_eq!(s1, s4);
    }

    #[test]
    fn test_string_display() {
        let s = ArcString::new("Hello, world!");
        assert_eq!(format!("{}", s), "Hello, world!");

        let empty = ArcString::new("");
        assert_eq!(format!("{}", empty), "");

        let unicode = ArcString::new("こんにちは");
        assert_eq!(format!("{}", unicode), "こんにちは");
    }

    #[test]
    fn test_string_cloning() {
        let s1 = ArcString::new("test string");
        let s2 = s1.clone();

        assert_eq!(s1, s2);
        assert_eq!(s1.as_str(), s2.as_str());

        // Both should reference the same underlying data
        assert!(Arc::ptr_eq(&s1.0, &s2.0));
    }

    #[test]
    fn test_string_conversion_traits() {
        // From &str
        let s1: ArcString = "hello".into();
        assert_eq!(s1.as_str(), "hello");

        // From owned String
        let owned = String::from("world");
        let s2: ArcString = owned.into();
        assert_eq!(s2.as_str(), "world");
    }

    #[test]
    fn test_string_edge_cases() {
        // Very long string
        let long_str = "a".repeat(10000);
        let s = ArcString::new(&long_str);
        assert_eq!(s.len(), 10000);
        assert_eq!(s.as_str(), long_str);

        // String with special characters
        let special = ArcString::new("Hello\nWorld\t!");
        assert_eq!(special.as_str(), "Hello\nWorld\t!");

        // String with null bytes
        let with_null = ArcString::new("Hello\0World");
        assert_eq!(with_null.as_str(), "Hello\0World");
        assert_eq!(with_null.len(), 11);
    }

    #[test]
    fn test_memory_efficiency() {
        let original = ArcString::new("shared string");
        let cloned = original.clone();

        // Should share the same underlying memory
        assert!(Arc::ptr_eq(&original.0, &cloned.0));

        // Reference count should be 2
        assert_eq!(Arc::strong_count(&original.0), 2);
    }
}
