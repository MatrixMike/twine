//! Interpreter module for the Twine Scheme interpreter
//!
//! This module contains the core interpreter functionality including
//! environment management, evaluation engine, and built-in procedures.

pub mod environment;
pub mod eval;

// Re-export key types for convenience
pub use environment::Environment;
pub use eval::eval;
