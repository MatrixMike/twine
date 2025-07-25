//! Parser module for the Twine Scheme interpreter.
//!
//! Converts tokens from the lexer into a structured AST representation.
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

use crate::lexer::{Lexer, Position};
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

/// Parser for converting tokens into Abstract Syntax Tree.
///
/// Implements recursive descent parsing for Scheme S-expressions.
/// Maintains current position in token stream for error reporting.
///
/// # Example
/// ```rust
/// use twine_scheme::parser::Parser;
///
/// let input = "(+ 1 2)".to_string();
/// let mut parser = Parser::new(input)?;
/// let expr = parser.parse_expression()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Parser {
    tokens: Vec<crate::lexer::PositionedToken>,
    current: usize,
}

impl Parser {
    /// Create a new parser for the given input string.
    ///
    /// Tokenizes the input using the lexer and prepares for parsing.
    pub fn new(input: String) -> crate::Result<Self> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();

        // Collect all tokens from the lexer
        loop {
            let positioned_token = lexer.next_token()?;
            let is_eof = positioned_token.token.is_eof();
            tokens.push(positioned_token);

            if is_eof {
                break;
            }
        }

        Ok(Self { tokens, current: 0 })
    }

    /// Get the current token without advancing the parser.
    ///
    /// Returns the EOF token if we've reached the end of input.
    pub fn peek(&self) -> &crate::lexer::PositionedToken {
        self.tokens
            .get(self.current)
            .unwrap_or(self.tokens.last().unwrap()) // Always has EOF token
    }

    /// Advance to the next token and return the previous current token.
    ///
    /// Returns the EOF token if we've already reached the end.
    pub fn advance(&mut self) -> crate::lexer::PositionedToken {
        let token = self.peek().clone();
        if self.current < self.tokens.len() - 1 {
            self.current += 1;
        }
        token
    }

    /// Check if we've reached the end of the token stream.
    pub fn is_at_end(&self) -> bool {
        self.peek().token.is_eof()
    }

    /// Get the current position in the source for error reporting.
    pub fn current_position(&self) -> crate::lexer::Position {
        self.peek().position.clone()
    }

    /// Parse a single expression from the token stream.
    ///
    /// Handles atoms, lists, and quoted expressions according to Scheme syntax.
    pub fn parse_expression(&mut self) -> crate::Result<PositionedExpression> {
        let position = self.current_position();

        match &self.peek().token {
            crate::lexer::Token::Quote => {
                self.advance(); // consume quote
                let quoted_expr = self.parse_expression()?;
                Ok(PositionedExpression::new(
                    Expression::quote(quoted_expr.expr),
                    position,
                ))
            }
            crate::lexer::Token::LeftParen => {
                self.advance(); // consume left paren
                let mut expressions = Vec::new();

                while !self.is_at_end()
                    && !matches!(self.peek().token, crate::lexer::Token::RightParen)
                {
                    expressions.push(self.parse_expression()?.expr);
                }

                if self.is_at_end() {
                    return Err(crate::Error::syntax_error(
                        "Unexpected end of input, expected ')'",
                        position.line,
                        position.column,
                    ));
                }

                self.advance(); // consume right paren
                Ok(PositionedExpression::new(
                    Expression::list(expressions),
                    position,
                ))
            }
            crate::lexer::Token::RightParen => Err(crate::Error::syntax_error(
                "Unexpected ')'",
                position.line,
                position.column,
            )),
            crate::lexer::Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(PositionedExpression::new(
                    Expression::atom(crate::types::Value::number(value)),
                    position,
                ))
            }
            crate::lexer::Token::String(s) => {
                let value = s.clone();
                self.advance();
                Ok(PositionedExpression::new(
                    Expression::atom(crate::types::Value::string(&value)),
                    position,
                ))
            }
            crate::lexer::Token::Symbol(s) => {
                let value = s.clone();
                self.advance();
                Ok(PositionedExpression::new(
                    Expression::atom(crate::types::Value::symbol(&value)),
                    position,
                ))
            }
            crate::lexer::Token::Boolean(b) => {
                let value = *b;
                self.advance();
                Ok(PositionedExpression::new(
                    Expression::atom(crate::types::Value::boolean(value)),
                    position,
                ))
            }
            crate::lexer::Token::Eof => Err(crate::Error::syntax_error(
                "Unexpected end of input",
                position.line,
                position.column,
            )),
        }
    }

    /// Parse all expressions from the token stream.
    ///
    /// Returns a vector of all top-level expressions in the input.
    pub fn parse_all(&mut self) -> crate::Result<Vec<PositionedExpression>> {
        let mut expressions = Vec::new();

        while !self.is_at_end() {
            expressions.push(self.parse_expression()?);
        }

        Ok(expressions)
    }
}

