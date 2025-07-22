//! Environment management for variable bindings
//!
//! This module provides the environment system for managing variable bindings
//! in the Scheme interpreter. Environments support lexical scoping through
//! parent environment chains and are designed to be thread-safe for use
//! in the fiber-based concurrency system.

use crate::types::Value;
use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-safe environment handle
///
/// The environment is wrapped in Arc<Mutex<>> to allow safe sharing
/// across multiple fibers and threads while maintaining mutability
/// for variable definitions.
pub type EnvironmentHandle = Arc<Mutex<Environment>>;

/// Environment for managing variable bindings
///
/// Supports lexical scoping through an optional parent environment.
/// All operations are designed to be thread-safe when accessed through
/// an EnvironmentHandle.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variable bindings in this environment scope
    bindings: HashMap<String, Value>,
    /// Optional parent environment for lexical scoping
    parent: Option<EnvironmentHandle>,
}

impl Environment {
    /// Create a new empty environment with no parent
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new environment with a parent environment
    pub fn with_parent(parent: EnvironmentHandle) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Create a new thread-safe environment handle
    pub fn new_handle() -> EnvironmentHandle {
        Arc::new(Mutex::new(Self::new()))
    }

    /// Create a new thread-safe environment handle with a parent
    pub fn new_handle_with_parent(parent: EnvironmentHandle) -> EnvironmentHandle {
        Arc::new(Mutex::new(Self::with_parent(parent)))
    }

    /// Define a variable in this environment
    ///
    /// This creates a new binding in the current environment scope,
    /// potentially shadowing bindings in parent environments.
    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Look up a variable in this environment or parent environments
    ///
    /// Searches the current environment first, then recursively
    /// searches parent environments until the variable is found
    /// or the top-level environment is reached.
    pub fn lookup(&self, name: &str) -> Result<Value> {
        // First check this environment
        if let Some(value) = self.bindings.get(name) {
            return Ok(value.clone());
        }

        // Then check parent environment if it exists
        if let Some(parent) = &self.parent {
            let parent_env = parent
                .lock()
                .map_err(|_| Error::parse_error("Environment lock poisoned"))?;
            return parent_env.lookup(name);
        }

        // Variable not found
        Err(Error::parse_error(&format!("Undefined variable: {}", name)))
    }

    /// Set a variable in the environment where it's defined
    ///
    /// Unlike define, this searches for the variable in the current
    /// environment and all parent environments, setting it where
    /// it's found. If the variable doesn't exist anywhere, returns an error.
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        // First check this environment
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            return Ok(());
        }

        // Then check parent environment if it exists
        if let Some(parent) = &self.parent {
            let mut parent_env = parent
                .lock()
                .map_err(|_| Error::parse_error("Environment lock poisoned"))?;
            return parent_env.set(name, value);
        }

        // Variable not found
        Err(Error::parse_error(&format!("Undefined variable: {}", name)))
    }

    /// Check if a variable is defined in this environment or any parent
    pub fn contains(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
            || self
                .parent
                .as_ref()
                .and_then(|p| p.lock().ok())
                .map_or(false, |env| env.contains(name))
    }

    /// Get the number of bindings in this environment (not including parents)
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if this environment has no bindings
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get all variable names defined in this environment (not including parents)
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.bindings.keys()
    }

    /// Get the parent environment handle if it exists
    pub fn parent(&self) -> Option<&EnvironmentHandle> {
        self.parent.as_ref()
    }
}

impl Default for Environment {
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
        let parent = Environment::new_handle();
        let env = Environment::with_parent(parent.clone());

