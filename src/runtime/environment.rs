//! Environment management for identifier bindings
//!
//! This module provides the environment system for managing identifier bindings
//! in the Scheme runtime. Environments support lexical scoping through
//! parent environment chains and are designed to be thread-safe for use
//! in the fiber-based concurrency system.

use crate::runtime::builtins::Builtin;
use crate::types::{Procedure, Symbol, Value};
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

        // Check for builtin procedures before failing
        if let Some(builtin) = Builtin::from_name(identifier.as_str()) {
            let procedure = Procedure::builtin(builtin);
            return Ok(Value::procedure(procedure));
        }

        // Identifier not found - provide detailed error
        self.create_unbound_identifier_error(identifier)
    }

    /// Create a detailed unbound identifier error with suggestions
    fn create_unbound_identifier_error(&self, identifier: &Symbol) -> Result<Value> {
        // Collect similar identifiers for suggestions
        let suggestions = self.find_similar_identifiers(identifier);

        if suggestions.is_empty() {
            Err(Error::unbound_identifier(identifier.as_str(), None))
        } else {
            let formatted_suggestions: Vec<String> =
                suggestions.iter().map(|s| format!("'{s}'")).collect();
            let context = format!("Did you mean one of: {}?", formatted_suggestions.join(", "));
            Err(Error::unbound_identifier(
                identifier.as_str(),
                Some(&context),
            ))
        }
    }

    /// Find similar identifiers in the environment chain for suggestions
    fn find_similar_identifiers(&self, target: &Symbol) -> Vec<Symbol> {
        let mut suggestions = Vec::new();
        let mut seen = std::collections::HashSet::<Symbol>::new();

        // Collect all identifiers from this environment and parents
        let mut current = Some(self);
        while let Some(env) = current {
            for identifier in env.bindings.keys() {
                if seen.insert(identifier.clone()) {
                    // Simple similarity check: same length or edit distance of 1-2
                    if is_similar_identifier(target, identifier) {
                        suggestions.push(identifier.clone());
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
                .is_some_and(|parent| parent.contains(identifier))
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

    /// Flatten the environment chain into a single 'static environment
    ///
    /// Creates a new environment with all bindings from this environment
    /// and its parent chain. This is useful for lambda closures where
    /// we need to capture the environment without lifetime constraints.
    pub fn flatten(&self) -> Environment<'static> {
        let mut bindings = HashMap::new();
        let mut current = Some(self);

        // Traverse the environment chain and collect all bindings
        // Inner scopes override outer scopes for shadowing
        let mut levels = Vec::new();
        while let Some(env) = current {
            levels.push(env);
            current = env.parent;
        }

        // Apply bindings from outermost to innermost to preserve shadowing
        for env in levels.iter().rev() {
            for (identifier, value) in &env.bindings {
                bindings.insert(identifier.clone(), value.clone());
            }
        }

        Environment {
            bindings,
            parent: None,
        }
    }
}

impl<'a> Default for Environment<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to check if two identifiers are similar
fn is_similar_identifier(target: &Symbol, candidate: &Symbol) -> bool {
    if target == candidate {
        return false; // Exact match is not a "similar" suggestion
    }

    let target_str = target.as_str();
    let candidate_str = candidate.as_str();
    let target_len = target_str.len();
    let candidate_len = candidate_str.len();

    // Same length - check for single character differences
    if target_len == candidate_len {
        let differences = target_str
            .chars()
            .zip(candidate_str.chars())
            .filter(|(a, b)| a != b)
            .count();
        return differences <= 2;
    }

    // Length difference of 1 - check for insertion/deletion
    if (target_len as i32 - candidate_len as i32).abs() == 1 {
        let (shorter, longer) = if target_len < candidate_len {
            (target_str, candidate_str)
        } else {
            (candidate_str, target_str)
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
        assert!(result.as_boolean().unwrap());
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
        assert!(result.as_boolean().unwrap());
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
                    "Should suggest for '{typo}': {error_msg}"
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
        assert!(child.lookup_str("z").unwrap().as_boolean().unwrap()); // Child's value

        // Test redefinition in same scope
        child.define_str("z", Value::number(99.0)); // Redefine z
        assert_eq!(child.lookup_str("z").unwrap().as_number().unwrap(), 99.0);

        // Verify parent environment unchanged
        assert_eq!(parent.lookup_str("x").unwrap().as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_lifetime_based_usage() {
        // Test basic environment creation and usage
        let mut global = Environment::new();
        global.define_str("global_var", Value::number(42.0));

        // Test scope creation with parent reference
        let mut function_env = Environment::new_scope(&global);
        function_env.define_str("local_var", Value::string("hello"));

        // Test nested scope
        let mut let_env = Environment::new_scope(&function_env);
        let_env.define_str("inner_var", Value::boolean(true));

        // Test lookups through environment chain
        assert!(
            let_env
                .lookup_str("inner_var")
                .unwrap()
                .as_boolean()
                .unwrap()
        );
        assert_eq!(
            let_env
                .lookup_str("local_var")
                .unwrap()
                .as_string()
                .unwrap(),
            "hello"
        );
        assert_eq!(
            let_env
                .lookup_str("global_var")
                .unwrap()
                .as_number()
                .unwrap(),
            42.0
        );

        // Test closure creation with efficient subset
        let_env.define_str("captured_var", Value::number(99.0));
        let_env.define_str("another_var", Value::symbol("test"));

        let keys = vec![Symbol::new("captured_var"), Symbol::new("another_var")];
        let closure_env = Environment::new_closure(&let_env, &keys);

        assert!(closure_env.parent().is_none()); // No parent - standalone
        assert_eq!(closure_env.len(), 2); // Only captured identifiers
        assert_eq!(
            closure_env
                .lookup_str("captured_var")
                .unwrap()
                .as_number()
                .unwrap(),
            99.0
        );
        assert_eq!(
            closure_env
                .lookup_str("another_var")
                .unwrap()
                .as_symbol()
                .unwrap(),
            "test"
        );

        // Test that closure doesn't have access to non-captured identifiers
        assert!(closure_env.lookup_str("global_var").is_err());
        assert!(closure_env.lookup_str("local_var").is_err());
        assert!(closure_env.lookup_str("inner_var").is_err());
    }

    #[test]
    fn test_environment_flatten() {
        // Create a chain of environments to test flattening
        let mut grandparent = Environment::new();
        grandparent.define_str("gp_var", Value::number(1.0));
        grandparent.define_str("shared", Value::string("grandparent"));

        let mut parent = Environment::new_scope(&grandparent);
        parent.define_str("parent_var", Value::number(2.0));
        parent.define_str("shared", Value::string("parent")); // Shadows grandparent

        let mut child = Environment::new_scope(&parent);
        child.define_str("child_var", Value::number(3.0));
        child.define_str("shared", Value::string("child")); // Shadows parent

        // Flatten the child environment
        let flattened = child.flatten();

        // Verify that all bindings are present
        assert_eq!(
            flattened.lookup_str("gp_var").unwrap().as_number().unwrap(),
            1.0
        );
        assert_eq!(
            flattened
                .lookup_str("parent_var")
                .unwrap()
                .as_number()
                .unwrap(),
            2.0
        );
        assert_eq!(
            flattened
                .lookup_str("child_var")
                .unwrap()
                .as_number()
                .unwrap(),
            3.0
        );

        // Verify that shadowing is preserved (innermost wins)
        assert_eq!(
            flattened.lookup_str("shared").unwrap().as_string().unwrap(),
            "child"
        );

        // Verify that the flattened environment has no parent
        assert!(flattened.parent().is_none());

        // Verify that the flattened environment has the correct number of bindings
        assert_eq!(flattened.len(), 4); // gp_var, parent_var, child_var, shared

        // Verify that the flattened environment contains all expected keys
        assert!(flattened.contains_str("gp_var"));
        assert!(flattened.contains_str("parent_var"));
        assert!(flattened.contains_str("child_var"));
        assert!(flattened.contains_str("shared"));

        // Verify that unbound identifiers still return errors
        assert!(flattened.lookup_str("nonexistent").is_err());
    }

    #[test]
    fn test_environment_flatten_empty_chain() {
        // Test flattening an empty environment
        let env = Environment::new();
        let flattened = env.flatten();

        assert!(flattened.is_empty());
        assert!(flattened.parent().is_none());
        assert_eq!(flattened.len(), 0);
    }

    #[test]
    fn test_environment_flatten_single_level() {
        // Test flattening a single-level environment (no parent)
        let mut env = Environment::new();
        env.define_str("var1", Value::number(42.0));
        env.define_str("var2", Value::string("test"));

        let flattened = env.flatten();

        assert_eq!(flattened.len(), 2);
        assert!(flattened.parent().is_none());
        assert_eq!(
            flattened.lookup_str("var1").unwrap().as_number().unwrap(),
            42.0
        );
        assert_eq!(
            flattened.lookup_str("var2").unwrap().as_string().unwrap(),
            "test"
        );
    }

    #[test]
    fn test_environment_flatten_preserves_shadowing_order() {
        // Test that flattening preserves correct shadowing behavior
        // where inner scopes override outer scopes
        let mut outer = Environment::new();
        outer.define_str("x", Value::number(1.0));
        outer.define_str("y", Value::number(10.0));

        let mut middle = Environment::new_scope(&outer);
        middle.define_str("x", Value::number(2.0)); // Shadows outer x
        middle.define_str("z", Value::number(20.0));

        let mut inner = Environment::new_scope(&middle);
        inner.define_str("x", Value::number(3.0)); // Shadows middle x
        inner.define_str("w", Value::number(30.0));

        let flattened = inner.flatten();

        // x should have the innermost value (3.0)
        assert_eq!(flattened.lookup_str("x").unwrap().as_number().unwrap(), 3.0);
        // y should come from outer scope
        assert_eq!(
            flattened.lookup_str("y").unwrap().as_number().unwrap(),
            10.0
        );
        // z should come from middle scope
        assert_eq!(
            flattened.lookup_str("z").unwrap().as_number().unwrap(),
            20.0
        );
        // w should come from inner scope
        assert_eq!(
            flattened.lookup_str("w").unwrap().as_number().unwrap(),
            30.0
        );

        assert_eq!(flattened.len(), 4);
        assert!(flattened.parent().is_none());
    }

    #[test]
    fn test_environment_flatten_with_complex_values() {
        // Test flattening with various value types
        let mut parent = Environment::new();
        parent.define_str("number", Value::number(42.0));
        parent.define_str("boolean", Value::boolean(true));
        parent.define_str("list", Value::List(crate::types::List::new()));

        let mut child = Environment::new_scope(&parent);
        child.define_str("string", Value::string("hello"));
        child.define_str("symbol", Value::symbol("test-symbol"));

        let flattened = child.flatten();

        assert_eq!(flattened.len(), 5);
        assert!(flattened.lookup_str("number").unwrap().is_number());
        assert!(flattened.lookup_str("boolean").unwrap().is_boolean());
        assert!(flattened.lookup_str("list").unwrap().is_list());
        assert!(flattened.lookup_str("string").unwrap().is_string());
        assert!(flattened.lookup_str("symbol").unwrap().is_symbol());
    }

    #[test]
    fn test_environment_flatten_static_lifetime() {
        // Test that flatten returns Environment<'static>
        let mut env = Environment::new();
        env.define_str("test", Value::number(123.0));

        let flattened = env.flatten();
        fn takes_static_env(_env: Environment<'static>) {}
        takes_static_env(flattened);
    }

    #[test]
    fn test_builtin_procedure_lookup() {
        let env = Environment::new();

        // Test arithmetic builtins
        let add_result = env.lookup_str("+").unwrap();
        assert!(add_result.is_procedure());
        let add_proc = add_result.as_procedure().unwrap();
        assert!(add_proc.is_builtin());
        assert_eq!(add_proc.name(), "+");

        let sub_result = env.lookup_str("-").unwrap();
        assert!(sub_result.is_procedure());
        let sub_proc = sub_result.as_procedure().unwrap();
        assert!(sub_proc.is_builtin());
        assert_eq!(sub_proc.name(), "-");

        let mul_result = env.lookup_str("*").unwrap();
        assert!(mul_result.is_procedure());
        let mul_proc = mul_result.as_procedure().unwrap();
        assert!(mul_proc.is_builtin());
        assert_eq!(mul_proc.name(), "*");

        let div_result = env.lookup_str("/").unwrap();
        assert!(div_result.is_procedure());
        let div_proc = div_result.as_procedure().unwrap();
        assert!(div_proc.is_builtin());
        assert_eq!(div_proc.name(), "/");

        // Test comparison builtins
        let eq_result = env.lookup_str("=").unwrap();
        assert!(eq_result.is_procedure());
        let eq_proc = eq_result.as_procedure().unwrap();
        assert!(eq_proc.is_builtin());
        assert_eq!(eq_proc.name(), "=");

        let lt_result = env.lookup_str("<").unwrap();
        assert!(lt_result.is_procedure());
        let lt_proc = lt_result.as_procedure().unwrap();
        assert!(lt_proc.is_builtin());
        assert_eq!(lt_proc.name(), "<");

        let gt_result = env.lookup_str(">").unwrap();
        assert!(gt_result.is_procedure());
        let gt_proc = gt_result.as_procedure().unwrap();
        assert!(gt_proc.is_builtin());
        assert_eq!(gt_proc.name(), ">");

        // Test list builtins
        let car_result = env.lookup_str("car").unwrap();
        assert!(car_result.is_procedure());
        let car_proc = car_result.as_procedure().unwrap();
        assert!(car_proc.is_builtin());
        assert_eq!(car_proc.name(), "car");

        let cdr_result = env.lookup_str("cdr").unwrap();
        assert!(cdr_result.is_procedure());
        let cdr_proc = cdr_result.as_procedure().unwrap();
        assert!(cdr_proc.is_builtin());
        assert_eq!(cdr_proc.name(), "cdr");

        let cons_result = env.lookup_str("cons").unwrap();
        assert!(cons_result.is_procedure());
        let cons_proc = cons_result.as_procedure().unwrap();
        assert!(cons_proc.is_builtin());
        assert_eq!(cons_proc.name(), "cons");

        let list_result = env.lookup_str("list").unwrap();
        assert!(list_result.is_procedure());
        let list_proc = list_result.as_procedure().unwrap();
        assert!(list_proc.is_builtin());
        assert_eq!(list_proc.name(), "list");

        let null_p_result = env.lookup_str("null?").unwrap();
        assert!(null_p_result.is_procedure());
        let null_p_proc = null_p_result.as_procedure().unwrap();
        assert!(null_p_proc.is_builtin());
        assert_eq!(null_p_proc.name(), "null?");
    }

    #[test]
    fn test_builtin_lookup_with_shadowing() {
        let mut env = Environment::new();

        // First verify builtin is accessible
        let add_result = env.lookup_str("+").unwrap();
        assert!(add_result.is_procedure());
        let add_proc = add_result.as_procedure().unwrap();
        assert!(add_proc.is_builtin());
        assert_eq!(add_proc.name(), "+");

        // Shadow the builtin with a user-defined binding
        env.define_str("+", Value::string("shadowed"));

        // Now lookup should return the user-defined value, not the builtin
        let shadowed_result = env.lookup_str("+").unwrap();
        assert!(shadowed_result.is_string());
        assert_eq!(shadowed_result.as_string().unwrap(), "shadowed");
        assert!(!shadowed_result.is_procedure());
    }

    #[test]
    fn test_builtin_lookup_in_parent_chain() {
        let parent = Environment::new();
        let mut child = Environment::new_scope(&parent);

        // Builtin should be accessible from child environment
        let add_result = child.lookup_str("+").unwrap();
        assert!(add_result.is_procedure());
        let add_proc = add_result.as_procedure().unwrap();
        assert!(add_proc.is_builtin());
        assert_eq!(add_proc.name(), "+");

        // Define something in child that doesn't shadow builtin
        child.define_str("x", Value::number(42.0));

        // Builtin should still be accessible
        let add_result2 = child.lookup_str("+").unwrap();
        assert!(add_result2.is_procedure());
        let add_proc2 = add_result2.as_procedure().unwrap();
        assert!(add_proc2.is_builtin());
        assert_eq!(add_proc2.name(), "+");

        // User-defined identifier should also be accessible
        let x_result = child.lookup_str("x").unwrap();
        assert!(x_result.is_number());
        assert_eq!(x_result.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_non_builtin_identifier_still_fails() {
        let env = Environment::new();

        // Non-existent identifier that's not a builtin should still fail
        let result = env.lookup_str("unknown-identifier");
        assert!(result.is_err());

        // Verify the error message is about unbound identifier
        let error = result.unwrap_err();
        let error_msg = format!("{error}");
        assert!(error_msg.contains("Unbound identifier"));
        assert!(error_msg.contains("unknown-identifier"));
    }

    #[test]
    fn test_builtin_lookup_precedence() {
        let mut parent = Environment::new();

        // Define + in parent
        parent.define_str("+", Value::string("parent-plus"));

        let mut child = Environment::new_scope(&parent);

        // Child should see parent's definition, not builtin
        let child_result = child.lookup_str("+").unwrap();
        assert!(child_result.is_string());
        assert_eq!(child_result.as_string().unwrap(), "parent-plus");

        // Define + in child
        child.define_str("+", Value::string("child-plus"));

        // Child should see its own definition
        let child_result2 = child.lookup_str("+").unwrap();
        assert!(child_result2.is_string());
        assert_eq!(child_result2.as_string().unwrap(), "child-plus");

        // Parent should still see its own definition
        let parent_result = parent.lookup_str("+").unwrap();
        assert!(parent_result.is_string());
        assert_eq!(parent_result.as_string().unwrap(), "parent-plus");

        // New environment should see builtin
        let fresh_env = Environment::new();
        let fresh_result = fresh_env.lookup_str("+").unwrap();
        assert!(fresh_result.is_procedure());
        let fresh_proc = fresh_result.as_procedure().unwrap();
        assert!(fresh_proc.is_builtin());
        assert_eq!(fresh_proc.name(), "+");
    }
}
