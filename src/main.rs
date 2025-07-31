use std::io::{self, Write};
use twine_scheme::{
    Error,
    parser::Parser,
    runtime::{Environment, eval},
    types::Value,
};

fn main() {
    println!("Twine Scheme Interpreter");
    println!("Type expressions to evaluate, or Ctrl+C to exit.");
    println!();

    let mut env = Environment::new();

    loop {
        // Print prompt
        print!("twine> ");
        io::stdout().flush().unwrap();

        // Read input
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                let input = input.trim();

                // Skip empty lines
                if input.is_empty() {
                    continue;
                }

                // Evaluate and print
                match eval_source(input, &mut env) {
                    Ok(value) => println!("{value}"),
                    Err(error) => eprintln!("Error: {error}"),
                }
            }
            Err(error) => {
                eprintln!("Input error: {error}");
                break;
            }
        }
    }
}

/// Helper function to evaluate source code strings in the REPL context
fn eval_source(source: &str, env: &mut Environment) -> Result<Value, Error> {
    let mut parser = Parser::new(source.to_string())?;
    let expr = parser.parse_expression()?.expr;
    eval(expr, env)
}
