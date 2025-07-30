//! Function and procedure types for Scheme
//!
//! This module implements the procedure system for Scheme functions,
//! including built-in procedures and user-defined lambdas with closures.

use crate::parser::Expression;
use crate::runtime::{Environment, builtins::Builtin};
use crate::types::Symbol;
use std::sync::{Arc, OnceLock, Weak};

/// Lambda procedure definition
///
/// Lambda procedures are created by the `lambda` special form and
/// capture their defining environment as a closure. This struct
/// enables efficient sharing via Arc.
#[derive(Debug)]
pub struct Lambda {
    /// Parameter identifiers for the procedure
    params: Vec<Symbol>,
    /// Expressions that form the procedure body
    body: Vec<Arc<Expression>>,
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

    /// Weak reference to a lambda procedure for recursive definitions
    ///
    /// Used as a placeholder during recursive procedure definition (define and letrec).
    /// The OnceLock ensures thread-safe, one-time initialization of the weak reference.
    /// This enables recursive and mutually recursive procedure definitions without
    /// circular reference issues.
    WeakLambda(Arc<OnceLock<Weak<Lambda>>>),
}

impl Lambda {
    /// Create a new Lambda instance wrapped in Arc
    ///
    /// # Arguments
    /// * `params` - Parameter identifiers for the procedure
    /// * `body` - Expressions that form the procedure body
    /// * `env` - Captured environment from procedure definition
    ///
    /// # Returns
    /// A new Arc<Lambda> for efficient sharing
    pub fn new(
        params: Vec<Symbol>,
        body: Vec<Arc<Expression>>,
        env: Environment<'static>,
    ) -> Arc<Self> {
        Arc::new(Lambda { params, body, env })
    }

    /// Get a reference to the parameter identifiers
    pub fn params(&self) -> &[Symbol] {
        &self.params
    }

    /// Get a reference to the body expressions
    pub fn body(&self) -> &[Arc<Expression>] {
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
    /// * `body` - Expressions that form the procedure body
    /// * `env` - Captured environment from procedure definition
    ///
    /// # Returns
    /// A new `Procedure::Lambda` instance with Arc sharing
    pub fn lambda(
        params: Vec<Symbol>,
        body: Vec<Arc<Expression>>,
        env: Environment<'static>,
    ) -> Self {
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
            Procedure::WeakLambda(_) => "<lambda>",
        }
    }

    /// Check if this is a built-in procedure
    pub fn is_builtin(&self) -> bool {
        matches!(self, Procedure::Builtin(_))
    }

    /// Check if this is a lambda procedure (including weak lambda)
    pub fn is_lambda(&self) -> bool {
        matches!(self, Procedure::Lambda(_) | Procedure::WeakLambda(_))
    }

    /// Get the parameter count for the procedure
    ///
    /// Returns the number of parameters this procedure expects.
    /// For built-in procedures, this is not directly available, so None is returned.
    /// For lambda procedures, returns the parameter count.
    /// For weak lambda procedures, returns None if not yet initialized.
    pub fn arity(&self) -> Option<usize> {
        match self {
            Procedure::Builtin(_) => None, // Arity varies for built-ins
            Procedure::Lambda(lambda) => Some(lambda.arity()),
            Procedure::WeakLambda(once_lock) => once_lock
                .get()
                .and_then(|weak| weak.upgrade())
                .map(|lambda| lambda.arity()),
        }
    }

