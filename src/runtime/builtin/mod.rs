//! Builtin procedures for the Twine Scheme runtime
//!
//! This module contains all built-in procedures organized by category.
//! These procedures are automatically available in the global environment.

pub mod arithmetic;

// Re-export arithmetic functions for convenience
pub use arithmetic::{
    add, divide, equal, greater_than, greater_than_or_equal, less_than, less_than_or_equal,
    multiply, subtract,
};
