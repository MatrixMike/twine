//! Runtime module for the Twine Scheme interpreter
//!
//! This module contains the core runtime functionality organized as follows:
//!
//! - `environment`: Variable binding and scope management
//! - `eval`: Core evaluation engine and expression dispatch
//! - `special_forms`: Language constructs with special evaluation rules
//! - `builtins`: Standard library procedures organized by category
//!
//! ## Special Forms
//! Special forms have unique evaluation rules that differ from normal procedure calls.
//! Arguments are not automatically evaluated, allowing control over evaluation flow.
//!
//! ### Current Special Forms
//! - `if`: Conditional expressions
//!
//! ### Future Special Forms (planned)
//! - `define`: Variable and function definition (T2.4.1)
//! - `let`: Local variable binding (T2.4.2)
//! - `lambda`: Function definition (T3.1.2)
//! - `quote`: Prevent evaluation (already partially implemented)
//! - `define-syntax`: Macro definition (T5.2.1)
//!
//! ## Builtin Procedures
//! Builtin procedures are standard library functions automatically available in the global environment.
//! Arguments are evaluated before being passed to the procedure.
//!
//! ### Current Builtin Categories
//! - `arithmetic`: Numeric operations (+, -, *, /, =, <, >, <=, >=)
//!
//! ### Future Builtin Categories (planned)
//! - `list`: List operations (car, cdr, cons, list, null?, append) (T2.3.4, T3.2.2)
//! - `predicates`: Type checking procedures (number?, boolean?, string?, list?, null?) (T3.2.1)
//! - `io`: Input/output procedures (read, write, display) (T3.2.3)
//! - `control`: Control flow procedures (call/cc, apply) (future)

pub mod builtins;
pub mod environment;
pub mod eval;
pub mod special_forms;

// Re-export key types for convenience
pub use environment::Environment;
pub use eval::eval;
