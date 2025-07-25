//! Expression types for the parser.
//!
//! Defines the Abstract Syntax Tree nodes and positioned expressions
//! used throughout the parsing phase.

use crate::lexer::Position;
use crate::types::Value;

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
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Atomic expressions (primitive values)
    Atom(Value),

    /// List expressions (compound structures)
    List(Vec<Expression>),

    /// Quoted expressions (prevent evaluation)
    ///
    /// Uses Box because recursive enum variants would have infinite size.
    /// Box provides heap allocation to break the recursion and enables
    /// arbitrarily deep nesting without stack overflow.
    Quote(Box<Expression>),
}

impl Expression {
    /// Create an atomic expression from a Value.
    pub fn atom(value: Value) -> Self {
        Expression::Atom(value)
    }

    /// Create a list expression from a vector of expressions.
    pub fn list(exprs: Vec<Expression>) -> Self {
        Expression::List(exprs)
    }

    /// Create a quoted expression.
    ///
    /// Handles the Box allocation required for the recursive structure.
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
    pub fn as_atom(&self) -> Option<&Value> {
        match self {
            Expression::Atom(value) => Some(value),
            _ => None,
        }
    }

    /// Get the list of expressions if this expression is a list.
    pub fn as_list(&self) -> Option<&Vec<Expression>> {
        match self {
            Expression::List(exprs) => Some(exprs),
            _ => None,
        }
    }

    /// Get the quoted expression if this expression is quoted.
    pub fn as_quoted(&self) -> Option<&Expression> {
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
    pub expr: Expression,
    pub position: Position,
}

impl PositionedExpression {
    /// Create a new positioned expression.
    pub fn new(expr: Expression, position: Position) -> Self {
        Self { expr, position }
    }

    /// Extract the expression without position information.
    pub fn into_expr(self) -> Expression {
        self.expr
    }
}
