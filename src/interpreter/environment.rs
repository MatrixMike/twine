//! Environment management for variable bindings
//!
//! This module provides the environment system for managing variable bindings
//! in the Scheme interpreter. Environments support lexical scoping through
//! parent environment chains and are designed to be thread-safe for use
//! in the fiber-based concurrency system.

use crate::types::{Symbol, Value};
use crate::{Error, Result};
use std::collections::HashMap;

/// Environment for managing variable bindings
///
/// Supports lexical scoping through an optional parent environment reference.
/// Uses lifetimes to ensure parent environments outlive their children.
#[derive(Debug)]
pub struct Environment<'a> {
    /// Variable bindings in this environment scope
    bindings: HashMap<Symbol, Value>,
    /// Optional parent environment for lexical scoping
    parent: Option<&'a Environment<'a>>,
}

impl<'a> Environment<'a> {
    /// Create a new empty environment with no parent
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new environment with a parent environment reference
    pub fn new_scope(parent: &'a Environment<'a>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Create a new environment for closures by capturing specific bindings
    /// No parent reference needed - only specified bindings are copied
    pub fn new_closure(env: &Environment<'a>, keys: &[Symbol]) -> Environment<'static> {
        let mut bindings = HashMap::with_capacity(keys.len());

        for key in keys {
            // Use direct hash lookup for each key
            let mut current = Some(env);
            while let Some(environment) = current {
                if let Some(value) = environment.bindings.get(key) {
                    bindings.insert(key.clone(), value.clone());
                    break; // Found the variable, stop traversing for this key
                }
                current = environment.parent;
            }
        }

        Environment {
            bindings,
            parent: None,
        }
    }

    /// Define a variable in this environment
    ///
    /// This creates a new binding in the current environment scope,
    /// potentially shadowing bindings in parent environments.
    pub fn define(&mut self, name: Symbol, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Define a variable using a string name (convenience method)
    pub fn define_str(&mut self, name: &str, value: Value) {
        self.bindings.insert(Symbol::new(name), value);
    }

    /// Look up a variable by Symbol in this environment or parent environments
    pub fn lookup(&self, key: &Symbol) -> Result<Value> {
        // First check this environment
        if let Some(value) = self.bindings.get(key) {
            return Ok(value.clone());
        }

        // Then check parent environment if it exists
        if let Some(parent) = self.parent {
            return parent.lookup(key);
        }

        // Variable not found
        Err(Error::parse_error(&format!(
            "Undefined variable: {}",
            key.as_str()
        )))
    }

    /// Look up a variable by string name (convenience method)
    pub fn lookup_str(&self, name: &str) -> Result<Value> {
        let key = Symbol::new(name);
        self.lookup(&key)
    }

    /// Check if a variable is defined in this environment or any parent
    pub fn contains(&self, key: &Symbol) -> bool {
        self.bindings.contains_key(key) || self.parent.map_or(false, |parent| parent.contains(key))
    }

    /// Check if a variable is defined by string name (convenience method)
    pub fn contains_str(&self, name: &str) -> bool {
        let key = Symbol::new(name);
        self.contains(&key)
    }

    /// Get the number of bindings in this environment (not including parents)
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if this environment has no bindings
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get all variable symbols defined in this environment (not including parents)
    pub fn keys(&self) -> impl Iterator<Item = &Symbol> {
        self.bindings.keys()
    }

    /// Get the parent environment reference if it exists
    pub fn parent(&self) -> Option<&Environment<'a>> {
        self.parent
    }
}

impl<'a> Default for Environment<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_environment_creation() {
        let env = Environment::new();
        assert!(env.is_empty());
        assert_eq!(env.len(), 0);
        assert!(env.parent().is_none());
    }

    #[test]
    fn test_environment_with_parent() {
        let parent = Environment::new();
        let env = Environment::new_scope(&parent);

        assert!(env.is_empty());
        assert!(env.parent().is_some());
    }

