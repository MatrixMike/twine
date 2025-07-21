//! Type system for Scheme values
//!
//! This module provides the core data types used throughout the Twine Scheme
//! interpreter. All types are designed to be immutable and efficiently shareable.

pub mod list;
pub mod number;
pub mod procedure;
pub mod string;
pub mod symbol;
pub mod value;

// Re-export core types for convenience
pub use list::List;
pub use number::Number;
pub use string::String;
pub use symbol::Symbol;
pub use value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_organization() {
        // Test that all types are properly re-exported
        let _num: Number = Number::new(42.0);
        let _string: String = String::new("test");
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
            std::any::type_name::<String>(),
            "twine_scheme::types::string::String"
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
