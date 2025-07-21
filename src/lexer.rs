//! Lexical analysis module for the Twine Scheme interpreter.
//!
//! This module provides tokenization of Scheme source code, converting raw text
//! into a stream of tokens that can be consumed by the parser. It includes
//! position tracking for error reporting and supports all essential Scheme tokens.

use crate::Result;

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
/// This enum represents all the different types of tokens that can appear in
/// Scheme source code, following the R7RS-small specification subset.
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

/// Lexical analyzer for Scheme source code.
///
/// The lexer converts raw source code into a stream of tokens with position
/// information for error reporting. It handles comments, whitespace, and all
/// essential Scheme syntax elements.
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given input string.
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get the current position in the source.
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    /// Check if we've reached the end of input.
    pub fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Peek at the current character without consuming it.
    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// Peek at the next character without consuming it.
    pub fn peek_next(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    /// Advance to the next character and return it.
    pub fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.input.chars().nth(self.position) {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    /// Skip whitespace characters.
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a comment from semicolon to end of line.
    pub fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Get the next token from the input stream.
    pub fn next_token(&mut self) -> Result<PositionedToken> {
        self.skip_whitespace();

        let position = self.current_position();

        if self.is_at_end() {
            return Ok(PositionedToken::new(Token::Eof, position));
        }

        // Skip comments
        if self.peek() == Some(';') {
            self.skip_comment();
            return self.next_token(); // Recurse to get the next non-comment token
        }

        match self.peek().unwrap() {
            // Delimiters
            '(' => {
                self.advance();
                Ok(PositionedToken::new(Token::LeftParen, position))
            }
            ')' => {
                self.advance();
                Ok(PositionedToken::new(Token::RightParen, position))
            }
            '\'' => {
                self.advance();
                Ok(PositionedToken::new(Token::Quote, position))
            }

            // String literals
            '"' => self.read_string(position),

            // Numbers and symbols starting with digits or signs
            ch if ch.is_ascii_digit() || ch == '+' || ch == '-' => {
                self.read_number_or_symbol(position)
            }

            // Boolean literals
            '#' => self.read_boolean(position),

            // Symbols and keywords
            ch if self.is_symbol_start_char(ch) => self.read_symbol(position),

            // Unexpected character
            ch => {
                use crate::Error;
                Err(Error::syntax_error(
                    &format!("Unexpected character '{}'", ch),
                    position.line,
                    position.column,
                ))
            }
        }
    }

    /// Check if a character can start a symbol.
    fn is_symbol_start_char(&self, ch: char) -> bool {
        ch.is_alphabetic() || "!$%&*+-./:<=>?@^_~".contains(ch)
    }

    /// Check if a character can continue a symbol.
    fn is_symbol_char(&self, ch: char) -> bool {
        ch.is_alphanumeric() || "!$%&*+-./:<=>?@^_~".contains(ch)
    }

    /// Read a string literal with escape sequence support.
    fn read_string(&mut self, position: Position) -> Result<PositionedToken> {
        use crate::Error;

        self.advance(); // consume opening quote
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // consume closing quote
                return Ok(PositionedToken::new(Token::String(value), position));
            }

            if ch == '\\' {
                self.advance(); // consume backslash
                match self.peek() {
                    Some('n') => {
                        value.push('\n');
                        self.advance();
                    }
                    Some('t') => {
                        value.push('\t');
                        self.advance();
                    }
                    Some('r') => {
                        value.push('\r');
                        self.advance();
                    }
                    Some('\\') => {
                        value.push('\\');
                        self.advance();
                    }
                    Some('"') => {
                        value.push('"');
                        self.advance();
                    }
                    Some(escaped) => {
                        return Err(Error::syntax_error(
                            &format!("Invalid escape sequence '\\{}'", escaped),
                            self.line,
                            self.column,
                        ));
                    }
                    None => {
                        return Err(Error::syntax_error(
                            "Unterminated escape sequence",
                            self.line,
                            self.column,
                        ));
                    }
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(Error::syntax_error(
            "Unterminated string literal",
            position.line,
            position.column,
        ))
    }

    /// Read a number or symbol that starts with a digit, +, or -.
    fn read_number_or_symbol(&mut self, position: Position) -> Result<PositionedToken> {
        let mut text = String::new();
        let start_ch = self.peek().unwrap();

        // Collect the token text
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || "()';\"".contains(ch) {
                break;
            }
            text.push(ch);
            self.advance();
        }

        // Try to parse as number
        if let Ok(num) = text.parse::<f64>() {
            Ok(PositionedToken::new(Token::Number(num), position))
        } else if start_ch == '+' || start_ch == '-' {
            // Single + or - are symbols, not numbers
            Ok(PositionedToken::new(Token::Symbol(text), position))
        } else {
            // Invalid number format, treat as symbol
            Ok(PositionedToken::new(Token::Symbol(text), position))
        }
    }

    /// Read a boolean literal (#t or #f).
    fn read_boolean(&mut self, position: Position) -> Result<PositionedToken> {
        use crate::Error;

        self.advance(); // consume '#'

        match self.peek() {
            Some('t') => {
                self.advance();
                Ok(PositionedToken::new(Token::Boolean(true), position))
            }
            Some('f') => {
                self.advance();
                Ok(PositionedToken::new(Token::Boolean(false), position))
            }
            Some(ch) => Err(Error::syntax_error(
                &format!("Invalid boolean literal '#{}'", ch),
                position.line,
                position.column,
            )),
            None => Err(Error::syntax_error(
                "Incomplete boolean literal",
                position.line,
                position.column,
            )),
        }
    }

    /// Read a symbol.
    fn read_symbol(&mut self, position: Position) -> Result<PositionedToken> {
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || "()';\"".contains(ch) {
                break;
            }
            if !self.is_symbol_char(ch) {
                break;
            }
            text.push(ch);
            self.advance();
        }

        Ok(PositionedToken::new(Token::Symbol(text), position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        // Test creating different token types
        let left_paren = Token::LeftParen;
        let right_paren = Token::RightParen;
        let quote = Token::Quote;
        let number = Token::Number(42.0);
        let string = Token::String("hello".to_string());
        let symbol = Token::Symbol("foo".to_string());
        let boolean_true = Token::Boolean(true);
        let boolean_false = Token::Boolean(false);
        let eof = Token::Eof;

        // Verify tokens can be created
        assert!(matches!(left_paren, Token::LeftParen));
        assert!(matches!(right_paren, Token::RightParen));
        assert!(matches!(quote, Token::Quote));
        assert!(matches!(number, Token::Number(42.0)));
        assert!(matches!(string, Token::String(_)));
        assert!(matches!(symbol, Token::Symbol(_)));
        assert!(matches!(boolean_true, Token::Boolean(true)));
        assert!(matches!(boolean_false, Token::Boolean(false)));
        assert!(matches!(eof, Token::Eof));
    }

    #[test]
    fn test_token_debug_output() {
        let number = Token::Number(42.5);
        let string = Token::String("test".to_string());
        let symbol = Token::Symbol("foo".to_string());
        let boolean = Token::Boolean(true);

        // Verify Debug formatting works
        let debug_number = format!("{:?}", number);
        let debug_string = format!("{:?}", string);
        let debug_symbol = format!("{:?}", symbol);
        let debug_boolean = format!("{:?}", boolean);

        assert!(debug_number.contains("Number"));
        assert!(debug_number.contains("42.5"));
        assert!(debug_string.contains("String"));
        assert!(debug_string.contains("test"));
        assert!(debug_symbol.contains("Symbol"));
        assert!(debug_symbol.contains("foo"));
        assert!(debug_boolean.contains("Boolean"));
        assert!(debug_boolean.contains("true"));
    }

    #[test]
    fn test_token_equality() {
        // Test that identical tokens are equal
        assert_eq!(Token::LeftParen, Token::LeftParen);
        assert_eq!(Token::RightParen, Token::RightParen);
        assert_eq!(Token::Quote, Token::Quote);
        assert_eq!(Token::Eof, Token::Eof);
        assert_eq!(Token::Number(42.0), Token::Number(42.0));
        assert_eq!(
            Token::String("hello".to_string()),
            Token::String("hello".to_string())
        );
        assert_eq!(
            Token::Symbol("foo".to_string()),
            Token::Symbol("foo".to_string())
        );
        assert_eq!(Token::Boolean(true), Token::Boolean(true));
        assert_eq!(Token::Boolean(false), Token::Boolean(false));

        // Test that different tokens are not equal
        assert_ne!(Token::LeftParen, Token::RightParen);
        assert_ne!(Token::Number(42.0), Token::Number(43.0));
        assert_ne!(
            Token::String("hello".to_string()),
            Token::String("world".to_string())
        );
        assert_ne!(
            Token::Symbol("foo".to_string()),
            Token::Symbol("bar".to_string())
        );
        assert_ne!(Token::Boolean(true), Token::Boolean(false));
    }

    #[test]
    fn test_position_tracking() {
        // Test Position creation and methods
        let start_pos = Position::start();
        assert_eq!(start_pos.line, 1);
        assert_eq!(start_pos.column, 1);

        let custom_pos = Position::new(5, 10);
        assert_eq!(custom_pos.line, 5);
        assert_eq!(custom_pos.column, 10);

        // Test Position equality
        assert_eq!(Position::new(1, 1), Position::start());
        assert_ne!(Position::new(2, 1), Position::start());

        // Test PositionedToken creation
        let token = Token::Number(42.0);
        let position = Position::new(3, 7);
        let positioned = PositionedToken::new(token.clone(), position.clone());

        assert_eq!(positioned.token, token);
        assert_eq!(positioned.position, position);

        // Test Debug formatting for positioned tokens
        let debug_output = format!("{:?}", positioned);
        assert!(debug_output.contains("PositionedToken"));
        assert!(debug_output.contains("Number"));
        assert!(debug_output.contains("42"));
        assert!(debug_output.contains("line: 3"));
        assert!(debug_output.contains("column: 7"));
    }

    #[test]
    fn test_token_type_checking() {
        // Test token type checking methods
        assert!(Token::LeftParen.is_delimiter());
        assert!(Token::RightParen.is_delimiter());
        assert!(Token::Quote.is_delimiter());
        assert!(!Token::Number(42.0).is_delimiter());
        assert!(!Token::String("test".to_string()).is_delimiter());

        assert!(Token::Number(42.0).is_literal());
        assert!(Token::String("test".to_string()).is_literal());
        assert!(Token::Symbol("foo".to_string()).is_literal());
        assert!(Token::Boolean(true).is_literal());
        assert!(!Token::LeftParen.is_literal());
        assert!(!Token::Eof.is_literal());

        assert!(Token::Eof.is_eof());
        assert!(!Token::LeftParen.is_eof());
        assert!(!Token::Number(42.0).is_eof());
    }

    #[test]
    fn test_lexer_creation() {
        let input = "test input".to_string();
        let lexer = Lexer::new(input.clone());

        assert_eq!(lexer.input, input);
        assert_eq!(lexer.position, 0);
        assert_eq!(lexer.line, 1);
        assert_eq!(lexer.column, 1);
        assert!(!lexer.is_at_end());
    }

    #[test]
    fn test_lexer_position_tracking() {
        let mut lexer = Lexer::new("hello\nworld".to_string());

        // Test initial position
        assert_eq!(lexer.current_position(), Position::new(1, 1));

        // Advance through "hello"
        assert_eq!(lexer.advance(), Some('h'));
        assert_eq!(lexer.current_position(), Position::new(1, 2));

        assert_eq!(lexer.advance(), Some('e'));
        assert_eq!(lexer.current_position(), Position::new(1, 3));

        assert_eq!(lexer.advance(), Some('l'));
        assert_eq!(lexer.current_position(), Position::new(1, 4));

        assert_eq!(lexer.advance(), Some('l'));
        assert_eq!(lexer.current_position(), Position::new(1, 5));

        assert_eq!(lexer.advance(), Some('o'));
        assert_eq!(lexer.current_position(), Position::new(1, 6));

        // Advance through newline
        assert_eq!(lexer.advance(), Some('\n'));
        assert_eq!(lexer.current_position(), Position::new(2, 1));

        // Advance through "world"
        assert_eq!(lexer.advance(), Some('w'));
        assert_eq!(lexer.current_position(), Position::new(2, 2));
    }

    #[test]
    fn test_lexer_peek_operations() {
        let mut lexer = Lexer::new("abc".to_string());

        // Test peek without advancing
        assert_eq!(lexer.peek(), Some('a'));
        assert_eq!(lexer.peek(), Some('a')); // Should still be 'a'
        assert_eq!(lexer.current_position(), Position::new(1, 1));

        // Test peek_next
        assert_eq!(lexer.peek_next(), Some('b'));

        // Advance and test peek again
        lexer.advance();
        assert_eq!(lexer.peek(), Some('b'));
        assert_eq!(lexer.peek_next(), Some('c'));

        // Advance to end
        lexer.advance();
        lexer.advance();
        assert_eq!(lexer.peek(), None);
        assert_eq!(lexer.peek_next(), None);
        assert!(lexer.is_at_end());
    }

    #[test]
    fn test_lexer_whitespace_handling() {
        let mut lexer = Lexer::new("  \t\n  hello".to_string());

        // Test skipping whitespace
        lexer.skip_whitespace();
        assert_eq!(lexer.peek(), Some('h'));
        assert_eq!(lexer.current_position(), Position::new(2, 3));
    }

    #[test]
    fn test_lexer_comment_handling() {
        let mut lexer = Lexer::new("; this is a comment\nhello".to_string());

        // Skip the comment
        lexer.skip_comment();
        assert_eq!(lexer.peek(), Some('\n'));

        // Advance past newline and check we're at 'hello'
        lexer.advance();
        assert_eq!(lexer.peek(), Some('h'));
    }

    #[test]
    fn test_lexer_basic_tokenization() {
        let mut lexer = Lexer::new("   ; comment\n  ".to_string());

        // Should return EOF after skipping whitespace and comments
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Eof);
        assert_eq!(token.position.line, 2);
        assert_eq!(token.position.column, 3);
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("".to_string());

        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Eof);
        assert_eq!(token.position, Position::start());
    }

    #[test]
    fn test_number_tokenization() {
        // Test integer parsing
        let mut lexer = Lexer::new("42".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Number(42.0));
        assert_eq!(token.position, Position::new(1, 1));

        // Test floating-point parsing
        let mut lexer = Lexer::new("3.14159".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Number(3.14159));

        // Test negative numbers
        let mut lexer = Lexer::new("-123".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Number(-123.0));

        // Test positive numbers with explicit sign
        let mut lexer = Lexer::new("+456".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Number(456.0));

        // Test zero
        let mut lexer = Lexer::new("0".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Number(0.0));

        // Test decimal starting with zero
        let mut lexer = Lexer::new("0.5".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Number(0.5));
    }

    #[test]
    fn test_string_tokenization() {
        // Test simple string
        let mut lexer = Lexer::new("\"hello\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::String("hello".to_string()));
        assert_eq!(token.position, Position::new(1, 1));

        // Test empty string
        let mut lexer = Lexer::new("\"\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::String("".to_string()));

        // Test string with escape sequences
        let mut lexer = Lexer::new("\"hello\\nworld\\t!\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::String("hello\nworld\t!".to_string()));

        // Test string with escaped quote
        let mut lexer = Lexer::new("\"say \\\"hello\\\"\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::String("say \"hello\"".to_string()));

        // Test string with backslash
        let mut lexer = Lexer::new("\"path\\\\to\\\\file\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::String("path\\to\\file".to_string()));
    }

    #[test]
    fn test_symbol_tokenization() {
        // Test simple symbol
        let mut lexer = Lexer::new("hello".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("hello".to_string()));
        assert_eq!(token.position, Position::new(1, 1));

        // Test symbol with special characters
        let mut lexer = Lexer::new("list->vector".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("list->vector".to_string()));

        // Test symbol with numbers
        let mut lexer = Lexer::new("x42".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("x42".to_string()));

        // Test single character symbols
        let mut lexer = Lexer::new("+".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("+".to_string()));

        let mut lexer = Lexer::new("*".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("*".to_string()));

        // Test predicate symbol
        let mut lexer = Lexer::new("null?".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("null?".to_string()));
    }

    #[test]
    fn test_boolean_tokenization() {
        // Test true
        let mut lexer = Lexer::new("#t".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Boolean(true));
        assert_eq!(token.position, Position::new(1, 1));

        // Test false
        let mut lexer = Lexer::new("#f".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Boolean(false));
        assert_eq!(token.position, Position::new(1, 1));
    }

    #[test]
    fn test_delimiter_tokenization() {
        // Test left parenthesis
        let mut lexer = Lexer::new("(".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::LeftParen);
        assert_eq!(token.position, Position::new(1, 1));

        // Test right parenthesis
        let mut lexer = Lexer::new(")".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::RightParen);
        assert_eq!(token.position, Position::new(1, 1));

        // Test quote
        let mut lexer = Lexer::new("'".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Quote);
        assert_eq!(token.position, Position::new(1, 1));
    }

    #[test]
    fn test_complex_tokenization() {
        let mut lexer = Lexer::new("(+ 1 (* 2 3))".to_string());

        let tokens: Vec<_> = std::iter::from_fn(|| match lexer.next_token() {
            Ok(token) if token.token != Token::Eof => Some(token.token),
            _ => None,
        })
        .collect();

        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("+".to_string()),
                Token::Number(1.0),
                Token::LeftParen,
                Token::Symbol("*".to_string()),
                Token::Number(2.0),
                Token::Number(3.0),
                Token::RightParen,
                Token::RightParen,
            ]
        );
    }

    #[test]
    fn test_string_with_whitespace() {
        let mut lexer = Lexer::new("\"hello world\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::String("hello world".to_string()));
    }

    #[test]
    fn test_quoted_expression() {
        let mut lexer = Lexer::new("'(a b c)".to_string());

        let quote_token = lexer.next_token().unwrap();
        assert_eq!(quote_token.token, Token::Quote);

        let paren_token = lexer.next_token().unwrap();
        assert_eq!(paren_token.token, Token::LeftParen);
    }

    #[test]
    fn test_error_handling() {
        // Test unterminated string
        let mut lexer = Lexer::new("\"unterminated".to_string());
        let result = lexer.next_token();
        assert!(result.is_err());

        // Test invalid escape sequence
        let mut lexer = Lexer::new("\"invalid\\x\"".to_string());
        let result = lexer.next_token();
        assert!(result.is_err());

        // Test invalid boolean
        let mut lexer = Lexer::new("#x".to_string());
        let result = lexer.next_token();
        assert!(result.is_err());

        // Test incomplete boolean
        let mut lexer = Lexer::new("#".to_string());
        let result = lexer.next_token();
        assert!(result.is_err());
    }

    #[test]
    fn test_real_scheme_expression() {
        // Test tokenizing a realistic Scheme expression
        let mut lexer = Lexer::new(
            "(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))".to_string(),
        );

        let mut tokens = Vec::new();
        loop {
            let token = lexer.next_token().unwrap();
            if token.token == Token::Eof {
                break;
            }
            tokens.push(token.token);
        }

        // Verify the complete tokenization
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("define".to_string()),
                Token::Symbol("factorial".to_string()),
                Token::LeftParen,
                Token::Symbol("lambda".to_string()),
                Token::LeftParen,
                Token::Symbol("n".to_string()),
                Token::RightParen,
                Token::LeftParen,
                Token::Symbol("if".to_string()),
                Token::LeftParen,
                Token::Symbol("=".to_string()),
                Token::Symbol("n".to_string()),
                Token::Number(0.0),
                Token::RightParen,
                Token::Number(1.0),
                Token::LeftParen,
                Token::Symbol("*".to_string()),
                Token::Symbol("n".to_string()),
                Token::LeftParen,
                Token::Symbol("factorial".to_string()),
                Token::LeftParen,
                Token::Symbol("-".to_string()),
                Token::Symbol("n".to_string()),
                Token::Number(1.0),
                Token::RightParen,
                Token::RightParen,
                Token::RightParen,
                Token::RightParen,
                Token::RightParen,
                Token::RightParen,
            ]
        );
    }

    #[test]
    fn test_r7rs_small_character_support() {
        // Test R7RS-small identifier characters
        // Letters: a-z, A-Z
        let mut lexer = Lexer::new("hello WORLD".to_string());
        let token1 = lexer.next_token().unwrap();
        assert_eq!(token1.token, Token::Symbol("hello".to_string()));
        let token2 = lexer.next_token().unwrap();
        assert_eq!(token2.token, Token::Symbol("WORLD".to_string()));

        // Digits in identifiers (not at start)
        let mut lexer = Lexer::new("var123".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("var123".to_string()));

        // Special characters: ! $ % & * + - . / : < = > ? @ ^ _ ~
        let special_chars = "! $ % & * + - . / : < = > ? @ ^ _ ~";
        for ch in special_chars.chars() {
            if ch == ' ' {
                continue;
            } // Skip spaces
            let mut lexer = Lexer::new(ch.to_string());
            let token = lexer.next_token().unwrap();
            assert_eq!(token.token, Token::Symbol(ch.to_string()));
        }

        // Test composite identifiers with special characters
        let mut lexer = Lexer::new("list->vector".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("list->vector".to_string()));

        let mut lexer = Lexer::new("number?".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::Symbol("number?".to_string()));
    }

    #[test]
    fn test_ascii_string_content() {
        // Test that strings can contain any characters (including Unicode)
        // but identifiers follow R7RS-small rules
        let mut lexer = Lexer::new("\"Hello, 世界! 🌍\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.token, Token::String("Hello, 世界! 🌍".to_string()));
    }
}
