//! Function and procedure types for Scheme
//!
//! This module implements the procedure system for Scheme functions,
//! including built-in procedures and user-defined lambdas with closures.

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::Environment;
use crate::types::{Symbol, Value};
use smol_str::SmolStr;

/// Function signature for built-in procedures
///
/// Built-in procedures take a slice of evaluated arguments and return a Result.
/// This enables proper error handling and integration with the evaluation system.
pub type BuiltinFn = fn(&[Value]) -> Result<Value>;

/// Procedure types in Scheme
///
/// Procedures represent callable entities in Scheme, including both built-in
/// procedures implemented in Rust and user-defined lambda procedures.
#[derive(Debug, Clone)]
pub enum Procedure {
    /// Built-in procedure implemented in Rust
    ///
    /// Built-in procedures are native Rust functions that implement
    /// core Scheme functionality like arithmetic, list operations, and I/O.
    Builtin {
        /// Display name of the procedure for error messages and debugging
        name: SmolStr,
        /// Rust function implementing the procedure
        function: BuiltinFn,
    },

    /// User-defined lambda procedure with closure
    ///
    /// Lambda procedures are created by the `lambda` special form and
    /// capture their defining environment as a closure.
    Lambda {
        /// Parameter identifiers for the procedure
        params: Vec<Symbol>,
        /// Expression that forms the procedure body
        /// Uses Box because recursive enum variants would have infinite size.
        /// Box provides heap allocation to break the recursion.
        body: Box<Expression>,
        /// Captured environment (closure) from procedure definition
        env: Environment<'static>,
    },
}

impl Procedure {
    /// Create a new built-in procedure
    ///
    /// # Arguments
    /// * `name` - Display name for the procedure
    /// * `function` - Rust function implementing the procedure
    ///
    /// # Returns
    /// A new `Procedure::Builtin` instance
    pub fn builtin(name: &str, function: BuiltinFn) -> Self {
        Procedure::Builtin {
            name: SmolStr::new(name),
            function,
        }
    }

    /// Create a new lambda procedure
    ///
    /// # Arguments
    /// * `params` - Parameter identifiers for the procedure
    /// * `body` - Expression that forms the procedure body
    /// * `env` - Captured environment from procedure definition
    ///
    /// # Returns
    /// A new `Procedure::Lambda` instance
    pub fn lambda(params: Vec<Symbol>, body: Expression, env: Environment<'static>) -> Self {
        Procedure::Lambda {
            params,
            body: Box::new(body),
            env,
        }
    }

    /// Get the display name of the procedure
    ///
    /// For built-in procedures, returns the stored name.
    /// For lambda procedures, returns a generic lambda description.
    pub fn name(&self) -> &str {
        match self {
            Procedure::Builtin { name, .. } => name.as_str(),
            Procedure::Lambda { .. } => "<lambda>",
        }
    }

    /// Check if this is a built-in procedure
    pub fn is_builtin(&self) -> bool {
        matches!(self, Procedure::Builtin { .. })
    }

    /// Check if this is a lambda procedure
    pub fn is_lambda(&self) -> bool {
        matches!(self, Procedure::Lambda { .. })
    }

    /// Get the parameter count for the procedure
    ///
    /// Returns the number of parameters this procedure expects.
    /// For built-in procedures, this is not directly available, so None is returned.
    /// For lambda procedures, returns the parameter count.
    pub fn arity(&self) -> Option<usize> {
        match self {
            Procedure::Builtin { .. } => None, // Arity varies for built-ins
            Procedure::Lambda { params, .. } => Some(params.len()),
        }
    }

    /// Get a reference to the parameters (lambda procedures only)
    pub fn params(&self) -> Option<&[Symbol]> {
        match self {
            Procedure::Builtin { .. } => None,
            Procedure::Lambda { params, .. } => Some(params),
        }
    }

    /// Get a reference to the body expression (lambda procedures only)
    pub fn body(&self) -> Option<&Expression> {
        match self {
            Procedure::Builtin { .. } => None,
            Procedure::Lambda { body, .. } => Some(body.as_ref()),
        }
    }

    /// Get a reference to the captured environment (lambda procedures only)
    pub fn env(&self) -> Option<&Environment<'static>> {
        match self {
            Procedure::Builtin { .. } => None,
            Procedure::Lambda { env, .. } => Some(env),
        }
    }
}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Built-in procedures are equal if they have the same name
            // (function pointers are not easily comparable)
            (Procedure::Builtin { name: name1, .. }, Procedure::Builtin { name: name2, .. }) => {
                name1 == name2
            }

            // Lambda procedures are equal if they have the same parameters and body
            // (environments are not easily comparable)
            (
                Procedure::Lambda {
                    params: params1,
                    body: body1,
                    ..
                },
                Procedure::Lambda {
                    params: params2,
                    body: body2,
                    ..
                },
            ) => {
                params1 == params2 && format!("{}", body1.as_ref()) == format!("{}", body2.as_ref())
            }

            // Different procedure types are never equal
            _ => false,
        }
    }
}

