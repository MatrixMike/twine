//! Type system for Scheme values
//!
//! This module provides the core data types used throughout the Twine Scheme
//! interpreter. All types are designed to be immutable and thread-safe for
//! efficient sharing across the multi-threaded fiber runtime.
//!
//! ## Performance Optimizations
//!
//! - **Symbols**: Use `SmolStr` for stack allocation of short identifiers (≤23 bytes)
//! - **Strings/Lists**: Use `Arc` for efficient sharing across threads
//! - **Numbers**: Use primitive `f64` with `Copy` semantics

pub mod list;
pub mod number;
pub mod procedure;
pub mod string;
pub mod symbol;
pub mod value;

// Re-export core types for convenience
pub use list::List;
pub use number::Number;
pub use string::ArcString;
pub use symbol::Symbol;
pub use value::Value;

// Re-export SmolStr for convenience when working with symbols
pub use smol_str::SmolStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_organization() {
        // Test that all types are properly re-exported
        let _num: Number = Number::new(42.0);
        let _string: ArcString = ArcString::new("test");
        let _symbol: Symbol = Symbol::new("test");
        let _list: List = List::new();
        let _value: Value = Value::nil();

        // Test that types work together
        let value = Value::list(vec![
            Value::number(1.0),
            Value::string("hello"),
            Value::symbol("world"),
        ]);
        assert!(value.is_list());
    }

    #[test]
    fn test_type_imports() {
        // Verify all types are accessible through the module
        assert_eq!(
            std::any::type_name::<Number>(),
            "twine_scheme::types::number::Number"
        );
        assert_eq!(
            std::any::type_name::<ArcString>(),
            "twine_scheme::types::string::ArcString"
        );
        assert_eq!(
            std::any::type_name::<Symbol>(),
            "twine_scheme::types::symbol::Symbol"
        );
        assert_eq!(
            std::any::type_name::<List>(),
            "twine_scheme::types::list::List"
        );
        assert_eq!(
            std::any::type_name::<Value>(),
            "twine_scheme::types::value::Value"
        );
    }
}
