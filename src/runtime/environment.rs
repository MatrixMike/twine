//! Environment management for identifier bindings
//!
//! This module provides the environment system for managing identifier bindings
//! in the Scheme runtime. Environments support lexical scoping through
//! parent environment chains and are designed to be thread-safe for use
//! in the fiber-based concurrency system.

use crate::types::{Symbol, Value};
use crate::{Error, Result};
use std::collections::HashMap;

/// Environment for managing identifier bindings
///
/// Supports lexical scoping through an optional parent environment reference.
/// Uses lifetimes to ensure parent environments outlive their children.
#[derive(Debug)]
pub struct Environment<'a> {
    /// Identifier bindings in this environment scope
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
    pub fn new_closure(env: &Environment<'a>, identifiers: &[Symbol]) -> Environment<'static> {
        let mut bindings = HashMap::with_capacity(identifiers.len());

        for identifier in identifiers {
            // Use direct hash lookup for each identifier
            let mut current = Some(env);
            while let Some(environment) = current {
                if let Some(value) = environment.bindings.get(identifier) {
                    bindings.insert(identifier.clone(), value.clone());
                    break; // Found the identifier, stop traversing for this identifier
                }
                current = environment.parent;
            }
        }

        Environment {
            bindings,
            parent: None,
        }
    }

    /// Define an identifier binding in this environment
    ///
    /// This creates a new binding in the current environment scope,
    /// potentially shadowing bindings in parent environments.
    pub fn define(&mut self, identifier: Symbol, value: Value) {
        self.bindings.insert(identifier, value);
    }

    /// Define an identifier binding using a string key (convenience method)
    pub fn define_str(&mut self, identifier: &str, value: Value) {
        self.bindings.insert(Symbol::new(identifier), value);
    }

    /// Look up a binding by identifier in this environment or parent environments
    pub fn lookup(&self, identifier: &Symbol) -> Result<Value> {
        // First check this environment
        if let Some(value) = self.bindings.get(identifier) {
            return Ok(value.clone());
        }

        // Then check parent environment if it exists
        if let Some(parent) = self.parent {
            return parent.lookup(identifier);
        }

        // Identifier not found - provide detailed error
        self.create_unbound_identifier_error(identifier)
    }

    /// Create a detailed unbound identifier error with suggestions
    fn create_unbound_identifier_error(&self, identifier: &Symbol) -> Result<Value> {
        let identifier_str = identifier.as_str();

        // Collect similar identifiers for suggestions
        let suggestions = self.find_similar_identifiers(identifier_str);

        if suggestions.is_empty() {
            Err(Error::unbound_identifier(identifier_str))
        } else {
            let context = format!("Did you mean one of: {}?", suggestions.join(", "));
            Err(Error::unbound_identifier_with_context(
                identifier_str,
                &context,
            ))
        }
    }

    /// Find similar identifiers in the environment chain for suggestions
    fn find_similar_identifiers(&self, target: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Collect all identifiers from this environment and parents
        let mut current = Some(self);
        while let Some(env) = current {
            for identifier in env.bindings.keys() {
                let identifier_str = identifier.as_str();
                if seen.insert(identifier_str.to_string()) {
                    // Simple similarity check: same length or edit distance of 1-2
                    if is_similar_identifier(target, identifier_str) {
                        suggestions.push(format!("'{}'", identifier_str));
                    }
                }
            }
            current = env.parent;
        }

        // Limit suggestions to avoid overwhelming output
        suggestions.truncate(3);
        suggestions
    }

    /// Look up a binding by string key (convenience method)
    pub fn lookup_str(&self, identifier: &str) -> Result<Value> {
        let symbol = Symbol::new(identifier);
        self.lookup(&symbol)
    }

    /// Check if an identifier is defined in this environment or any parent
    pub fn contains(&self, identifier: &Symbol) -> bool {
        self.bindings.contains_key(identifier)
            || self
                .parent
                .map_or(false, |parent| parent.contains(identifier))
    }

    /// Check if an identifier is defined by string key (convenience method)
    pub fn contains_str(&self, identifier: &str) -> bool {
        let symbol = Symbol::new(identifier);
        self.contains(&symbol)
    }

    /// Get the number of bindings in this environment (not including parents)
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if this environment has no bindings
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Get all identifier symbols defined in this environment (not including parents)
    pub fn keys(&self) -> impl Iterator<Item = &Symbol> {
        self.bindings.keys()
    }

    /// Get the parent environment reference if it exists
    pub fn parent(&self) -> Option<&Environment<'a>> {
        self.parent
    }

    /// Get information about the environment chain depth
    pub fn chain_depth(&self) -> usize {
        let mut depth = 1;
        let mut current = self.parent;
        while let Some(env) = current {
            depth += 1;
            current = env.parent;
        }
        depth
    }

    /// Find the environment level where an identifier is bound
    pub fn find_binding_level(&self, identifier: &Symbol) -> Option<usize> {
        let mut level = 0;
        let mut current = Some(self);

        while let Some(env) = current {
            if env.bindings.contains_key(identifier) {
                return Some(level);
            }
            level += 1;
            current = env.parent;
        }

        None
    }
}

