//! Expression types for the parser.
//!
//! Defines the Abstract Syntax Tree nodes and positioned expressions
//! used throughout the parsing phase.

use crate::lexer::Position;
use crate::types::Value;
use std::sync::Arc;

/// Abstract Syntax Tree node for Scheme expressions.
///
/// Represents the hierarchical structure of parsed Scheme code.
///
/// Expression types:
/// - **Atom**: Primitive values (numbers, strings, symbols, booleans)
/// - **List**: Compound expressions for procedure calls and special forms
/// - **Quote**: Quoted expressions that prevent evaluation
///
/// | Scheme Code | AST Representation |
/// |-------------|-------------------|
/// | `42` | `Expression::Atom(Value::Number(42.0))` |
/// | `"hello"` | `Expression::Atom(Value::String("hello"))` |
/// | `x` | `Expression::Atom(Value::Symbol("x"))` |
/// | `(+ 1 2)` | `Expression::List([Atom(+), Atom(1), Atom(2)])` |
/// | `'x` | `Expression::Quote(Box::new(Atom(Symbol("x"))))` |
/// | `'(a b)` | `Expression::Quote(Box::new(List([Atom(a), Atom(b)])))` |
#[derive(Debug, PartialEq)]
pub enum Expression {
    /// Atomic expressions (primitive values)
    Atom(Value),

    /// List expressions (compound structures)
    List(Vec<Arc<Expression>>),

    /// Quoted expressions (prevent evaluation)
    ///
    /// Uses Arc because recursive enum variants would have infinite size.
    /// Arc provides heap allocation to break the recursion and enables
    /// arbitrarily deep nesting without stack overflow while allowing
    /// efficient sharing of expression trees.
    Quote(Arc<Expression>),
}

impl Expression {
    /// Create an atomic expression from a Value.
    pub fn atom(value: Value) -> Self {
        Expression::Atom(value)
    }

    /// Create an atomic expression from a Value wrapped in Arc.
    pub fn arc_atom(value: Value) -> Arc<Self> {
        Arc::new(Expression::Atom(value))
    }

    /// Create a list expression from a vector of expressions.
    pub fn list(exprs: Vec<Arc<Expression>>) -> Self {
        Expression::List(exprs)
    }

    /// Create a list expression from a vector of expressions wrapped in Arc.
    pub fn arc_list(exprs: Vec<Arc<Expression>>) -> Arc<Self> {
        Arc::new(Expression::List(exprs))
    }

    /// Create a quoted expression.
    ///
    /// Handles the Arc allocation required for the recursive structure.
    pub fn quote(expr: Arc<Expression>) -> Self {
        Expression::Quote(expr)
    }

    /// Create a quoted expression wrapped in Arc.
    pub fn arc_quote(expr: Arc<Expression>) -> Arc<Self> {
        Arc::new(Expression::Quote(expr))
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
    pub fn as_atom(&self) -> Option<&Value> {
        match self {
            Expression::Atom(value) => Some(value),
            _ => None,
        }
    }

    /// Get the list of expressions if this expression is a list.
    pub fn as_list(&self) -> Option<&Vec<Arc<Expression>>> {
        match self {
            Expression::List(exprs) => Some(exprs),
            _ => None,
        }
    }

    /// Get the quoted expression if this expression is quoted.
    pub fn as_quoted(&self) -> Option<&Arc<Expression>> {
        match self {
            Expression::Quote(expr) => Some(expr),
            _ => None,
        }
    }

    /// Get a human-readable description of the expression type.
    ///
    /// Useful for error messages and debugging output.
    pub fn type_name(&self) -> &'static str {
        match self {
            Expression::Atom(_) => "atom",
            Expression::List(_) => "list",
            Expression::Quote(_) => "quote",
        }
    }
}

/// Expression with source position information for error reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedExpression {
    pub expr: Arc<Expression>,
    pub position: Position,
}

impl PositionedExpression {
    /// Create a new positioned expression.
    pub fn new(expr: Arc<Expression>, position: Position) -> Self {
        Self { expr, position }
    }

    /// Extract the expression without position information.
    pub fn into_expr(self) -> Arc<Expression> {
        self.expr
    }
}