/// Display formatting for expressions.
///
/// Provides readable string representation that closely matches Scheme syntax.
impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Atom(value) => write!(f, "{value}"),
            Expression::List(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{expr}")?;
                }
                write!(f, ")")
            }
            Expression::Quote(expr) => write!(f, "'{expr}"),
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

        // Expression size should be manageable (now larger due to Procedure variant in Value)
        // The size increased because Value now includes Procedure which contains Environment
        assert!(
            expr_size <= 96,
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

    // Parser struct tests
    #[test]
    fn test_parser_creation() {
        let parser = Parser::new("42".to_string()).unwrap();
        assert_eq!(parser.current, 0);
        assert!(!parser.is_at_end());
    }

    #[test]
    fn test_parser_peek_and_advance() {
        let mut parser = Parser::new("42 hello".to_string()).unwrap();

        // Should start with number token
        let first_token = parser.peek().clone();
        assert!(matches!(
            first_token.token,
            crate::lexer::Token::Number(42.0)
        ));

        // Advance should return the token we just peeked
        let advanced_token = parser.advance();
        assert_eq!(advanced_token.token, first_token.token);

        // Now should be at symbol
        let second_token = parser.peek();
        assert!(matches!(second_token.token, crate::lexer::Token::Symbol(ref s) if s == "hello"));
    }

    #[test]
    fn test_parser_is_at_end() {
        let mut parser = Parser::new("42".to_string()).unwrap();

        assert!(!parser.is_at_end());
        parser.advance(); // consume number
        assert!(parser.is_at_end()); // should be at EOF
    }

    #[test]
    fn test_parse_atom_expressions() {
        // Number
        let mut parser = Parser::new("42".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_atom());
        assert_eq!(expr.expr.as_atom().unwrap().as_number().unwrap(), 42.0);

        // String
        let mut parser = Parser::new("\"hello\"".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_atom());
        assert_eq!(
            expr.expr
                .as_atom()
                .unwrap()
                .as_string()
                .unwrap()
                .to_string(),
            "hello"
        );

        // Symbol
        let mut parser = Parser::new("foo".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_atom());
        assert_eq!(
            expr.expr
                .as_atom()
                .unwrap()
                .as_symbol()
                .unwrap()
                .to_string(),
            "foo"
        );

        // Boolean
        let mut parser = Parser::new("#t".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_atom());
        assert_eq!(expr.expr.as_atom().unwrap().as_boolean().unwrap(), true);
    }

    #[test]
    fn test_parse_list_expressions() {
        // Simple list: (1 2 3)
        let mut parser = Parser::new("(1 2 3)".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_list());
        let list = expr.expr.as_list().unwrap();
        assert_eq!(list.len(), 3);

        // Nested list: (+ (* 2 3) 4)
        let mut parser = Parser::new("(+ (* 2 3) 4)".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_list());
        let outer_list = expr.expr.as_list().unwrap();
        assert_eq!(outer_list.len(), 3);
        assert!(outer_list[1].is_list()); // (* 2 3) should be a list

        // Empty list: ()
        let mut parser = Parser::new("()".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_list());
        assert_eq!(expr.expr.as_list().unwrap().len(), 0);
    }

    #[test]
    fn test_parse_quoted_expressions() {
        // Simple quote: 'x
        let mut parser = Parser::new("'x".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_quoted());
        let quoted = expr.expr.as_quoted().unwrap();
        assert!(quoted.is_atom());

        // Quoted list: '(a b c)
        let mut parser = Parser::new("'(a b c)".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_quoted());
        let quoted = expr.expr.as_quoted().unwrap();
        assert!(quoted.is_list());
        assert_eq!(quoted.as_list().unwrap().len(), 3);

        // Nested quotes: ''x
        let mut parser = Parser::new("''x".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_quoted());
        let first_quote = expr.expr.as_quoted().unwrap();
        assert!(first_quote.is_quoted());
    }

    #[test]
    fn test_parse_all_expressions() {
        let mut parser = Parser::new("42 hello (+ 1 2) 'world".to_string()).unwrap();
        let expressions = parser.parse_all().unwrap();

        assert_eq!(expressions.len(), 4);
        assert!(expressions[0].expr.is_atom()); // 42
        assert!(expressions[1].expr.is_atom()); // hello
        assert!(expressions[2].expr.is_list()); // (+ 1 2)
        assert!(expressions[3].expr.is_quoted()); // 'world
    }

    #[test]
    fn test_parser_error_handling() {
        // Unexpected right paren
        let mut parser = Parser::new(")".to_string()).unwrap();
        assert!(parser.parse_expression().is_err());

        // Unmatched left paren
        let mut parser = Parser::new("(1 2".to_string()).unwrap();
        assert!(parser.parse_expression().is_err());

        // Empty input
        let mut parser = Parser::new("".to_string()).unwrap();
        assert!(parser.parse_expression().is_err());
    }

    #[test]
    fn test_parser_position_tracking() {
        let mut parser = Parser::new("42".to_string()).unwrap();
        let position = parser.current_position();
        assert_eq!(position.line, 1);
        assert_eq!(position.column, 1);

        let expr = parser.parse_expression().unwrap();
        assert_eq!(expr.position.line, 1);
        assert_eq!(expr.position.column, 1);
    }

    #[test]
    fn test_fr2_compliance_parser() {
        // Test FR-2: Syntactic Analysis compliance

        // Build AST from S-expressions
        let mut parser = Parser::new(
            "(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))".to_string(),
        )
        .unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_list());

        // Validate parentheses handling
        let mut parser = Parser::new("((()))".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert!(expr.expr.is_list());
        let outer = expr.expr.as_list().unwrap();
        assert_eq!(outer.len(), 1);
        assert!(outer[0].is_list());

        // Report syntax errors
        let mut parser = Parser::new("(unclosed".to_string()).unwrap();
        assert!(parser.parse_expression().is_err());

        // Unexpected right paren at start should error
        let mut parser = Parser::new(")".to_string()).unwrap();
        assert!(parser.parse_expression().is_err());

        // Multiple unmatched parens should error
        let mut parser = Parser::new("(()".to_string()).unwrap();
        assert!(parser.parse_expression().is_err());
    }

    #[test]
    fn test_educational_parser_examples() {
        // Simple arithmetic that students can understand
        let mut parser = Parser::new("(+ 1 2)".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert_eq!(format!("{}", expr.expr), "(+ 1 2)");

        // Procedure definition example
        let mut parser = Parser::new("(define square (lambda (x) (* x x)))".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert_eq!(
            format!("{}", expr.expr),
            "(define square (lambda (x) (* x x)))"
        );

        // Data structure example
        let mut parser = Parser::new("'(the quick brown fox)".to_string()).unwrap();
        let expr = parser.parse_expression().unwrap();
        assert_eq!(format!("{}", expr.expr), "'(the quick brown fox)");
    }
}