    #[test]
    fn test_closure_creation() {
        let mut parent = Environment::new();
        parent.define_str("x", Value::number(42.0));
        parent.define_str("y", Value::string("hello"));
        parent.define_str("z", Value::boolean(true));

        // Create closure that captures only x and y
        let keys = vec![Symbol::new("x"), Symbol::new("y")];
        let closure_env = Environment::new_closure(&parent, &keys);

        assert_eq!(closure_env.len(), 2);
        assert!(closure_env.parent().is_none());
        assert!(closure_env.contains_str("x"));
        assert!(closure_env.contains_str("y"));
        assert!(!closure_env.contains_str("z")); // z not captured

        // Verify captured values
        assert_eq!(
            closure_env.lookup_str("x").unwrap().as_number().unwrap(),
            42.0
        );
        assert_eq!(
            closure_env.lookup_str("y").unwrap().as_string().unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_new_closure_direct_lookups() {
        let mut grandparent = Environment::new();
        grandparent.define_str("a", Value::number(1.0));
        grandparent.define_str("b", Value::number(2.0));

        let mut parent = Environment::new_scope(&grandparent);
        parent.define_str("c", Value::number(3.0));
        parent.define_str("d", Value::number(4.0));

        let mut child = Environment::new_scope(&parent);
        child.define_str("e", Value::number(5.0));

        // Create closure with bindings from different levels
        let keys = vec![
            Symbol::new("a"),           // from grandparent
            Symbol::new("c"),           // from parent
            Symbol::new("e"),           // from child
            Symbol::new("nonexistent"), // missing variable
        ];

        let closure_env = Environment::new_closure(&child, &keys);

        assert_eq!(closure_env.len(), 3); // Only found variables
        assert_eq!(
            closure_env.lookup_str("a").unwrap().as_number().unwrap(),
            1.0
        );
        assert_eq!(
            closure_env.lookup_str("c").unwrap().as_number().unwrap(),
            3.0
        );
        assert_eq!(
            closure_env.lookup_str("e").unwrap().as_number().unwrap(),
            5.0
        );
        assert!(closure_env.lookup_str("nonexistent").is_err());
    }

    #[test]
    fn test_new_closure_shadowing() {
        let mut parent = Environment::new();
        parent.define_str("x", Value::number(1.0));

        let mut child = Environment::new_scope(&parent);
        child.define_str("x", Value::number(2.0)); // shadows parent

        let keys = vec![Symbol::new("x")];
        let closure_env = Environment::new_closure(&child, &keys);

        // Should get child's value (shadowing)
        assert_eq!(closure_env.len(), 1);
        assert_eq!(
            closure_env.lookup_str("x").unwrap().as_number().unwrap(),
            2.0
        );
    }

    #[test]
    fn test_closure_creation_with_direct_lookups() {
        // Create a deep environment chain to test batch optimization
        let mut grandparent = Environment::new();
        grandparent.define_str("global1", Value::number(10.0));
        grandparent.define_str("global2", Value::string("hello"));
        grandparent.define_str("unused", Value::boolean(false));

        let mut parent = Environment::new_scope(&grandparent);
        parent.define_str("parent1", Value::number(20.0));
        parent.define_str("parent2", Value::symbol("test"));

        let mut child = Environment::new_scope(&parent);
        child.define_str("local1", Value::number(30.0));
        child.define_str("local2", Value::boolean(true));

        // Create closure that captures bindings from different levels
        let keys = vec![
            Symbol::new("global1"),     // from grandparent
            Symbol::new("parent1"),     // from parent
            Symbol::new("local1"),      // from child
            Symbol::new("nonexistent"), // missing variable (should be ignored)
        ];

        let closure_env = Environment::new_closure(&child, &keys);

        // Verify closure captured only the existing variables
        assert_eq!(closure_env.len(), 3); // Should have 3 variables (nonexistent ignored)
        assert!(closure_env.parent().is_none()); // No parent - standalone

        // Verify captured values are correct
        assert_eq!(
            closure_env
                .lookup_str("global1")
                .unwrap()
                .as_number()
                .unwrap(),
            10.0
        );
        assert_eq!(
            closure_env
                .lookup_str("parent1")
                .unwrap()
                .as_number()
                .unwrap(),
            20.0
        );
        assert_eq!(
            closure_env
                .lookup_str("local1")
                .unwrap()
                .as_number()
                .unwrap(),
            30.0
        );

        // Verify non-captured variables are not accessible
        assert!(closure_env.lookup_str("global2").is_err());
        assert!(closure_env.lookup_str("parent2").is_err());
        assert!(closure_env.lookup_str("local2").is_err());
        assert!(closure_env.lookup_str("unused").is_err());
        assert!(closure_env.lookup_str("nonexistent").is_err());
    }

    #[test]
    fn test_variable_definition() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(42.0));

        assert!(!env.is_empty());
        assert_eq!(env.len(), 1);
        assert!(env.contains_str("x"));
    }

