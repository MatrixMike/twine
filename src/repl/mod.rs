//! Interactive REPL (Read-Eval-Print Loop) for the Twine Scheme interpreter.
//!
//! Provides an enhanced command-line interface with multi-line input support
//! using standard I/O without external dependencies.

use std::io::{self, Write};

use crate::{
    Error,
    lexer::{Lexer, Token},
    parser::Parser,
    runtime::{Environment, eval},
    types::Value,
};

/// REPL configuration and state management
pub struct Repl {
    env: Environment<'static>,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    /// Run the interactive REPL loop
    pub fn run(&mut self) -> io::Result<()> {
        println!("Twine Scheme Interpreter");
        println!("Type expressions to evaluate, or Ctrl+C to exit.");
        println!();

        loop {
            match self.read_complete_expression() {
                Ok(Some(input)) => {
                    // Evaluate and print
                    match eval_source(&input, &mut self.env) {
                        Ok(value) => println!("{value}"),
                        Err(error) => eprintln!("Error: {error}"),
                    }
                }
                Ok(None) => {
                    // EOF (Ctrl+D)
                    println!("\nGoodbye!");
                    break;
                }
                Err(error) => {
                    eprintln!("Input error: {error}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Read a complete S-expression with multi-line support
    fn read_complete_expression(&mut self) -> io::Result<Option<String>> {
        let mut input = String::new();
        let mut line_buffer = String::new();

        // Read first line with prompt
        print!("twine> ");
        io::stdout().flush()?;

        line_buffer.clear();
        match io::stdin().read_line(&mut line_buffer)? {
            0 => return Ok(None), // EOF
            _ => input.push_str(&line_buffer),
        }

        let trimmed_input = input.trim();
        if trimmed_input.is_empty() {
            return Ok(Some(String::new()));
        }

        // Check if expression is already complete
        match is_expression_complete(trimmed_input) {
            Ok(true) => return Ok(Some(trimmed_input.to_string())),
            Ok(false) => {} // Continue reading
            Err(_) => return Ok(Some(trimmed_input.to_string())),
        }

        // Continue reading lines for incomplete expressions
        loop {
            line_buffer.clear();
            match io::stdin().read_line(&mut line_buffer)? {
                0 => break, // EOF - treat as completing current expression
                _ => {
                    input.push_str(&line_buffer);
                    let trimmed_input = input.trim();

                    match is_expression_complete(trimmed_input) {
                        Ok(true) => break,
                        Ok(false) => continue,
                        Err(_) => break,
                    }
                }
            }
        }

        let trimmed_input = input.trim();
        if trimmed_input.is_empty() {
            Ok(Some(String::new()))
        } else {
            Ok(Some(trimmed_input.to_string()))
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

/// Determines if an expression is complete by tokenizing and checking bracket balance
fn is_expression_complete(input: &str) -> Result<bool, Error> {
    let mut lexer = Lexer::new(input.to_string());
    let mut paren_count = 0;
    let mut has_content = false;

    loop {
        let positioned_token = lexer.next_token()?;

        match positioned_token.token {
            Token::LeftParen => {
                paren_count += 1;
                has_content = true;
            }
            Token::RightParen => {
                paren_count -= 1;
                has_content = true;

                // If we go negative, the expression is malformed but complete
                // (let the parser handle the error)
                if paren_count < 0 {
                    return Ok(true);
                }
            }
            Token::Eof => {
                break;
            }
            _ => {
                has_content = true;
            }
        }
    }

    // Expression is complete when:
    // 1. All parentheses are balanced (paren_count == 0)
    // 2. We have some content (not just whitespace)
    Ok(paren_count == 0 && has_content)
}

/// Helper function to evaluate source code strings in the REPL context
fn eval_source(source: &str, env: &mut Environment) -> Result<Value, Error> {
    if source.trim().is_empty() {
        return Ok(Value::nil());
    }

    let mut parser = Parser::new(source.to_string())?;
    let expr = parser.parse_expression()?.expr;
    eval(expr, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expressions_complete() {
        assert!(is_expression_complete("42").unwrap());
        assert!(is_expression_complete("hello").unwrap());
        assert!(is_expression_complete("#t").unwrap());
        assert!(is_expression_complete("\"hello world\"").unwrap());
    }

    #[test]
    fn test_balanced_parentheses_complete() {
        assert!(is_expression_complete("(+ 1 2)").unwrap());
        assert!(is_expression_complete("(define x 10)").unwrap());
        assert!(is_expression_complete("(lambda (x) (* x x))").unwrap());
        assert!(is_expression_complete("(if #t 1 2)").unwrap());
    }

    #[test]
    fn test_nested_expressions_complete() {
        assert!(is_expression_complete("(+ (* 2 3) (/ 8 4))").unwrap());
        assert!(is_expression_complete("(define (square x) (* x x))").unwrap());
        assert!(is_expression_complete("((lambda (x) (+ x 1)) 5)").unwrap());
    }

    #[test]
    fn test_unbalanced_parentheses_incomplete() {
        assert!(!is_expression_complete("(+ 1 2").unwrap());
        assert!(!is_expression_complete("(define x").unwrap());
        assert!(!is_expression_complete("((lambda").unwrap());
        assert!(!is_expression_complete("(+ 1 (").unwrap());
    }

    #[test]
    fn test_extra_closing_brackets_complete() {
        // These are malformed but we let the parser handle the error
        assert!(is_expression_complete("(+ 1 2))").unwrap());
        assert!(is_expression_complete(")").unwrap());
    }

    #[test]
    fn test_multiline_expressions() {
        let multiline =
            "(define (factorial n)\n  (if (= n 0)\n      1\n      (* n (factorial (- n 1)))))";
        assert!(is_expression_complete(multiline).unwrap());

        let incomplete_multiline =
            "(define (factorial n)\n  (if (= n 0)\n      1\n      (* n (factorial (- n 1)))";
        assert!(!is_expression_complete(incomplete_multiline).unwrap());
    }

    #[test]
    fn test_comments_ignored() {
        assert!(is_expression_complete("42 ; this is a comment").unwrap());
        assert!(is_expression_complete("(+ 1 2) ; comment with (parentheses)").unwrap());
    }

    #[test]
    fn test_empty_and_whitespace() {
        // Empty input is considered incomplete to avoid evaluating nothing
        assert!(!is_expression_complete("").unwrap());
        assert!(!is_expression_complete("   ").unwrap());
        assert!(!is_expression_complete("\n\n  \t  \n").unwrap());
    }
}
