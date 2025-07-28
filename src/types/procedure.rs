//! Function and procedure types for Scheme
//!
//! This module implements the procedure system for Scheme functions,
//! including built-in procedures and user-defined lambdas with closures.

use crate::parser::Expression;
use crate::runtime::{Environment, builtins::Builtin};
use crate::types::Symbol;
use std::sync::Arc;

/// Lambda procedure definition
///
/// Lambda procedures are created by the `lambda` special form and
/// capture their defining environment as a closure. This struct
/// enables efficient sharing via Arc.
#[derive(Debug)]
pub struct Lambda {
    /// Parameter identifiers for the procedure
    params: Vec<Symbol>,
    /// Expression that forms the procedure body
    body: Arc<Expression>,
    /// Captured environment (closure) from procedure definition
    env: Environment<'static>,
}

/// Procedure types in Scheme
///
/// Procedures represent callable entities in Scheme, including both built-in
/// procedures implemented in Rust and user-defined lambda procedures.
#[derive(Debug, Clone)]
pub enum Procedure {
    /// Built-in procedure implemented in Rust
    ///
    /// Built-in procedures are represented by a Builtin enum variant,
    /// eliminating the need to store redundant name and function pointer data.
    Builtin(Builtin),

    /// User-defined lambda procedure with closure
    ///
    /// Lambda procedures use Arc for efficient sharing and cloning.
    /// Multiple Values can reference the same lambda with zero copying.
    Lambda(Arc<Lambda>),
}

impl Lambda {
    /// Create a new Lambda instance wrapped in Arc
    ///
    /// # Arguments
    /// * `params` - Parameter identifiers for the procedure
    /// * `body` - Expression that forms the procedure body
    /// * `env` - Captured environment from procedure definition
    ///
    /// # Returns
    /// A new Arc<Lambda> for efficient sharing
    pub fn new(params: Vec<Symbol>, body: Arc<Expression>, env: Environment<'static>) -> Arc<Self> {
        Arc::new(Lambda { params, body, env })
    }

    /// Get a reference to the parameter identifiers
    pub fn params(&self) -> &[Symbol] {
        &self.params
    }

    /// Get a reference to the body expression
    pub fn body(&self) -> &Arc<Expression> {
        &self.body
    }

    /// Get a reference to the captured environment
    pub fn env(&self) -> &Environment<'static> {
        &self.env
    }

    /// Get the parameter count
    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

impl Procedure {
    /// Create a new built-in procedure
    ///
    /// # Arguments
    /// * `builtin` - The specific builtin procedure variant
    ///
    /// # Returns
    /// A new `Procedure::Builtin` instance
    pub fn builtin(builtin: Builtin) -> Self {
        Procedure::Builtin(builtin)
    }

    /// Create a new lambda procedure
    ///
    /// # Arguments
    /// * `params` - Parameter identifiers for the procedure
    /// * `body` - Expression that forms the procedure body
    /// * `env` - Captured environment from procedure definition
    ///
    /// # Returns
    /// A new `Procedure::Lambda` instance with Arc sharing
    pub fn lambda(params: Vec<Symbol>, body: Arc<Expression>, env: Environment<'static>) -> Self {
        Procedure::Lambda(Lambda::new(params, body, env))
    }

    /// Get the display name of the procedure
    ///
    /// For built-in procedures, returns the stored name.
    /// For lambda procedures, returns a generic lambda description.
    pub fn name(&self) -> &str {
        match self {
            Procedure::Builtin(builtin) => builtin.name(),
            Procedure::Lambda(_) => "<lambda>",
        }
    }

    /// Check if this is a built-in procedure
    pub fn is_builtin(&self) -> bool {
        matches!(self, Procedure::Builtin(_))
    }

    /// Check if this is a lambda procedure
    pub fn is_lambda(&self) -> bool {
        matches!(self, Procedure::Lambda(_))
    }

    /// Get the parameter count for the procedure
    ///
    /// Returns the number of parameters this procedure expects.
    /// For built-in procedures, this is not directly available, so None is returned.
    /// For lambda procedures, returns the parameter count.
    pub fn arity(&self) -> Option<usize> {
        match self {
            Procedure::Builtin(_) => None, // Arity varies for built-ins
            Procedure::Lambda(lambda) => Some(lambda.arity()),
        }
    }

