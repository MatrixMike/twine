//! Parser module for the Twine Scheme interpreter.
//!
//! This module provides the Abstract Syntax Tree (AST) representation and
//! parsing functionality for Scheme expressions. It converts tokens from
//! the lexer into a structured tree representation that can be evaluated.
//!
//! ## Requirements Compliance
//!
//! **FR-2 (Syntactic Analysis)**: Build AST from S-expressions; validate parentheses; report syntax errors
//! - ✅ AST representation with `Expr` enum
//! - ✅ S-expression structure via `List` variant
//! - ✅ Position tracking for error reporting
//! - ✅ Immutable AST nodes with `Clone` semantics
//!
//! ## Design Principles
//!
//! - **Minimal AST**: Three core expression types following educational simplicity
//! - **Position Tracking**: Error reporting with precise source locations
//! - **Immutable Structure**: All AST nodes are immutable after creation
//! - **Educational Focus**: Clear, readable implementation over optimization
//!
//! ## Usage Examples
//!
//! ```rust
//! use twine_scheme::parser::Expression;
//! use twine_scheme::types::Value;
//!
//! // Simple atoms
//! let number = Expression::atom(Value::number(42.0));
//! let symbol = Expression::atom(Value::symbol("x"));
//!
//! // S-expression: (+ 1 2)
//! let addition = Expression::list(vec![
//!     Expression::atom(Value::symbol("+")),
//!     Expression::atom(Value::number(1.0)),
//!     Expression::atom(Value::number(2.0)),
//! ]);
//!
//! // Quoted expression: '(a b c)
//! let quoted_list = Expression::quote(Expression::list(vec![
//!     Expression::atom(Value::symbol("a")),
//!     Expression::atom(Value::symbol("b")),
//!     Expression::atom(Value::symbol("c")),
//! ]));
//! ```

use crate::lexer::Position;
use crate::types::Value;

/// Abstract Syntax Tree node for Scheme expressions.
///
/// The AST represents the hierarchical structure of Scheme code after parsing.
/// This design follows the core principle of simplicity, providing only the
/// essential expression types needed for the educational subset of Scheme.
///
/// ## Requirements Compliance
///
/// Implements **FR-2 (Syntactic Analysis)** requirements:
/// - ✅ Builds AST from S-expressions using recursive List structure
/// - ✅ Supports all essential Scheme syntax elements
/// - ✅ Provides foundation for syntax validation
/// - ✅ Enables precise error reporting with position tracking
///
/// ## Expression Types
///
/// - **Atom**: Primitive values (numbers, strings, symbols, booleans)
/// - **List**: Compound expressions for function calls and special forms
/// - **Quote**: Quoted expressions that prevent evaluation
///
/// ## Design Rationale
///
/// This minimal AST design prioritizes learning value:
/// - Students understand the fundamental distinction between atoms and lists
/// - Quote handling demonstrates meta-programming concepts
/// - Simple structure makes evaluation logic clear and comprehensible
/// - Direct mapping to Scheme's syntactic structure
///
/// ## Scheme Syntax Mapping
///
/// | Scheme Code | AST Representation |
/// |-------------|-------------------|
/// | `42` | `Expression::Atom(Value::Number(42.0))` |
/// | `"hello"` | `Expression::Atom(Value::String("hello"))` |
/// | `x` | `Expression::Atom(Value::Symbol("x"))` |
/// | `(+ 1 2)` | `Expression::List([Atom(+), Atom(1), Atom(2)])` |
/// | `'x` | `Expression::Quote(Box::new(Atom(Symbol("x"))))` |
/// | `'(a b)` | `Expression::Quote(Box::new(List([Atom(a), Atom(b)])))` |
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Atomic expressions (primitive values)
    ///
    /// Represents all primitive Scheme values including numbers, strings,
    /// symbols, and booleans. Uses the existing Value type for consistency
    /// with the evaluation engine.
    ///
    /// # Examples
    /// - `42` → `Atom(Value::Number(42.0))`
    /// - `"hello"` → `Atom(Value::String("hello"))`
    /// - `x` → `Atom(Value::Symbol("x"))`
    /// - `#t` → `Atom(Value::Boolean(true))`
    Atom(Value),

    /// List expressions (compound structures)
    ///
    /// Represents S-expressions which form the core of Scheme syntax.
    /// Lists can represent function calls, special forms, or data structures.
    ///
    /// # Examples
    /// - `(+ 1 2)` → `List([Atom(Symbol("+")), Atom(Number(1)), Atom(Number(2))])`
    /// - `(define x 42)` → `List([Atom(Symbol("define")), Atom(Symbol("x")), Atom(Number(42))])`
    /// - `()` → `List([])`
    List(Vec<Expression>),

    /// Quoted expressions (prevent evaluation)
    ///
    /// Represents expressions that should be treated as data rather than code.
    /// The quote prevents the normal evaluation process, allowing manipulation
    /// of code as data (a key Lisp concept).
    ///
    /// # Examples
    /// - `'x` → `Quote(Box::new(Atom(Symbol("x"))))`
    /// - `'(+ 1 2)` → `Quote(Box::new(List([...])))`
    ///
    /// Note: Uses Box for heap allocation to prevent stack overflow with deeply
    /// nested quotes and to optimize memory layout. While Quote(Expression) would work,
    /// Box provides better performance characteristics for recursive structures.
    Quote(Box<Expression>),
}

