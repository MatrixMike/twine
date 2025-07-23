//! Twine Scheme Interpreter
//!
//! A minimalist Scheme interpreter written in Rust that implements a functional
//! subset of R7RS-small Scheme with fiber-based concurrency and strict immutability.

pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod types;

// Re-export error types for convenience
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_deps_directory_structure() {
        use std::path::Path;

        // Verify deps directory structure exists
        assert!(Path::new("deps").exists(), "deps/ directory should exist");
        assert!(
            Path::new("deps/vendor").exists(),
            "deps/vendor/ directory should exist"
        );
        assert!(
            Path::new("deps/doc").exists(),
            "deps/doc/ directory should exist"
        );
    }

    #[test]
    fn test_gitignore_excludes_deps() {
        use std::fs;

        // Read .gitignore and verify it contains deps/
        let gitignore_content =
            fs::read_to_string(".gitignore").expect(".gitignore file should exist");

        assert!(
            gitignore_content.contains("/deps"),
            ".gitignore should exclude /deps directory"
        );
    }

    #[test]
    fn test_vendored_dependencies_exist() {
        use std::path::Path;

        // Verify that vendored dependencies exist
        assert!(
            Path::new("deps/vendor").exists(),
            "deps/vendor/ should exist"
        );

        // Check for at least one vendored dependency
        let vendor_dir = std::fs::read_dir("deps/vendor");
        if let Ok(entries) = vendor_dir {
            let has_dependencies = entries.count() > 0;
            assert!(
                has_dependencies,
                "deps/vendor/ should contain vendored dependencies"
            );
        }
    }

    #[test]
    fn test_generated_documentation_exists() {
        use std::path::Path;

        // Verify that generated documentation exists
        assert!(Path::new("deps/doc").exists(), "deps/doc/ should exist");

        // Check for generated documentation files
        let docs_dir = std::fs::read_dir("deps/doc");
        if let Ok(entries) = docs_dir {
            let has_docs = entries.count() > 0;
            assert!(has_docs, "deps/doc/ should contain generated documentation");
        }
    }

    #[test]
    fn test_parser_type_imports() {
        use crate::lexer::Position;
        use crate::parser::{Expression, PositionedExpression};
        use crate::types::Value;

        // Test that parser types work correctly when imported directly
        let expr = Expression::atom(Value::number(42.0));
        assert!(expr.is_atom());
        assert_eq!(expr.type_name(), "atom");

        let list = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]);
        assert!(list.is_list());
        assert_eq!(list.type_name(), "list");

        let quoted = Expression::quote(Expression::atom(Value::symbol("x")));
        assert!(quoted.is_quoted());
        assert_eq!(quoted.type_name(), "quote");

        // Test positioned expression
        let position = Position::new(1, 5);
        let positioned = PositionedExpression::new(expr, position);
        assert_eq!(positioned.position.line, 1);
        assert_eq!(positioned.position.column, 5);
    }

    #[test]
    fn test_lifetime_based_environment_usage() {
        use crate::interpreter::environment::Environment;
        use crate::types::{Symbol, Value};

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
        assert_eq!(
            let_env
                .lookup_str("inner_var")
                .unwrap()
                .as_boolean()
                .unwrap(),
            true
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
}