impl<'a> Default for Environment<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to check if two identifiers are similar
fn is_similar_identifier(target: &str, candidate: &str) -> bool {
    if target == candidate {
        return false; // Exact match is not a "similar" suggestion
    }

    let target_len = target.len();
    let candidate_len = candidate.len();

    // Same length - check for single character differences
    if target_len == candidate_len {
        let differences = target
            .chars()
            .zip(candidate.chars())
            .filter(|(a, b)| a != b)
            .count();
        return differences <= 2;
    }

    // Length difference of 1 - check for insertion/deletion
    if (target_len as i32 - candidate_len as i32).abs() == 1 {
        let (shorter, longer) = if target_len < candidate_len {
            (target, candidate)
        } else {
            (candidate, target)
        };

        // Check if longer string contains shorter as subsequence with one extra char
        return edit_distance_one(shorter, longer);
    }

    false
}

/// Check if two strings have edit distance of at most 1
fn edit_distance_one(shorter: &str, longer: &str) -> bool {
    let mut i = 0;
    let mut j = 0;
    let mut found_difference = false;

    let shorter_chars: Vec<char> = shorter.chars().collect();
    let longer_chars: Vec<char> = longer.chars().collect();

    while i < shorter_chars.len() && j < longer_chars.len() {
        if shorter_chars[i] == longer_chars[j] {
            i += 1;
            j += 1;
        } else if !found_difference {
            found_difference = true;
            if shorter.len() == longer.len() {
                // Substitution
                i += 1;
                j += 1;
            } else {
                // Insertion in longer string
                j += 1;
            }
        } else {
            return false; // More than one difference
        }
    }

    true
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
        let identifiers = vec![Symbol::new("x"), Symbol::new("y")];
        let closure_env = Environment::new_closure(&parent, &identifiers);

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
        let identifiers = vec![
            Symbol::new("a"),           // from grandparent
            Symbol::new("c"),           // from parent
            Symbol::new("e"),           // from child
            Symbol::new("nonexistent"), // missing identifier
        ];

        let closure_env = Environment::new_closure(&child, &identifiers);

        assert_eq!(closure_env.len(), 3); // Only found identifiers
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

        let identifiers = vec![Symbol::new("x")];
        let closure_env = Environment::new_closure(&child, &identifiers);

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
        let identifiers = vec![
            Symbol::new("global1"),     // from grandparent
            Symbol::new("parent1"),     // from parent
            Symbol::new("local1"),      // from child
            Symbol::new("nonexistent"), // missing identifier (should be ignored)
        ];

        let closure_env = Environment::new_closure(&child, &identifiers);

        // Verify closure captured only the existing identifiers
        assert_eq!(closure_env.len(), 3); // Should have 3 identifiers (nonexistent ignored)
        assert!(closure_env.parent().is_none()); // No parent - standalone

        // Verify captured bindings are correct
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

        // Verify non-captured identifiers are not accessible
        assert!(closure_env.lookup_str("global2").is_err());
        assert!(closure_env.lookup_str("parent2").is_err());
        assert!(closure_env.lookup_str("local2").is_err());
        assert!(closure_env.lookup_str("unused").is_err());
        assert!(closure_env.lookup_str("nonexistent").is_err());
    }

    #[test]
    fn test_identifier_definition() {
        let mut env = Environment::new();
        env.define_str("x", Value::number(42.0));

        assert!(!env.is_empty());
        assert_eq!(env.len(), 1);
        assert!(env.contains_str("x"));
    }

    #[test]
    fn test_identifier_lookup() {
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
    fn test_identifier_lookup_undefined() {
        let env = Environment::new();
        let result = env.lookup_str("undefined");

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unbound identifier")
        );
    }

    #[test]
    fn test_identifier_lookup_with_parent() {
        let mut parent = Environment::new();
        parent.define_str("x", Value::number(42.0));
        parent.define_str("y", Value::boolean(true));

        let mut child = Environment::new_scope(&parent);
        child.define_str("z", Value::string("hello"));

        // Should find identifier in child
        let result = child.lookup_str("z").unwrap();
        assert!(result.is_string());
        assert_eq!(result.as_string().unwrap(), "hello");

        // Should find identifier in parent
        let result = child.lookup_str("x").unwrap();
        assert!(result.is_number());
        assert_eq!(result.as_number().unwrap(), 42.0);

        let result = child.lookup_str("y").unwrap();
        assert!(result.is_boolean());
        assert_eq!(result.as_boolean().unwrap(), true);
    }

    #[test]
    fn test_identifier_shadowing() {
        let mut parent = Environment::new();
        parent.define_str("x", Value::number(42.0));

        let mut child = Environment::new_scope(&parent);
        child.define_str("x", Value::string("shadowed"));

        // Should find shadowed identifier in child
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

        // Test unique identifiers at each level
        let result = child.lookup_str("unique_c").unwrap();
        assert_eq!(result.as_number().unwrap(), 3.0);

        let result = child.lookup_str("unique_p").unwrap();
        assert_eq!(result.as_number().unwrap(), 2.0);

        let result = child.lookup_str("unique_gp").unwrap();
        assert_eq!(result.as_number().unwrap(), 1.0);
    }

    #[test]
    fn test_detailed_unbound_identifier_errors() {
        let mut env = Environment::new();
        env.define_str("identifier", Value::number(1.0));
        env.define_str("function", Value::number(2.0));

        // Test error with similar identifier suggestion
        let result = env.lookup_str("identifer"); // typo
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Unbound identifier"));
        assert!(error.to_string().contains("Did you mean"));
        assert!(error.to_string().contains("'identifier'"));
    }

    #[test]
    fn test_environment_chain_depth() {
        let grandparent = Environment::new();
        assert_eq!(grandparent.chain_depth(), 1);

        let parent = Environment::new_scope(&grandparent);
        assert_eq!(parent.chain_depth(), 2);

        let child = Environment::new_scope(&parent);
        assert_eq!(child.chain_depth(), 3);
    }

    #[test]
    fn test_binding_level_detection() {
        let mut grandparent = Environment::new();
        grandparent.define_str("gp_var", Value::number(1.0));

        let mut parent = Environment::new_scope(&grandparent);
        parent.define_str("parent_var", Value::number(2.0));

        let mut child = Environment::new_scope(&parent);
        child.define_str("child_var", Value::number(3.0));

        // Test binding level detection
        assert_eq!(child.find_binding_level(&Symbol::new("child_var")), Some(0));
        assert_eq!(
            child.find_binding_level(&Symbol::new("parent_var")),
            Some(1)
        );
        assert_eq!(child.find_binding_level(&Symbol::new("gp_var")), Some(2));
        assert_eq!(child.find_binding_level(&Symbol::new("nonexistent")), None);
    }

    #[test]
    fn test_similar_identifier_detection() {
        let mut env = Environment::new();
        env.define_str("identifier", Value::number(1.0));
        env.define_str("function", Value::number(2.0));
        env.define_str("procedure", Value::number(3.0));

        // Test various typo patterns
        let test_cases = [
            ("identifer", true),  // single character substitution
            ("functio", true),    // single character deletion
            ("procedurex", true), // single character addition
            ("xyz", false),       // completely different
        ];

        for (typo, should_suggest) in test_cases {
            let result = env.lookup_str(typo);
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();

            if should_suggest {
                assert!(
                    error_msg.contains("Did you mean"),
                    "Should suggest for '{}': {}",
                    typo,
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_error_message_quality() {
        let env = Environment::new();
        let result = env.lookup_str("undefined_identifier");

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();

        // Verify error message contains helpful information
        assert!(error_msg.contains("Unbound identifier"));
        assert!(error_msg.contains("undefined_identifier"));
        // No generic context since there are no similar identifiers
        assert_eq!(error_msg, "Unbound identifier: 'undefined_identifier'");
    }

    #[test]
    fn test_enhanced_error_handling_demo() {
        let mut parent = Environment::new();
        parent.define_str("count", Value::number(42.0));
        parent.define_str("result", Value::string("success"));

        let mut child = Environment::new_scope(&parent);
        child.define_str("local_var", Value::boolean(true));

        // Test 1: Normal binding definition (shadowing is allowed)
        child.define_str("count", Value::number(100.0)); // Shadows parent binding normally

        // Test 2: Detailed unbound identifier error with suggestions
        let error = child.lookup_str("cout").unwrap_err(); // typo for "count"
        let error_msg = error.to_string();
        assert!(error_msg.contains("Unbound identifier: 'cout'"));
        assert!(error_msg.contains("Did you mean one of: 'count'"));

        // Test 3: Error without suggestions for completely different identifier
        let error = child.lookup_str("xyz").unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("Unbound identifier: 'xyz'"));
        assert_eq!(error_msg, "Unbound identifier: 'xyz'");

        // Test 4: Environment chain information
        assert_eq!(child.chain_depth(), 2);
        assert_eq!(child.find_binding_level(&Symbol::new("count")), Some(0)); // count is redefined in child
        assert_eq!(child.find_binding_level(&Symbol::new("result")), Some(1)); // result is only in parent
        assert_eq!(child.find_binding_level(&Symbol::new("local_var")), Some(0));
    }

    #[test]
    fn test_why_unbound_identifier_errors_matter() {
        // This test demonstrates why unbound identifier errors are essential
        let mut env = Environment::new();
        env.define_str("counter", Value::number(42.0));
        env.define_str("width", Value::number(10.0));
        env.define_str("calculate", Value::string("function"));

        // Scenario 1: Typo in identifier name - close enough for suggestions
        let result = env.lookup_str("counte"); // missing 'r' from "counter"
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unbound identifier"));
        // May or may not have suggestions depending on similarity

        // Scenario 2: Completely missing identifier
        // Without error handling: program continues with undefined state
        // With error handling: clear error message prevents silent bugs
        let result = env.lookup_str("height"); // forgot to define
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unbound identifier: 'height'"));

        // Scenario 3: Close typo that should get suggestions
        let result = env.lookup_str("calcuate"); // missing 'l' from "calculate"
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unbound identifier"));

        // The key point: ALL of these return proper errors instead of:
        // 1. Crashing the interpreter
        // 2. Returning null/undefined values
        // 3. Continuing with garbage data
        // 4. Silent failures that are hard to debug

        // This demonstrates why unbound identifier detection is essential:
        // - Prevents runtime crashes and undefined behavior
        // - Provides clear feedback about what went wrong
        // - Helps catch typos and missing definitions early
        // - Ensures the interpreter behaves predictably
        // - Meets Scheme language specification requirements
    }

    #[test]
    fn test_normal_shadowing_behavior() {
        let mut parent = Environment::new();
        parent.define_str("x", Value::number(42.0));
        parent.define_str("y", Value::string("parent"));

        let mut child = Environment::new_scope(&parent);

        // Test normal shadowing - should work without warnings
        child.define_str("x", Value::number(100.0)); // Shadows parent's x
        child.define_str("z", Value::boolean(true)); // New binding

        // Verify shadowing works correctly
        assert_eq!(child.lookup_str("x").unwrap().as_number().unwrap(), 100.0); // Child's value
        assert_eq!(
            child.lookup_str("y").unwrap().as_string().unwrap(),
            "parent"
        ); // Parent's value
        assert_eq!(child.lookup_str("z").unwrap().as_boolean().unwrap(), true); // Child's value

        // Test redefinition in same scope
        child.define_str("z", Value::number(99.0)); // Redefine z
        assert_eq!(child.lookup_str("z").unwrap().as_number().unwrap(), 99.0);

        // Verify parent environment unchanged
        assert_eq!(parent.lookup_str("x").unwrap().as_number().unwrap(), 42.0);
    }
}