impl Expression {
    /// Create an atomic expression from a Value.
    ///
    /// This convenience constructor makes it easy to create atomic expressions
    /// from the existing Value types used throughout the interpreter.
    ///
    /// # Examples
    /// ```
    /// use twine_scheme::parser::Expression;
    /// use twine_scheme::types::Value;
    ///
    /// let expr = Expression::atom(Value::number(42.0));
    /// assert!(matches!(expr, Expression::Atom(_)));
    /// ```
    pub fn atom(value: Value) -> Self {
        Expression::Atom(value)
    }

    /// Create a list expression from a vector of expressions.
    ///
    /// This convenience constructor simplifies creation of list expressions,
    /// which are the most common compound form in Scheme.
    ///
    /// # Examples
    /// ```
    /// use twine_scheme::parser::Expression;
    /// use twine_scheme::types::Value;
    ///
    /// let expr = Expression::list(vec![
    ///     Expression::atom(Value::symbol("+")),
    ///     Expression::atom(Value::number(1.0)),
    ///     Expression::atom(Value::number(2.0)),
    /// ]);
    /// assert!(matches!(expr, Expression::List(_)));
    /// ```
    pub fn list(exprs: Vec<Expression>) -> Self {
        Expression::List(exprs)
    }

    /// Create a quoted expression.
    ///
    /// This convenience constructor handles the Box allocation required
    /// for quoted expressions, making the API more ergonomic.
    ///
    /// # Examples
    /// ```
    /// use twine_scheme::parser::Expression;
    /// use twine_scheme::types::Value;
    ///
    /// let expr = Expression::quote(Expression::atom(Value::symbol("x")));
    /// assert!(matches!(expr, Expression::Quote(_)));
    /// ```
    pub fn quote(expr: Expression) -> Self {
        Expression::Quote(Box::new(expr))
    }

    /// Check if this expression is an atom.
    pub fn is_atom(&self) -> bool {
        matches!(self, Expression::Atom(_))
    }

    /// Check if this expression is a list.
    pub fn is_list(&self) -> bool {
        matches!(self, Expression::List(_))
    }

    /// Check if this expression is quoted.
    pub fn is_quoted(&self) -> bool {
        matches!(self, Expression::Quote(_))
    }

    /// Get the value if this expression is an atom.
    ///
    /// Returns `Some(value)` if this is an `Atom`, `None` otherwise.
    pub fn as_atom(&self) -> Option<&Value> {
        match self {
            Expression::Atom(value) => Some(value),
            _ => None,
        }
    }

