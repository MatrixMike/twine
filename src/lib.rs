//! Twine Scheme Interpreter
//!
//! A minimalist Scheme interpreter written in Rust that implements a functional
//! subset of R7RS-small Scheme with fiber-based concurrency and strict immutability.

pub mod error;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod runtime;
pub mod types;

// Re-export error types for convenience
pub use error::{Error, Result};
