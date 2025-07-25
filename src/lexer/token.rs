//! Token types for the lexical analyzer.
//!
//! Defines the core token types and position tracking used throughout
//! the lexical analysis phase.

/// Position information for tracking token locations in source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    /// Create a new position at the given line and column.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create a position at the start of the source (line 1, column 1).
    pub fn start() -> Self {
        Self::new(1, 1)
    }
}

/// A token with its position in the source code.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedToken {
    pub token: Token,
    pub position: Position,
}

impl PositionedToken {
    /// Create a new positioned token.
    pub fn new(token: Token, position: Position) -> Self {
        Self { token, position }
    }
}

/// Lexical tokens for Scheme source code.
///
/// Represents all token types that can appear in Scheme source code.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Delimiters
    /// Left parenthesis '('
    LeftParen,
    /// Right parenthesis ')'
    RightParen,
    /// Quote character '\''
    Quote,

    // Literals
    /// Numeric literal (integers and floating-point)
    Number(f64),
    /// String literal with escape sequences
    String(String),
    /// Symbol identifier
    Symbol(String),
    /// Boolean literal (#t or #f)
    Boolean(bool),

    // Control
    /// End of file marker
    Eof,
}

impl Token {
    /// Check if this token is a delimiter.
    pub fn is_delimiter(&self) -> bool {
        matches!(self, Token::LeftParen | Token::RightParen | Token::Quote)
    }

    /// Check if this token is a literal value.
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::Number(_) | Token::String(_) | Token::Symbol(_) | Token::Boolean(_)
        )
    }

    /// Check if this token indicates end of input.
    pub fn is_eof(&self) -> bool {
        matches!(self, Token::Eof)
    }
}