    /// Get a reference to the parameters (lambda procedures only)
    pub fn params(&self) -> Option<&[Symbol]> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda.params()),
        }
    }

    /// Get a reference to the body expression (lambda procedures only)
    pub fn body(&self) -> Option<&Arc<Expression>> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda.body()),
        }
    }

    /// Get a reference to the captured environment (lambda procedures only)
    pub fn env(&self) -> Option<&Environment<'static>> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda.env()),
        }
    }

    /// Get a reference to the Lambda struct (lambda procedures only)
    pub fn as_lambda(&self) -> Option<&Arc<Lambda>> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda),
        }
    }
}

impl PartialEq for Lambda {
    fn eq(&self, other: &Self) -> bool {
        // Lambda procedures are equal if they have the same parameters and body
        // (environments are not easily comparable)
        self.params == other.params && format!("{}", self.body) == format!("{}", other.body)
    }
}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Built-in procedures are equal if they have the same kind
            (Procedure::Builtin(kind1), Procedure::Builtin(kind2)) => kind1 == kind2,

            // Lambda procedures are equal if their Lambda structs are equal
            (Procedure::Lambda(lambda1), Procedure::Lambda(lambda2)) => {
                lambda1.as_ref() == lambda2.as_ref()
            }

            // Different procedure types are never equal
            _ => false,
        }
    }
}

impl std::fmt::Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#<lambda:")?;
        for (i, param) in self.params.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{param}")?;
        }
        write!(f, ">")
    }
}

