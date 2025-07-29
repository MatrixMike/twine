//! Test binary for subprocess-based IO integration tests.
//!
//! This binary evaluates Scheme expressions passed as command line arguments
//! and writes output to stdout, allowing integration tests to capture and
//! verify the actual stdout output using subprocess execution.

use std::env;
use std::process;
use twine_scheme::parser::Parser;
use twine_scheme::runtime::Environment;
use twine_scheme::runtime::eval::eval;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <scheme-expression>", args[0]);
        process::exit(1);
    }

    let source = &args[1];
    let mut env = Environment::new();

    // Parse and evaluate multiple expressions in sequence
    match Parser::new(source.to_string()) {
        Ok(mut parser) => {
            while !parser.is_at_end() {
                match parser.parse_expression() {
                    Ok(parse_result) => {
                        match eval(parse_result.expr, &mut env) {
                            Ok(_) => {
                                // Expression evaluated successfully
                                // Continue to next expression
                            }
                            Err(e) => {
                                eprintln!("Error: {e}");
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