    /// Get a reference to the parameters (lambda procedures only)
    pub fn params(&self) -> Option<&[Symbol]> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda.params()),
            Procedure::WeakLambda(_) => None, // Cannot access params through weak reference
        }
    }

    /// Get a reference to the body expressions (lambda procedures only)
    pub fn body(&self) -> Option<&[Arc<Expression>]> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda.body()),
            Procedure::WeakLambda(_) => None, // Cannot access body through weak reference
        }
    }

    /// Get a reference to the captured environment (lambda procedures only)
    pub fn env(&self) -> Option<&Environment<'static>> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda.env()),
            Procedure::WeakLambda(_) => None, // Cannot access env through weak reference
        }
    }

    /// Get a reference to the Lambda struct (lambda procedures only)
    pub fn as_lambda(&self) -> Option<&Arc<Lambda>> {
        match self {
            Procedure::Builtin(_) => None,
            Procedure::Lambda(lambda) => Some(lambda),
            Procedure::WeakLambda(_) => None, // Cannot return Arc through weak reference
        }
    }

    /// Create a new weak lambda procedure placeholder
    ///
    /// Creates an uninitialized WeakLambda that can later be set to point
    /// to an actual lambda. Used for recursive procedure definitions.
    pub fn weak_lambda() -> Self {
        Procedure::WeakLambda(Arc::new(OnceLock::new()))
    }

    /// Initialize a WeakLambda with the actual lambda
    ///
    /// This can only be called once per WeakLambda instance.
    /// Returns an error if the WeakLambda was already initialized.
    pub fn set_weak_lambda(&self, lambda: &Arc<Lambda>) -> Result<(), crate::Error> {
        match self {
            Procedure::WeakLambda(once_lock) => once_lock
                .set(Arc::downgrade(lambda))
                .map_err(|_| crate::Error::runtime_error("WeakLambda already initialized")),
            _ => Err(crate::Error::runtime_error(
                "Cannot set weak lambda on non-WeakLambda procedure",
            )),
        }
    }

    /// Get the actual lambda from a WeakLambda
    ///
    /// Returns the upgraded Arc<Lambda> if the WeakLambda is initialized
    /// and the lambda hasn't been dropped. Used during procedure application.
    pub fn resolve_weak_lambda(&self) -> Result<Arc<Lambda>, crate::Error> {
        match self {
            Procedure::WeakLambda(once_lock) => {
                let weak = once_lock
                    .get()
                    .ok_or_else(|| crate::Error::runtime_error("WeakLambda not yet initialized"))?;
                weak.upgrade()
                    .ok_or_else(|| crate::Error::runtime_error("Lambda was dropped"))
            }
            Procedure::Lambda(lambda) => Ok(Arc::clone(lambda)),
            Procedure::Builtin(_) => Err(crate::Error::runtime_error(
                "Cannot resolve lambda from builtin procedure",
            )),
        }
    }
}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Built-in procedures are equal if they have the same kind
            (Procedure::Builtin(kind1), Procedure::Builtin(kind2)) => kind1 == kind2,

            // Lambda procedures are equal if both Arc<Lambda> point to the same Lambda.
            // This uses pointer equality rather than content equality - two lambdas
            // are only equal if they're the exact same Arc instance (e.g., via cloning).
            // Lambdas with identical content but different Arc instances are NOT equal.
            (Procedure::Lambda(lambda1), Procedure::Lambda(lambda2)) => {
                Arc::ptr_eq(lambda1, lambda2)
            }

            // WeakLambda procedures are equal if they point to the same lambda
            (Procedure::WeakLambda(once_lock1), Procedure::WeakLambda(once_lock2)) => {
                match (once_lock1.get(), once_lock2.get()) {
                    (Some(weak1), Some(weak2)) => {
                        match (weak1.upgrade(), weak2.upgrade()) {
                            (Some(lambda1), Some(lambda2)) => Arc::ptr_eq(&lambda1, &lambda2),
                            _ => false, // One or both lambdas were dropped
                        }
                    }
                    (None, None) => false, // Both uninitialized, not equal
                    _ => false,            // One initialized, one not
                }
            }

            // Lambda and WeakLambda can be equal if they point to the same lambda
            (Procedure::Lambda(lambda1), Procedure::WeakLambda(once_lock)) => once_lock
                .get()
                .and_then(|weak| weak.upgrade())
                .map(|lambda2| Arc::ptr_eq(lambda1, &lambda2))
                .unwrap_or(false),

            (Procedure::WeakLambda(once_lock), Procedure::Lambda(lambda2)) => once_lock
                .get()
                .and_then(|weak| weak.upgrade())
                .map(|lambda1| Arc::ptr_eq(&lambda1, lambda2))
                .unwrap_or(false),

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
            Procedure::WeakLambda(once_lock) => {
                match once_lock.get().and_then(|weak| weak.upgrade()) {
                    Some(lambda) => write!(f, "{lambda}"),
                    None => write!(f, "#<weak-lambda:uninitialized>"),
                }
            }
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
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
        let env = Environment::new();

        let lambda = Lambda::new(params.clone(), body.clone(), env);

        assert_eq!(lambda.params(), &params);
        assert_eq!(lambda.body(), &body);
        assert_eq!(lambda.arity(), 2);
        // Environment is captured correctly
        assert!(lambda.env().lookup(&Symbol::new("nonexistent")).is_err());
    }

    #[test]
    fn test_lambda_display() {
        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
        let env = Environment::new();
        let lambda = Lambda::new(params, body, env);

        assert_eq!(format!("{lambda}"), "#<lambda:x y>");

        // Test lambda with no parameters
        let no_params = vec![];
        let body = vec![Expression::arc_atom(Value::number(42.0))];
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
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
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
        let body = vec![Expression::arc_atom(Value::number(42.0))];
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
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
        let env = Environment::new();

        let proc1 = Procedure::lambda(params.clone(), body.clone(), env.flatten());
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

        // Should NOT be equal since they don't share the same Arc
        if let (Procedure::Lambda(arc1), Procedure::Lambda(arc3)) = (&proc1, &proc3) {
            assert!(!Arc::ptr_eq(arc1, arc3));
            // With pointer equality, different Arc instances are not equal
            assert_ne!(proc1, proc3);
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

        // Test lambda procedure Arc pointer equality
        let params = vec![Symbol::new("x")];
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
        let env1 = Environment::new();
        let env2 = Environment::new();

        let lambda1 = Procedure::lambda(params.clone(), body.clone(), env1);
        let lambda2 = Procedure::lambda(params.clone(), body.clone(), env2);

        // Lambdas with same content but different Arc instances are NOT equal
        assert_ne!(lambda1, lambda2);

        // Cloned lambda shares the same Arc, so they ARE equal
        let lambda1_clone = lambda1.clone();
        assert_eq!(lambda1, lambda1_clone);

        let different_params = vec![Symbol::new("y")];
        let env3 = Environment::new();
        let lambda3 = Procedure::lambda(different_params, body.clone(), env3);
        assert_ne!(lambda1, lambda3);

        // Built-in and lambda are never equal
        assert_ne!(builtin1, lambda1);
    }

    #[test]
    fn test_procedure_display() {
        let builtin = Procedure::builtin(Builtin::Add);
        assert_eq!(format!("{builtin}"), "#<builtin:+>");

        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
        let env = Environment::new();
        let lambda = Procedure::lambda(params, body, env);
        assert_eq!(format!("{lambda}"), "#<lambda:x y>");

        // Test lambda with no parameters
        let no_params = vec![];
        let body = vec![Expression::arc_atom(Value::number(42.0))];
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
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
        let env = Environment::new();
        let lambda = Procedure::lambda(params, body, env);
        let debug_output = format!("{lambda:?}");
        assert!(debug_output.contains("Lambda"));
    }

    #[test]
    fn test_procedure_clone_efficiency() {
        // Test that cloning is efficient (Arc-based)
        let params = vec![Symbol::new("x")];
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
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
        let body = vec![Expression::arc_atom(Value::symbol("x"))];
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