impl std::fmt::Display for Procedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Procedure::Builtin { name, .. } => write!(f, "#<builtin:{}>", name),
            Procedure::Lambda { params, .. } => {
                write!(f, "#<lambda:")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ">")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    // Sample builtin function for testing
    fn sample_builtin(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(crate::Error::runtime_error("Expected 1 argument"));
        }
        Ok(args[0].clone())
    }

    #[test]
    fn test_builtin_procedure_creation() {
        let proc = Procedure::builtin("identity", sample_builtin);

        assert!(proc.is_builtin());
        assert!(!proc.is_lambda());
        assert_eq!(proc.name(), "identity");
        assert_eq!(proc.arity(), None); // Built-ins don't report arity
    }

    #[test]
    fn test_lambda_procedure_creation() {
        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = Expression::atom(Value::symbol("x"));
        let env = Environment::new();

        let proc = Procedure::lambda(params.clone(), body, env);

        assert!(proc.is_lambda());
        assert!(!proc.is_builtin());
        assert_eq!(proc.name(), "<lambda>");
        assert_eq!(proc.arity(), Some(2));
        assert_eq!(proc.params().unwrap(), &params);
        assert!(proc.body().is_some());
        assert!(proc.env().is_some());
    }

    #[test]
    fn test_procedure_accessors() {
        // Test builtin accessors
        let builtin = Procedure::builtin("test", sample_builtin);
        assert!(builtin.params().is_none());
        assert!(builtin.body().is_none());
        assert!(builtin.env().is_none());

        // Test lambda accessors
        let params = vec![Symbol::new("x")];
        let body = Expression::atom(Value::number(42.0));
        let env = Environment::new();
        let lambda = Procedure::lambda(params.clone(), body, env);

        assert_eq!(lambda.params().unwrap(), &params);
        assert!(lambda.body().is_some());
        assert!(lambda.env().is_some());
    }

    #[test]
    fn test_procedure_equality() {
        // Same name built-ins are equal
        let builtin1 = Procedure::builtin("add", sample_builtin);
        let builtin2 = Procedure::builtin("add", sample_builtin);
        assert_eq!(builtin1, builtin2);

        // Different name built-ins are not equal
        let builtin3 = Procedure::builtin("sub", sample_builtin);
        assert_ne!(builtin1, builtin3);

        // Same lambda procedures are equal
        let params = vec![Symbol::new("x")];
        let body = Expression::atom(Value::symbol("x"));
        let env1 = Environment::new();
        let env2 = Environment::new();
        let lambda1 = Procedure::lambda(params.clone(), body.clone(), env1);
        let lambda2 = Procedure::lambda(params.clone(), body.clone(), env2);
        assert_eq!(lambda1, lambda2); // Equal despite different environments

        // Different parameter lambda procedures are not equal
        let different_params = vec![Symbol::new("y")];
        let env3 = Environment::new();
        let lambda3 = Procedure::lambda(different_params, body.clone(), env3);
        assert_ne!(lambda1, lambda3);

        // Built-in and lambda are never equal
        assert_ne!(builtin1, lambda1);
    }

    #[test]
    fn test_procedure_display() {
        let builtin = Procedure::builtin("add", sample_builtin);
        assert_eq!(format!("{}", builtin), "#<builtin:add>");

        let params = vec![Symbol::new("x"), Symbol::new("y")];
        let body = Expression::atom(Value::symbol("x"));
        let env = Environment::new();
        let lambda = Procedure::lambda(params, body, env);
        assert_eq!(format!("{}", lambda), "#<lambda:x y>");

        // Test lambda with no parameters
        let no_params = vec![];
        let body = Expression::atom(Value::number(42.0));
        let env = Environment::new();
        let lambda_no_params = Procedure::lambda(no_params, body, env);
        assert_eq!(format!("{}", lambda_no_params), "#<lambda:>");
    }

    #[test]
    fn test_procedure_debug_output() {
        let builtin = Procedure::builtin("test", sample_builtin);
        let debug_output = format!("{:?}", builtin);
        assert!(debug_output.contains("Builtin"));
        assert!(debug_output.contains("test"));

        let params = vec![Symbol::new("x")];
        let body = Expression::atom(Value::symbol("x"));
        let env = Environment::new();
        let lambda = Procedure::lambda(params, body, env);
        let debug_output = format!("{:?}", lambda);
        assert!(debug_output.contains("Lambda"));
        assert!(debug_output.contains("params"));
        assert!(debug_output.contains("body"));
        assert!(debug_output.contains("env"));
    }

    #[test]
    fn test_procedure_clone() {
        let builtin = Procedure::builtin("test", sample_builtin);
        let builtin_clone = builtin.clone();
        assert_eq!(builtin, builtin_clone);

        let params = vec![Symbol::new("x")];
        let body = Expression::atom(Value::symbol("x"));
        let env = Environment::new();
        let lambda = Procedure::lambda(params, body, env);
        let lambda_clone = lambda.clone();
        assert_eq!(lambda, lambda_clone);
    }
}