        assert!(env.is_empty());
        assert!(env.parent().is_some());
    }

    #[test]
    fn test_environment_handles() {
        let handle = Environment::new_handle();
        assert!(handle.lock().is_ok());

        let parent = Environment::new_handle();
        let child_handle = Environment::new_handle_with_parent(parent);
        assert!(child_handle.lock().unwrap().parent().is_some());
    }

    #[test]
    fn test_variable_definition() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::number(42.0));

        assert!(!env.is_empty());
        assert_eq!(env.len(), 1);
        assert!(env.contains("x"));
    }

    #[test]
    fn test_variable_lookup() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::number(42.0));
        env.define("y".to_string(), Value::boolean(true));

        let result = env.lookup("x").unwrap();
        assert!(result.is_number());
        assert_eq!(result.as_number().unwrap(), 42.0);

        let result = env.lookup("y").unwrap();
        assert!(result.is_boolean());
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_variable_lookup_undefined() {
        let env = Environment::new();
        let result = env.lookup("undefined");

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
        let parent_handle = Environment::new_handle();
        {
            let mut parent = parent_handle.lock().unwrap();
            parent.define("x".to_string(), Value::number(42.0));
            parent.define("y".to_string(), Value::boolean(true));
        }

        let mut child = Environment::with_parent(parent_handle);
        child.define("z".to_string(), Value::string("hello"));

        // Should find variable in child
        let result = child.lookup("z").unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "hello");

        // Should find variable in parent
        let result = child.lookup("x").unwrap();
        assert!(result.is_number());
        assert_eq!(result.as_number().unwrap(), 42.0);

        let result = child.lookup("y").unwrap();
        assert!(result.is_boolean());
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_variable_shadowing() {
        let parent_handle = Environment::new_handle();
        {
            let mut parent = parent_handle.lock().unwrap();
            parent.define("x".to_string(), Value::number(42.0));
        }

        let mut child = Environment::with_parent(parent_handle);
        child.define("x".to_string(), Value::string("shadowed"));

        // Should find shadowed variable in child
        let result = child.lookup("x").unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "shadowed");
    }

    #[test]
    fn test_variable_set() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::number(42.0));

        // Set existing variable
        env.set("x", Value::number(99.0)).unwrap();
        let result = env.lookup("x").unwrap();
        assert_eq!(result.as_number().unwrap(), 99.0);

        // Try to set non-existent variable
        let result = env.set("y", Value::number(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_set_with_parent() {
        let parent_handle = Environment::new_handle();
        {
            let mut parent = parent_handle.lock().unwrap();
            parent.define("x".to_string(), Value::number(42.0));
        }

        let mut child = Environment::with_parent(parent_handle.clone());
        child.define("y".to_string(), Value::boolean(true));

        // Set variable in child
        child.set("y", Value::boolean(false)).unwrap();
        let result = child.lookup("y").unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);

        // Set variable in parent through child
        child.set("x", Value::number(99.0)).unwrap();
        let result = child.lookup("x").unwrap();
        assert_eq!(result.as_number().unwrap(), 99.0);

        // Verify parent was actually modified
        {
            let parent = parent_handle.lock().unwrap();
            let result = parent.lookup("x").unwrap();
            assert_eq!(result.as_number().unwrap(), 99.0);
        }
    }

    #[test]
    fn test_environment_contains() {
        let parent_handle = Environment::new_handle();
        {
            let mut parent = parent_handle.lock().unwrap();
            parent.define("parent_var".to_string(), Value::number(1.0));
        }

        let mut child = Environment::with_parent(parent_handle);
        child.define("child_var".to_string(), Value::number(2.0));

        assert!(child.contains("child_var"));
        assert!(child.contains("parent_var"));
        assert!(!child.contains("nonexistent"));
    }

    #[test]
    fn test_environment_keys() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::number(1.0));
        env.define("y".to_string(), Value::number(2.0));
        env.define("z".to_string(), Value::number(3.0));

        let keys: std::collections::HashSet<_> = env.keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"x".to_string()));
        assert!(keys.contains(&"y".to_string()));
        assert!(keys.contains(&"z".to_string()));
    }

    #[test]
    fn test_environment_thread_safety() {
        use std::thread;

        let env_handle = Environment::new_handle();

        // Define a variable in the main thread
        {
            let mut env = env_handle.lock().unwrap();
            env.define("shared_var".to_string(), Value::number(42.0));
        }

        let env_handle_clone = env_handle.clone();
        let handle = thread::spawn(move || {
            let env = env_handle_clone.lock().unwrap();
            env.lookup("shared_var").unwrap()
        });

        let result = handle.join().unwrap();
        assert!(result.is_number());
        assert_eq!(result.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_environment_concurrent_modification() {
        use std::thread;

        let env_handle = Environment::new_handle();
        let env_handle1 = env_handle.clone();
        let env_handle2 = env_handle.clone();

        let handle1 = thread::spawn(move || {
            for i in 0..100 {
                let mut env = env_handle1.lock().unwrap();
                env.define(format!("var_{}", i), Value::number(i as f64));
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 100..200 {
                let mut env = env_handle2.lock().unwrap();
                env.define(format!("var_{}", i), Value::number(i as f64));
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        let env = env_handle.lock().unwrap();
        assert_eq!(env.len(), 200);

        // Check some random values
        assert_eq!(env.lookup("var_50").unwrap().as_number().unwrap(), 50.0);
        assert_eq!(env.lookup("var_150").unwrap().as_number().unwrap(), 150.0);
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert!(env.is_empty());
        assert!(env.parent().is_none());
    }

    #[test]
    fn test_environment_chain_lookup() {
        // Create a chain: grandparent -> parent -> child
        let grandparent_handle = Environment::new_handle();
        {
            let mut grandparent = grandparent_handle.lock().unwrap();
            grandparent.define("level".to_string(), Value::string("grandparent"));
            grandparent.define("unique_gp".to_string(), Value::number(1.0));
        }

        let parent_handle = Environment::new_handle_with_parent(grandparent_handle);
        {
            let mut parent = parent_handle.lock().unwrap();
            parent.define("level".to_string(), Value::string("parent")); // shadows grandparent
            parent.define("unique_p".to_string(), Value::number(2.0));
        }

        let mut child = Environment::with_parent(parent_handle);
        child.define("level".to_string(), Value::string("child")); // shadows parent
        child.define("unique_c".to_string(), Value::number(3.0));

        // Test shadowing - should get child's value
        let result = child.lookup("level").unwrap();
        assert_eq!(result.as_string().unwrap(), "child");

        // Test unique variables at each level
        let result = child.lookup("unique_c").unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        let result = child.lookup("unique_p").unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);

        let result = child.lookup("unique_gp").unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }
}