    #[test]
    fn test_variable_lookup() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(42.0));
        env.define_str("y", Value::boolean(true));

        let result = env.lookup_str("x").unwrap();
        assert!(result.is_number());
        assert_eq!(result.as_number().unwrap(), 42.0);

        let result = env.lookup_str("y").unwrap();
        assert!(result.is_boolean());
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_variable_lookup_undefined() {
        let env = Environment::new();
        let result = env.lookup_str("undefined");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Undefined variable")
        );
    }

    #[test]
    fn test_variable_lookup_with_parent() {
        let mut parent = Environment::new();
        parent.define_str("x", Value::number(42.0));
        parent.define_str("y", Value::boolean(true));

        let mut child = Environment::new_scope(&parent);
        child.define_str("z", Value::string("hello"));

        // Should find variable in child
        let result = child.lookup_str("z").unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "hello");

        // Should find variable in parent
        let result = child.lookup_str("x").unwrap();
        assert!(result.is_number());
        assert_eq!(result.as_number().unwrap(), 42.0);

        let result = child.lookup_str("y").unwrap();
        assert!(result.is_boolean());
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_variable_shadowing() {
        let mut parent = Environment::new();
        parent.define_str("x", Value::number(42.0));

        let mut child = Environment::new_scope(&parent);
        child.define_str("x", Value::string("shadowed"));

        // Should find shadowed variable in child
        let result = child.lookup_str("x").unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "shadowed");
    }

    #[test]
    fn test_environment_contains() {
        let mut parent = Environment::new();
        parent.define_str("parent_var", Value::number(1.0));

        let mut child = Environment::new_scope(&parent);
        child.define_str("child_var", Value::number(2.0));

        assert!(child.contains_str("child_var"));
        assert!(child.contains_str("parent_var"));
        assert!(!child.contains_str("nonexistent"));
    }

    #[test]
    fn test_environment_keys() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(1.0));
        env.define_str("y", Value::number(2.0));
        env.define_str("z", Value::number(3.0));

        let keys: std::collections::HashSet<_> = env.keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&Symbol::new("x")));
        assert!(keys.contains(&Symbol::new("y")));
        assert!(keys.contains(&Symbol::new("z")));
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert!(env.is_empty());
        assert!(env.parent().is_none());
    }

    #[test]
    fn test_environment_chain_lookup() {
        let mut grandparent = Environment::new();
        grandparent.define_str("level", Value::string("grandparent"));
        grandparent.define_str("unique_gp", Value::number(1.0));

        let mut parent = Environment::new_scope(&grandparent);
        parent.define_str("level", Value::string("parent")); // shadows grandparent
        parent.define_str("unique_p", Value::number(2.0));

        let mut child = Environment::new_scope(&parent);
        child.define_str("level", Value::string("child")); // shadows parent
        child.define_str("unique_c", Value::number(3.0));

        // Test shadowing - should get child's value
        let result = child.lookup_str("level").unwrap();
        assert_eq!(result.as_string().unwrap(), "child");

        // Test unique variables at each level
        let result = child.lookup_str("unique_c").unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        let result = child.lookup_str("unique_p").unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);

        let result = child.lookup_str("unique_gp").unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }
}