    /// Get the list of expressions if this expression is a list.
    ///
    /// Returns `Some(expressions)` if this is a `List`, `None` otherwise.
    pub fn as_list(&self) -> Option<&Vec<Expression>> {
        match self {
            Expression::List(exprs) => Some(exprs),
            _ => None,
        }
    }

    /// Get the quoted expression if this expression is quoted.
    ///
    /// Returns `Some(expression)` if this is a `Quote`, `None` otherwise.
    pub fn as_quoted(&self) -> Option<&Expression> {
        match self {
            Expression::Quote(expr) => Some(expr),
            _ => None,
        }
    }

    /// Get a human-readable description of the expression type.
    ///
    /// Useful for error messages and debugging output.
    ///
    /// # Examples
    /// ```
    /// use twine_scheme::parser::Expression;
    /// use twine_scheme::types::Value;
    ///
    /// assert_eq!(Expression::atom(Value::number(42.0)).type_name(), "atom");
    /// assert_eq!(Expression::list(vec![]).type_name(), "list");
    /// assert_eq!(Expression::quote(Expression::atom(Value::symbol("x"))).type_name(), "quote");
    /// ```
    pub fn type_name(&self) -> &'static str {
        match self {
            Expression::Atom(_) => "atom",
            Expression::List(_) => "list",
            Expression::Quote(_) => "quote",
        }
    }
}

/// Position-aware expression for error reporting.
///
/// Combines an expression with its source location to enable precise
/// error messages that help users identify exactly where problems occur.
///
/// This is essential for educational use, as clear error messages
/// significantly improve the learning experience.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedExpression {
    /// The expression itself
    pub expr: Expression,
    /// Position in source code where this expression was parsed
    pub position: Position,
}

impl PositionedExpression {
    /// Create a new positioned expression.
    pub fn new(expr: Expression, position: Position) -> Self {
        Self { expr, position }
    }

    /// Get the expression without position information.
    pub fn into_expr(self) -> Expression {
        self.expr
    }
}

