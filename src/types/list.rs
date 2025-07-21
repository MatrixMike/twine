//! List type implementation for Scheme
//!
//! This module implements the List type that represents immutable list values
//! in Scheme. Lists are the fundamental compound data structure in Scheme.

use std::sync::Arc;

// Forward declaration to avoid circular dependency
// The actual Value enum is defined in value.rs
#[allow(dead_code)]
type Value = crate::types::Value;

/// Immutable list type for Scheme
///
/// Wraps a thread-safe reference-counted vector to enable efficient sharing
/// across multiple threads while maintaining immutability guarantees required
/// by Scheme semantics. Lists are the primary compound data structure in Scheme.
#[derive(Debug, Clone, PartialEq)]
pub struct List(Arc<Vec<Value>>);

impl List {
    /// Create a new empty list
    pub fn new() -> Self {
        List(Arc::new(Vec::new()))
    }

    /// Create a new list from a vector of values
    pub fn from_vec(values: Vec<crate::types::Value>) -> Self {
        List(Arc::new(values))
    }

    /// Create a new list from an iterator of values
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        List(Arc::new(iter.into_iter().collect()))
    }

    /// Get the length of the list
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get a value at the specified index
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Get an iterator over the list elements
    pub fn iter(&self) -> std::slice::Iter<'_, Value> {
        self.0.iter()
    }

    /// Convert the list into a vector (cloning the underlying data)
    pub fn into_vec(self) -> Vec<Value> {
        match Arc::try_unwrap(self.0) {
            Ok(vec) => vec,
            Err(arc) => (*arc).clone(),
        }
    }

    /// Get a slice view of the list contents
    pub fn as_slice(&self) -> &[Value] {
        &self.0
    }
}

impl Default for List {
    fn default() -> Self {
        List::new()
    }
}

impl std::fmt::Display for List {
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

impl From<Vec<Value>> for List {
    fn from(vec: Vec<Value>) -> Self {
        List::from_vec(vec)
    }
}

impl FromIterator<Value> for List {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        List::from_iter(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_list_creation() {
        // Test empty list creation
        let empty = List::new();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        // Test list creation from vector
        let values = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let list = List::from_vec(values.clone());
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());

        // Test list creation from iterator
        let iter_list = List::from_iter(values.into_iter());
        assert_eq!(iter_list.len(), 3);

        // Test default creation
        let default_list = List::default();
        assert!(default_list.is_empty());
    }

    #[test]
    fn test_list_access() {
        let list = List::from_vec(vec![
            Value::number(10.0),
            Value::string("hello"),
            Value::boolean(true),
        ]);

        // Test get method
        assert_eq!(list.get(0), Some(&Value::number(10.0)));
        assert_eq!(list.get(1), Some(&Value::string("hello")));
        assert_eq!(list.get(2), Some(&Value::boolean(true)));
        assert_eq!(list.get(3), None);

        // Test iterator
        let collected: Vec<_> = list.iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], &Value::number(10.0));
        assert_eq!(collected[1], &Value::string("hello"));
        assert_eq!(collected[2], &Value::boolean(true));

        // Test as_slice
        let slice = list.as_slice();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], Value::number(10.0));
    }

    #[test]
    fn test_list_display() {
        // Test empty list display
        let empty = List::new();
        assert_eq!(format!("{}", empty), "()");

        // Test single element list
        let single = List::from_vec(vec![Value::number(42.0)]);
        assert_eq!(format!("{}", single), "(42)");

        // Test multi-element list
        let multi = List::from_vec(vec![
            Value::number(1.0),
            Value::string("hello"),
            Value::boolean(true),
        ]);
        assert_eq!(format!("{}", multi), "(1 \"hello\" #t)");

        // Test nested list
        let nested = List::from_vec(vec![
            Value::number(1.0),
            Value::list(vec![Value::number(2.0), Value::number(3.0)]),
            Value::number(4.0),
        ]);
        assert_eq!(format!("{}", nested), "(1 (2 3) 4)");
    }

    #[test]
    fn test_list_equality() {
        let list1 = List::from_vec(vec![Value::number(1.0), Value::number(2.0)]);
        let list2 = List::from_vec(vec![Value::number(1.0), Value::number(2.0)]);
        let list3 = List::from_vec(vec![Value::number(1.0), Value::number(3.0)]);

        assert_eq!(list1, list2);
        assert_ne!(list1, list3);

        // Test empty list equality
        let empty1 = List::new();
        let empty2 = List::new();
        assert_eq!(empty1, empty2);
    }

    #[test]
    fn test_list_cloning() {
        let original = List::from_vec(vec![
            Value::number(1.0),
            Value::string("test"),
            Value::boolean(false),
        ]);
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.len(), cloned.len());

        // Should share the same underlying data
        assert!(Arc::ptr_eq(&original.0, &cloned.0));
    }

    #[test]
    fn test_list_conversion_traits() {
        // From Vec
        let vec = vec![Value::number(1.0), Value::number(2.0)];
        let list: List = vec.into();
        assert_eq!(list.len(), 2);

        // FromIterator
        let values = vec![Value::string("a"), Value::string("b"), Value::string("c")];
        let list: List = values.into_iter().collect();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_list_into_vec() {
        let original_vec = vec![Value::number(1.0), Value::number(2.0), Value::number(3.0)];
        let list = List::from_vec(original_vec.clone());
        let converted_vec = list.into_vec();

        assert_eq!(converted_vec, original_vec);
    }

    #[test]
    fn test_list_edge_cases() {
        // Very large list
        let large_vec: Vec<_> = (0..1000).map(|i| Value::number(i as f64)).collect();
        let large_list = List::from_vec(large_vec);
        assert_eq!(large_list.len(), 1000);
        assert_eq!(large_list.get(999), Some(&Value::number(999.0)));

        // List with mixed types
        let mixed = List::from_vec(vec![
            Value::number(42.0),
            Value::string("hello"),
            Value::boolean(true),
            Value::symbol("symbol"),
            Value::nil(),
            Value::list(vec![Value::number(1.0)]),
        ]);
        assert_eq!(mixed.len(), 6);

        // Deeply nested lists
        let deep = List::from_vec(vec![Value::list(vec![Value::list(vec![Value::list(
            vec![Value::number(1.0)],
        )])])]);
        assert_eq!(format!("{}", deep), "((((1))))");
    }

    #[test]
    fn test_memory_efficiency() {
        let original = List::from_vec(vec![Value::number(1.0), Value::number(2.0)]);
        let cloned = original.clone();

        // Should share the same underlying memory
        assert!(Arc::ptr_eq(&original.0, &cloned.0));

        // Reference count should be 2
        assert_eq!(Arc::strong_count(&original.0), 2);
    }

    #[test]
    fn test_list_iteration() {
        let list = List::from_vec(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);

        // Test iterator collecting
        let collected: Vec<_> = list.iter().cloned().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], Value::number(1.0));
        assert_eq!(collected[2], Value::number(3.0));

        // Test iterator with enumeration
        for (i, value) in list.iter().enumerate() {
            assert_eq!(value, &Value::number((i + 1) as f64));
        }
    }
}
