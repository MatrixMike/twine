//! Twine Scheme Interpreter
//!
//! A minimalist Scheme interpreter written in Rust that implements a functional
//! subset of R7RS-small Scheme with fiber-based concurrency and strict immutability.

pub mod error;
pub mod lexer;
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
            Path::new("deps/docs").exists(),
            "deps/docs/ directory should exist"
        );
        assert!(
            Path::new("deps/registry").exists(),
            "deps/registry/ directory should exist"
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

        // Check for at least one vendored dependency (thiserror)
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
        assert!(Path::new("deps/docs").exists(), "deps/docs/ should exist");

        // Check for generated documentation files
        let docs_dir = std::fs::read_dir("deps/docs");
        if let Ok(entries) = docs_dir {
            let has_docs = entries.count() > 0;
            assert!(
                has_docs,
                "deps/docs/ should contain generated documentation"
            );
        }
    }
}