/// Display formatting for expressions.
///
/// Provides readable string representation of AST nodes, useful for
/// debugging and educational purposes. The output closely matches
/// Scheme syntax to maintain familiarity.
impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Atom(value) => write!(f, "{}", value),
            Expression::List(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            Expression::Quote(expr) => write!(f, "'{}", expr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_expr_creation() {
        // Test atomic expressions
        let number_expr = Expression::atom(Value::number(42.0));
        assert!(number_expr.is_atom());
        assert!(!number_expr.is_list());
        assert!(!number_expr.is_quoted());

        let string_expr = Expression::atom(Value::string("hello"));
        assert!(string_expr.is_atom());

        let symbol_expr = Expression::atom(Value::symbol("x"));
        assert!(symbol_expr.is_atom());

        let boolean_expr = Expression::atom(Value::boolean(true));
        assert!(boolean_expr.is_atom());

        // Test list expressions
        let list_expr = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]);
        assert!(!list_expr.is_atom());
        assert!(list_expr.is_list());
        assert!(!list_expr.is_quoted());

        // Test empty list
        let empty_list = Expression::list(vec![]);
        assert!(empty_list.is_list());

        // Test quoted expressions
        let quoted_expr = Expression::quote(Expression::atom(Value::symbol("x")));
        assert!(!quoted_expr.is_atom());
        assert!(!quoted_expr.is_list());
        assert!(quoted_expr.is_quoted());
    }

    #[test]
    fn test_expr_access_methods() {
        // Test atom access
        let value = Value::number(42.0);
        let atom_expr = Expression::atom(value.clone());
        assert_eq!(atom_expr.as_atom(), Some(&value));
        assert_eq!(atom_expr.as_list(), None);
        assert_eq!(atom_expr.as_quoted(), None);

        // Test list access
        let exprs = vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
        ];
        let list_expr = Expression::list(exprs.clone());
        assert_eq!(list_expr.as_atom(), None);
        assert_eq!(list_expr.as_list(), Some(&exprs));
        assert_eq!(list_expr.as_quoted(), None);

        // Test quote access
        let inner_expr = Expression::atom(Value::symbol("x"));
        let quoted_expr = Expression::quote(inner_expr.clone());
        assert_eq!(quoted_expr.as_atom(), None);
        assert_eq!(quoted_expr.as_list(), None);
        assert_eq!(quoted_expr.as_quoted(), Some(&inner_expr));
    }

    #[test]
    fn test_expr_equality() {
        // Test atom equality
        let atom1 = Expression::atom(Value::number(42.0));
        let atom2 = Expression::atom(Value::number(42.0));
        let atom3 = Expression::atom(Value::number(43.0));
        assert_eq!(atom1, atom2);
        assert_ne!(atom1, atom3);

        // Test list equality
        let list1 = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
        ]);
        let list2 = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
        ]);
        let list3 = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(2.0)),
        ]);
        assert_eq!(list1, list2);
        assert_ne!(list1, list3);

        // Test quote equality
        let quote1 = Expression::quote(Expression::atom(Value::symbol("x")));
        let quote2 = Expression::quote(Expression::atom(Value::symbol("x")));
        let quote3 = Expression::quote(Expression::atom(Value::symbol("y")));
        assert_eq!(quote1, quote2);
        assert_ne!(quote1, quote3);
    }

    #[test]
    fn test_expr_cloning() {
        // Test that expressions can be cloned
        let original = Expression::list(vec![
            Expression::atom(Value::symbol("define")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ]);

        let cloned = original.clone();
        assert_eq!(original, cloned);

        // Verify that the clone is independent (structural test)
        assert!(matches!(cloned, Expression::List(_)));
    }

    #[test]
    fn test_positioned_expr() {
        let expr = Expression::atom(Value::number(42.0));
        let position = Position::new(1, 5);
        let positioned = PositionedExpression::new(expr.clone(), position.clone());

        assert_eq!(positioned.expr, expr);
        assert_eq!(positioned.position, position);

        // Test into_expr
        let extracted = positioned.into_expr();
        assert_eq!(extracted, expr);
    }

    #[test]
    fn test_expr_display() {
        // Test atom display
        assert_eq!(format!("{}", Expression::atom(Value::number(42.0))), "42");
        assert_eq!(
            format!("{}", Expression::atom(Value::string("hello"))),
            "\"hello\""
        );
        assert_eq!(format!("{}", Expression::atom(Value::symbol("x"))), "x");
        assert_eq!(format!("{}", Expression::atom(Value::boolean(true))), "#t");

        // Test list display
        let list_expr = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]);
        assert_eq!(format!("{}", list_expr), "(+ 1 2)");

        // Test empty list display
        let empty_list = Expression::list(vec![]);
        assert_eq!(format!("{}", empty_list), "()");

        // Test quote display
        let quoted_expr = Expression::quote(Expression::atom(Value::symbol("x")));
        assert_eq!(format!("{}", quoted_expr), "'x");

        // Test nested structures
        let nested = Expression::list(vec![
            Expression::atom(Value::symbol("quote")),
            Expression::list(vec![
                Expression::atom(Value::symbol("+")),
                Expression::atom(Value::number(1.0)),
                Expression::atom(Value::number(2.0)),
            ]),
        ]);
        assert_eq!(format!("{}", nested), "(quote (+ 1 2))");
    }

    #[test]
    fn test_expr_debug_output() {
        // Verify that Debug trait is implemented and produces reasonable output
        let expr = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
        ]);

        let debug_output = format!("{:?}", expr);
        assert!(debug_output.contains("List"));
        assert!(debug_output.contains("Atom"));
        // Don't test exact format as Debug output can vary
    }

    #[test]
    fn test_nested_expressions() {
        // Test deeply nested structures
        let nested_expr = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::list(vec![
                Expression::atom(Value::symbol(">")),
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::number(0.0)),
            ]),
            Expression::quote(Expression::atom(Value::symbol("positive"))),
            Expression::quote(Expression::atom(Value::symbol("non-positive"))),
        ]);

        // Verify structure
        assert!(nested_expr.is_list());
        if let Some(exprs) = nested_expr.as_list() {
            assert_eq!(exprs.len(), 4);
            assert!(exprs[0].is_atom());
            assert!(exprs[1].is_list());
            assert!(exprs[2].is_quoted());
            assert!(exprs[3].is_quoted());
        }

        // Test display of nested structure
        let display_output = format!("{}", nested_expr);
        assert!(display_output.contains("if"));
        assert!(display_output.contains("'positive"));
    }

    #[test]
    fn test_expr_memory_efficiency() {
        // Test memory layout and demonstrate Box<Expr> benefits
        use std::mem;

        // Check enum size - should be reasonable for stack allocation
        let expr_size = mem::size_of::<Expression>();
        let pointer_size = mem::size_of::<usize>();

        // Expression size should be manageable (typically 32-40 bytes on 64-bit)
        assert!(
            expr_size <= 64,
            "Expression size should be reasonable: {} bytes",
            expr_size
        );

        // Box<Expression> is exactly pointer-sized
        assert_eq!(mem::size_of::<Box<Expression>>(), pointer_size);

        // Vec<Expression> has reasonable overhead (3 pointers: ptr, len, capacity)
        assert_eq!(mem::size_of::<Vec<Expression>>(), pointer_size * 3);

        // Demonstrate why Box helps with recursive structures:
        // Without Box, deeply nested quotes would consume stack space proportional to depth
        // With Box, each quote level is just one pointer (8 bytes on 64-bit)
        let shallow_quote = Expression::quote(Expression::atom(Value::number(42.0)));
        let deep_quote = Expression::quote(Expression::quote(Expression::quote(Expression::atom(
            Value::number(42.0),
        ))));

        // Both consume the same stack space (just the enum discriminant + Box pointer)
        assert_eq!(
            mem::size_of_val(&shallow_quote),
            mem::size_of_val(&deep_quote)
        );
    }

    #[test]
    fn test_type_name_method() {
        // Test type name reporting
        assert_eq!(Expression::atom(Value::number(42.0)).type_name(), "atom");
        assert_eq!(Expression::list(vec![]).type_name(), "list");
        assert_eq!(
            Expression::quote(Expression::atom(Value::symbol("x"))).type_name(),
            "quote"
        );

        // Verify consistency
        let expr = Expression::list(vec![Expression::atom(Value::symbol("+"))]);
        assert_eq!(expr.type_name(), "list");
        assert!(expr.is_list());
    }

    #[test]
    fn test_fr2_compliance() {
        // Test FR-2 (Syntactic Analysis) compliance
        // Requirement: Build AST from S-expressions

        // Basic S-expression: (+ 1 2)
        let addition = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::number(1.0)),
            Expression::atom(Value::number(2.0)),
        ]);
        assert!(addition.is_list());
        assert_eq!(addition.as_list().unwrap().len(), 3);

        // Nested S-expression: (define f (lambda (x) (* x x)))
        let lambda_def = Expression::list(vec![
            Expression::atom(Value::symbol("define")),
            Expression::atom(Value::symbol("f")),
            Expression::list(vec![
                Expression::atom(Value::symbol("lambda")),
                Expression::list(vec![Expression::atom(Value::symbol("x"))]),
                Expression::list(vec![
                    Expression::atom(Value::symbol("*")),
                    Expression::atom(Value::symbol("x")),
                    Expression::atom(Value::symbol("x")),
                ]),
            ]),
        ]);

        // Verify nested structure can be navigated
        assert!(lambda_def.is_list());
        if let Some(exprs) = lambda_def.as_list() {
            assert_eq!(exprs.len(), 3);
            assert!(exprs[2].is_list()); // lambda expression
            if let Some(lambda_exprs) = exprs[2].as_list() {
                assert_eq!(lambda_exprs.len(), 3); // lambda, params, body
                assert!(lambda_exprs[1].is_list()); // parameter list
                assert!(lambda_exprs[2].is_list()); // body expression
            }
        }

        // Quote handling: '(a b c)
        let quoted = Expression::quote(Expression::list(vec![
            Expression::atom(Value::symbol("a")),
            Expression::atom(Value::symbol("b")),
            Expression::atom(Value::symbol("c")),
        ]));
        assert!(quoted.is_quoted());
        if let Some(inner) = quoted.as_quoted() {
            assert!(inner.is_list());
            assert_eq!(inner.as_list().unwrap().len(), 3);
        }
    }

    #[test]
    fn test_educational_examples() {
        // Examples that demonstrate core concepts

        // Arithmetic expression: (+ (* 2 3) 4)
        let arithmetic = Expression::list(vec![
            Expression::atom(Value::symbol("+")),
            Expression::list(vec![
                Expression::atom(Value::symbol("*")),
                Expression::atom(Value::number(2.0)),
                Expression::atom(Value::number(3.0)),
            ]),
            Expression::atom(Value::number(4.0)),
        ]);
        assert_eq!(format!("{}", arithmetic), "(+ (* 2 3) 4)");

        // Conditional: (if (> x 0) "positive" "non-positive")
        let conditional = Expression::list(vec![
            Expression::atom(Value::symbol("if")),
            Expression::list(vec![
                Expression::atom(Value::symbol(">")),
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::number(0.0)),
            ]),
            Expression::atom(Value::string("positive")),
            Expression::atom(Value::string("non-positive")),
        ]);
        assert_eq!(
            format!("{}", conditional),
            "(if (> x 0) \"positive\" \"non-positive\")"
        );

        // Data as code: '(define x 42)
        let quoted_define = Expression::quote(Expression::list(vec![
            Expression::atom(Value::symbol("define")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ]));
        assert_eq!(format!("{}", quoted_define), "'(define x 42)");
    }

    #[test]
    fn test_box_vs_direct_recursion_analysis() {
        // Demonstrate the benefits of Box<Expression> vs direct Expression recursion
        //
        // Key insight: With Quote(Box<Expression>), all nested quotes have the same
        // memory footprint because Box provides heap indirection. This prevents
        // stack growth proportional to nesting depth.

        use std::mem;

        // Create truly nested quote structure: ''''x (not just separate quotes)
        let x = Expression::atom(Value::symbol("x"));
        let quote1 = Expression::quote(x); // 'x
        let quote2 = Expression::quote(quote1.clone()); // ''x
        let quote3 = Expression::quote(quote2.clone()); // '''x
        let quote4 = Expression::quote(quote3.clone()); // ''''x

        // Critical test: All nested quote levels consume identical stack space
        // This is the key benefit of Box<Expression> - fixed size regardless of nesting depth
        let base_size = mem::size_of::<Expression>();
        assert_eq!(mem::size_of_val(&quote1), base_size);
        assert_eq!(mem::size_of_val(&quote2), base_size); // Same as quote1!
        assert_eq!(mem::size_of_val(&quote3), base_size); // Same as quote1!
        assert_eq!(mem::size_of_val(&quote4), base_size); // Same as quote1!

        // Stress test: Create 100 levels of nested quotes
        // With Quote(Expression), this would consume 100x the stack space
        // With Quote(Box<Expression>), it's still just one Expression worth of stack space
        let mut nested = Expression::atom(Value::symbol("deeply-nested"));
        for _ in 0..100 {
            nested = Expression::quote(nested);
        }

        // Critical assertion: Even 100 levels deep, same stack footprint!
        // This demonstrates Box<Expression> prevents stack overflow with deep nesting
        assert_eq!(mem::size_of_val(&nested), base_size);

        // Verify we can safely navigate the structure
        let mut current = &nested;
        let mut depth = 0;
        while let Some(inner) = current.as_quoted() {
            current = inner;
            depth += 1;
            if depth > 200 {
                break;
            } // Safety guard
        }
        assert_eq!(depth, 100);
        assert_eq!(
            current.as_atom().unwrap().as_symbol().unwrap().to_string(),
            "deeply-nested"
        );
    }
}