impl std::fmt::Display for Procedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Procedure::Builtin(builtin) => write!(f, "#<builtin:{}>", builtin.name()),
            Procedure::Lambda(lambda) => write!(f, "{lambda}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_lambda_struct_creation() {
        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();

        let lambda = Lambda::new(params.clone(), Arc::clone(&body), env);

        assert_eq!(lambda.params(), &params);
        assert_eq!(lambda.body(), &body);
        assert_eq!(lambda.arity(), 2);
        // Environment is captured correctly
        assert!(lambda.env().lookup(&Symbol::new("nonexistent")).is_err());
    }

    #[test]
    fn test_lambda_struct_equality() {
        let params = vec![Symbol::new("x")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env1 = Environment::new();
        let env2 = Environment::new();

        let lambda1 = Lambda::new(params.clone(), Arc::clone(&body), env1);
        let lambda2 = Lambda::new(params.clone(), Arc::clone(&body), env2);

        // Lambda structs with same params and body are equal (despite different envs)
        assert_eq!(lambda1.as_ref(), lambda2.as_ref());

        let different_params = vec![Symbol::new("y")];
        let env3 = Environment::new();
        let lambda3 = Lambda::new(different_params, Arc::clone(&body), env3);

        assert_ne!(lambda1.as_ref(), lambda3.as_ref());
    }

    #[test]
    fn test_lambda_display() {
        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();
        let lambda = Lambda::new(params, body, env);

        assert_eq!(format!("{lambda}"), "#<lambda:x y>");

        // Test lambda with no parameters
        let no_params = vec![];
        let body = Expression::arc_atom(Value::number(42.0));
        let env = Environment::new();
        let lambda_no_params = Lambda::new(no_params, body, env);
        assert_eq!(format!("{lambda_no_params}"), "#<lambda:>");
    }

    #[test]
    fn test_builtin_procedure_creation() {
        let proc = Procedure::builtin(Builtin::Add);

        assert!(proc.is_builtin());
        assert!(!proc.is_lambda());
        assert_eq!(proc.name(), "+");
        assert_eq!(proc.arity(), None); // Built-ins don't report arity
    }

    #[test]
    fn test_lambda_procedure_creation() {
        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();

        let proc = Procedure::lambda(params.clone(), body, env);

        assert!(proc.is_lambda());
        assert!(!proc.is_builtin());
        assert_eq!(proc.name(), "<lambda>");
        assert_eq!(proc.arity(), Some(2));
        assert_eq!(proc.params().unwrap(), &params);
        assert!(proc.body().is_some());
        assert!(proc.env().is_some());
        assert!(proc.as_lambda().is_some());
    }

    #[test]
    fn test_procedure_accessors() {
        // Test builtin accessors
        let builtin = Procedure::builtin(Builtin::Add);
        assert!(builtin.params().is_none());
        assert!(builtin.body().is_none());
        assert!(builtin.env().is_none());
        assert!(builtin.as_lambda().is_none());

        // Test lambda accessors
        let params = vec![Symbol::new("x")];
        let body = Expression::arc_atom(Value::number(42.0));
        let env = Environment::new();
        let lambda = Procedure::lambda(params.clone(), body, env);

        assert_eq!(lambda.params().unwrap(), &params);
        assert!(lambda.body().is_some());
        assert!(lambda.env().is_some());
        assert!(lambda.as_lambda().is_some());
    }

    #[test]
    fn test_arc_sharing() {
        // Test that Arc sharing works correctly
        let params = vec![Symbol::new("x")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();

        let proc1 = Procedure::lambda(params.clone(), Arc::clone(&body), env.flatten());
        let proc2 = proc1.clone();

        // Verify they share the same Arc
        if let (Procedure::Lambda(arc1), Procedure::Lambda(arc2)) = (&proc1, &proc2) {
            assert!(Arc::ptr_eq(arc1, arc2));
        } else {
            panic!("Expected lambda procedures");
        }

        // Create a new procedure with same content
        let env2 = Environment::new();
        let proc3 = Procedure::lambda(params, body, env2);

        // Should be equal but not share the same Arc
        if let (Procedure::Lambda(arc1), Procedure::Lambda(arc3)) = (&proc1, &proc3) {
            assert!(!Arc::ptr_eq(arc1, arc3));
            assert_eq!(arc1.as_ref(), arc3.as_ref()); // Content equality
        } else {
            panic!("Expected lambda procedures");
        }
    }

    #[test]
    fn test_procedure_equality() {
        // Same kind built-ins are equal
        let builtin1 = Procedure::builtin(Builtin::Add);
        let builtin2 = Procedure::builtin(Builtin::Add);
        assert_eq!(builtin1, builtin2);

        // Different kind built-ins are not equal
        let builtin3 = Procedure::builtin(Builtin::Subtract);
        assert_ne!(builtin1, builtin3);

        // Same lambda procedures are equal
        let params = vec![Symbol::new("x")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env1 = Environment::new();
        let env2 = Environment::new();

        let lambda1 = Procedure::lambda(params.clone(), Arc::clone(&body), env1);
        let _lambda2 = Procedure::lambda(params.clone(), Arc::clone(&body), env2);

        let different_params = vec![Symbol::new("y")];
        let env3 = Environment::new();
        let lambda3 = Procedure::lambda(different_params, Arc::clone(&body), env3);
        assert_ne!(lambda1, lambda3);

        // Built-in and lambda are never equal
        assert_ne!(builtin1, lambda1);
    }

    #[test]
    fn test_procedure_display() {
        let builtin = Procedure::builtin(Builtin::Add);
        assert_eq!(format!("{builtin}"), "#<builtin:+>");

        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();
        let lambda = Procedure::lambda(params, body, env);
        assert_eq!(format!("{lambda}"), "#<lambda:x y>");

        // Test lambda with no parameters
        let no_params = vec![];
        let body = Expression::arc_atom(Value::number(42.0));
        let env = Environment::new();

        let lambda_no_params = Procedure::lambda(no_params, body, env);
        assert_eq!(format!("{lambda_no_params}"), "#<lambda:>");
    }

    #[test]
    fn test_procedure_debug_output() {
        let builtin = Procedure::builtin(Builtin::Add);
        let debug_output = format!("{builtin:?}");
        assert!(debug_output.contains("Builtin"));
        assert!(debug_output.contains("Add"));

        let params = vec![Symbol::new("x")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();
        let lambda = Procedure::lambda(params, body, env);
        let debug_output = format!("{lambda:?}");
        assert!(debug_output.contains("Lambda"));
    }

    #[test]
    fn test_procedure_clone_efficiency() {
        // Test that cloning is efficient (Arc-based)
        let params = vec![Symbol::new("x")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();
        let original = Procedure::lambda(params, body, env);

        // Clone should be very fast (just Arc clone)
        let cloned = original.clone();

        // Verify they're equal and share the same Arc
        assert_eq!(original, cloned);
        if let (Procedure::Lambda(arc1), Procedure::Lambda(arc2)) = (&original, &cloned) {
            assert!(Arc::ptr_eq(arc1, arc2));
        }
    }

    #[test]
    fn test_lambda_arc_strong_count() {
        let params = vec![Symbol::new("x")];
        let body = Expression::arc_atom(Value::symbol("x"));
        let env = Environment::new();

        let proc1 = Procedure::lambda(params, body, env);

        if let Procedure::Lambda(arc) = &proc1 {
            assert_eq!(Arc::strong_count(arc), 1);
        }

        let proc2 = proc1.clone();

        if let Procedure::Lambda(arc) = &proc1 {
            assert_eq!(Arc::strong_count(arc), 2);
        }

        drop(proc2);

        if let Procedure::Lambda(arc) = &proc1 {
            assert_eq!(Arc::strong_count(arc), 1);
        }
    }

    // Tests for the Builtin enum are now in the builtins module
}
