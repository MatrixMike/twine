//! Binding and definition special forms
//!
//! This module implements binding special forms like `define` and `let`.

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::Value;

/// Evaluate a define special form
///
/// Syntax:
/// - Binding definition: (define <identifier> <expression>)
/// - Procedure definition: (define (<identifier> <param>...) <body>...)
///
/// For binding definition:
/// - Evaluates <expression> and binds the result to <identifier> in the current environment
/// - Returns Nil
///
/// For procedure definition (syntactic sugar):
/// - (define (name param1 param2) body1 body2...)
/// - Equivalent to: (define name (lambda (param1 param2) body1 body2...))
pub fn eval_define(mut args: Vec<Expression>, env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::Error::arity_error("define", 1, 0));
    }

    let first_arg = args.remove(0);
    match first_arg {
        // Binding definition: (define identifier expression)
        Expression::Atom(Value::Symbol(identifier)) => {
            if args.len() != 1 {
                return Err(crate::Error::arity_error("define", 2, args.len() + 1));
            }

            // Evaluate the expression and bind it to the identifier
            let value_expr = args.into_iter().next().unwrap();
            let value = eval(value_expr, env)?;
            env.define(identifier, value);
            Ok(Value::Nil)
        }

        // Procedure definition: (define (name param...) body...)
        Expression::List(elements) => {
            if elements.is_empty() {
                return Err(crate::Error::runtime_error(
                    "define: procedure definition requires non-empty parameter list",
                ));
            }

            // Extract procedure name and parameters
            let procedure_name = match &elements[0] {
                Expression::Atom(Value::Symbol(name)) => name.clone(),
                _ => {
                    return Err(crate::Error::runtime_error(
                        "define: procedure name must be a symbol",
                    ));
                }
            };

            // Extract parameters - all must be symbols
            let mut parameters = Vec::new();
            for param_expr in &elements[1..] {
                match param_expr {
                    Expression::Atom(Value::Symbol(param)) => {
                        parameters.push(param.clone());
                    }
                    _ => {
                        return Err(crate::Error::runtime_error(
                            "define: procedure parameters must be symbols",
                        ));
                    }
                }
            }

            // Procedure body is everything remaining in args
            if args.is_empty() {
                return Err(crate::Error::runtime_error(
                    "define: procedure definition requires at least one body expression",
                ));
            }

            let body_expressions = args;

            // Create a lambda expression: (lambda (param...) body...)
            let mut lambda_args = vec![Expression::List(
                parameters
                    .into_iter()
                    .map(|p| Expression::atom(Value::Symbol(p)))
                    .collect(),
            )];
            lambda_args.extend(body_expressions);

            // For now, we'll store a placeholder since lambda isn't implemented yet
            // This will be updated when T3.1.2 implements lambda
            // TODO: Replace with actual lambda creation when lambda is implemented
            let placeholder_value =
                Value::string(&format!("<procedure:{}>", procedure_name.as_str()));

            env.define(procedure_name, placeholder_value);
            Ok(Value::Nil)
        }

        _ => Err(crate::Error::runtime_error(
            "define: first argument must be a symbol or parameter list",
        )),
    }
}

/// Evaluate a let special form
///
/// Syntax: (let ((id1 expr1) (id2 expr2) ...) body1 body2 ...)
///
/// Semantics:
/// 1. Evaluate all expressions (expr1, expr2, ...) in the current environment
/// 2. Create a new environment with current environment as parent
/// 3. Bind identifiers (id1, id2, ...) to their evaluated values simultaneously
/// 4. Evaluate body expressions sequentially in the new environment
/// 5. Return the value of the last body expression
pub fn eval_let(mut args: Vec<Expression>, env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::Error::arity_error("let", 1, 0));
    }

    // First argument must be the binding list
    let bindings_expr = args.remove(0);
    let body_exprs = args;

    if body_exprs.is_empty() {
        return Err(crate::Error::runtime_error(
            "let: requires at least one body expression",
        ));
    }

    // Parse binding list: ((var1 expr1) (var2 expr2) ...)
    let binding_pairs = match bindings_expr {
        Expression::List(pairs) => pairs,
        _ => {
            return Err(crate::Error::runtime_error(
                "let: first argument must be a list of bindings",
            ));
        }
    };

    // Parse and validate each binding pair
    let mut identifiers = Vec::new();
    let mut expressions = Vec::new();

    for pair in binding_pairs {
        match pair {
            Expression::List(mut elements) => {
                if elements.len() != 2 {
                    return Err(crate::Error::runtime_error(
                        "let: each binding must be a list of exactly 2 elements (identifier expression)",
                    ));
                }

                // First element must be a symbol (identifier name)
                let identifier = match elements.remove(0) {
                    Expression::Atom(Value::Symbol(sym)) => sym,
                    _ => {
                        return Err(crate::Error::runtime_error(
                            "let: binding identifier must be a symbol",
                        ));
                    }
                };

                // Second element is the expression to evaluate
                let expression = elements.remove(0);

                identifiers.push(identifier);
                expressions.push(expression);
            }
            _ => {
                return Err(crate::Error::runtime_error(
                    "let: each binding must be a list",
                ));
            }
        }
    }

    // Evaluate all expressions in the current environment BEFORE creating bindings
    let mut values = Vec::new();
    for expr in expressions {
        let value = eval(expr, env)?;
        values.push(value);
    }

    // Create new environment with current environment as parent
    let mut let_env = Environment::new_scope(env);

    // Bind all identifiers simultaneously in the new environment
    for (identifier, value) in identifiers.into_iter().zip(values.into_iter()) {
        let_env.define(identifier, value);
    }

    // Evaluate body expressions sequentially in the new environment
    // Evaluate the body expressions in sequence
    let mut result = Value::Nil;
    for body_expr in body_exprs {
        result = eval(body_expr, &mut let_env)?;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Expression;
    use crate::runtime::environment::Environment;
    use crate::types::{Symbol, Value};

    #[test]
    fn test_eval_define_binding() {
        let mut env = Environment::new();

        // Test simple binding definition
        let args = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ];

        let result = eval_define(args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));
    }

    #[test]
    fn test_eval_define_binding_with_expression() {
        let mut env = Environment::new();

        // Define an identifier first
        env.define(Symbol::new("y"), Value::number(10.0));

        // Test defining with an expression that references another identifier
        let args = vec![
            Expression::atom(Value::symbol("z")),
            Expression::atom(Value::symbol("y")), // Should evaluate to 10.0
        ];

        let result = eval_define(args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created with the evaluated value
        assert_eq!(env.lookup(&Symbol::new("z")).unwrap(), Value::number(10.0));
    }

    #[test]
    fn test_eval_define_procedure_syntax() {
        let mut env = Environment::new();

        // Test procedure definition syntax: (define (square x) (* x x))
        let procedure_def = Expression::List(vec![
            Expression::atom(Value::symbol("square")),
            Expression::atom(Value::symbol("x")),
        ]);

        let body = Expression::List(vec![
            Expression::atom(Value::symbol("*")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("x")),
        ]);

        let args = vec![procedure_def, body];

        let result = eval_define(args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the procedure binding was created (placeholder for now)
        let binding = env.lookup(&Symbol::new("square")).unwrap();
        assert!(binding.as_string().unwrap().contains("procedure:square"));
    }

    #[test]
    fn test_eval_define_procedure_multiple_params() {
        let mut env = Environment::new();

        // Test procedure with multiple parameters: (define (add x y) (+ x y))
        let procedure_def = Expression::List(vec![
            Expression::atom(Value::symbol("add")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
        ]);

        let body = Expression::List(vec![
            Expression::atom(Value::symbol("+")),
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("y")),
        ]);

        let args = vec![procedure_def, body];

        let result = eval_define(args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the procedure binding was created
        let binding = env.lookup(&Symbol::new("add")).unwrap();
        assert!(binding.as_string().unwrap().contains("procedure:add"));
    }

    #[test]
    fn test_eval_define_procedure_multiple_body_expressions() {
        let mut env = Environment::new();

        // Test procedure with multiple body expressions
        let procedure_def = Expression::List(vec![Expression::atom(Value::symbol("test-fn"))]);

        let body1 = Expression::atom(Value::number(1.0));
        let body2 = Expression::atom(Value::number(2.0));

        let args = vec![procedure_def, body1, body2];

        let result = eval_define(args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the procedure binding was created
        let binding = env.lookup(&Symbol::new("test-fn")).unwrap();
        assert!(binding.as_string().unwrap().contains("procedure:test-fn"));
    }

    #[test]
    fn test_eval_define_errors() {
        let mut env = Environment::new();

        // Test no arguments
        let result = eval_define(vec![], &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("define: expected 1 argument, got 0")
        );

        // Test binding definition with wrong arity
        let args = vec![Expression::atom(Value::symbol("x"))];
        let result = eval_define(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("define: expected 2 arguments")
        );

        // Test procedure definition with non-symbol name
        let procedure_def = Expression::List(vec![
            Expression::atom(Value::number(42.0)), // Not a symbol
        ]);
        let args = vec![procedure_def];
        let result = eval_define(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("procedure name must be a symbol")
        );

        // Test procedure definition with non-symbol parameter
        let procedure_def = Expression::List(vec![
            Expression::atom(Value::symbol("fn")),
            Expression::atom(Value::number(42.0)), // Parameter must be symbol
        ]);
        let args = vec![procedure_def];
        let result = eval_define(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("procedure parameters must be symbols")
        );

        // Test procedure definition with empty parameter list
        let procedure_def = Expression::List(vec![]);
        let args = vec![procedure_def];
        let result = eval_define(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires non-empty parameter list")
        );

        // Test procedure definition without body
        let procedure_def = Expression::List(vec![Expression::atom(Value::symbol("fn"))]);
        let args = vec![procedure_def];
        let result = eval_define(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("procedure definition requires at least one body expression")
        );

        // Test invalid first argument
        let args = vec![Expression::atom(Value::number(42.0))];
        let result = eval_define(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("first argument must be a symbol or parameter list")
        );
    }

    #[test]
    fn test_eval_define_binding_shadowing() {
        let mut env = Environment::new();

        // Define an identifier
        let args1 = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ];
        eval_define(args1, &mut env).unwrap();
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));

        // Redefine the same identifier (should shadow)
        let args2 = vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::string("hello")),
        ];
        eval_define(args2, &mut env).unwrap();
        assert_eq!(
            env.lookup(&Symbol::new("x")).unwrap(),
            Value::string("hello")
        );
    }

    #[test]
    fn test_eval_let_basic() {
        let mut env = Environment::new();

        // Test basic let: (let ((x 42)) x)
        let bindings = Expression::List(vec![Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ])]);
        let body = Expression::atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));

        // Verify the original environment is unchanged
        assert!(env.lookup(&Symbol::new("x")).is_err());
    }

    #[test]
    fn test_eval_let_multiple_bindings() {
        let mut env = Environment::new();

        // Test multiple bindings: (let ((x 10) (y 20)) (+ x y))
        let bindings = Expression::List(vec![
            Expression::List(vec![
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::number(10.0)),
            ]),
            Expression::List(vec![
                Expression::atom(Value::symbol("y")),
                Expression::atom(Value::number(20.0)),
            ]),
        ]);

        // Body: (+ x y) - but since + isn't implemented yet, just return y
        let body = Expression::atom(Value::symbol("y"));

        let args = vec![bindings, body];
        let result = eval_let(args, &mut env).unwrap();
        assert_eq!(result, Value::number(20.0));
    }

    #[test]
    fn test_eval_let_multiple_body_expressions() {
        let mut env = Environment::new();

        // Test multiple body expressions: (let ((x 5)) 1 2 x)
        let bindings = Expression::List(vec![Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(5.0)),
        ])]);

        let body1 = Expression::atom(Value::number(1.0));
        let body2 = Expression::atom(Value::number(2.0));
        let body3 = Expression::atom(Value::symbol("x"));

        let args = vec![bindings, body1, body2, body3];
        let result = eval_let(args, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0)); // Should return last expression
    }

    #[test]
    fn test_eval_let_lexical_scoping() {
        let mut env = Environment::new();

        // Define x in outer environment
        env.define(Symbol::new("x"), Value::number(100.0));

        // Test lexical scoping: (let ((x 42)) x)
        // Inner x should shadow outer x
        let bindings = Expression::List(vec![Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::number(42.0)),
        ])]);
        let body = Expression::atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));

        // Verify outer environment is unchanged
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(100.0));
    }

    #[test]
    fn test_eval_let_simultaneous_binding() {
        let mut env = Environment::new();

        // Define x in outer environment
        env.define(Symbol::new("x"), Value::number(10.0));

        // Test simultaneous binding: (let ((x 42) (y x)) y)
        // y should bind to the outer x (10), not the inner x (42)
        let bindings = Expression::List(vec![
            Expression::List(vec![
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::number(42.0)),
            ]),
            Expression::List(vec![
                Expression::atom(Value::symbol("y")),
                Expression::atom(Value::symbol("x")), // This should reference outer x
            ]),
        ]);
        let body = Expression::atom(Value::symbol("y"));

        let args = vec![bindings, body];
        let result = eval_let(args, &mut env).unwrap();
        assert_eq!(result, Value::number(10.0)); // y should be 10 (outer x), not 42
    }

    #[test]
    fn test_eval_let_empty_bindings() {
        let mut env = Environment::new();

        // Test empty bindings: (let () 42)
        let bindings = Expression::List(vec![]);
        let body = Expression::atom(Value::number(42.0));

        let args = vec![bindings, body];
        let result = eval_let(args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_let_with_expressions() {
        let mut env = Environment::new();

        // Define an identifier to use in binding expressions
        env.define(Symbol::new("a"), Value::number(5.0));

        // Test: (let ((x a) (y 10)) y)
        let bindings = Expression::List(vec![
            Expression::List(vec![
                Expression::atom(Value::symbol("x")),
                Expression::atom(Value::symbol("a")), // Should evaluate to 5
            ]),
            Expression::List(vec![
                Expression::atom(Value::symbol("y")),
                Expression::atom(Value::number(10.0)),
            ]),
        ]);
        let body = Expression::atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(args, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0));
    }

    #[test]
    fn test_eval_let_errors() {
        let mut env = Environment::new();

        // Test no arguments
        let result = eval_let(vec![], &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: expected 1 argument, got 0")
        );

        // Test no body expressions
        let bindings = Expression::List(vec![]);
        let args = vec![bindings];
        let result = eval_let(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: requires at least one body expression")
        );

        // Test non-list bindings
        let bindings = Expression::atom(Value::number(42.0));
        let body = Expression::atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: first argument must be a list of bindings")
        );

        // Test invalid binding format (not a list)
        let bindings = Expression::List(vec![Expression::atom(Value::number(42.0))]);
        let body = Expression::atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: each binding must be a list")
        );

        // Test binding with wrong arity
        let bindings = Expression::List(vec![Expression::List(vec![Expression::atom(
            Value::symbol("x"),
        )])]);
        let body = Expression::atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: each binding must be a list of exactly 2 elements")
        );

        // Test binding with non-symbol identifier
        let bindings = Expression::List(vec![Expression::List(vec![
            Expression::atom(Value::number(42.0)), // Not a symbol
            Expression::atom(Value::number(1.0)),
        ])]);
        let body = Expression::atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: binding identifier must be a symbol")
        );
    }

    #[test]
    fn test_eval_let_unbound_identifier_in_binding() {
        let mut env = Environment::new();

        // Test evaluation error in binding expression
        let bindings = Expression::List(vec![Expression::List(vec![
            Expression::atom(Value::symbol("x")),
            Expression::atom(Value::symbol("undefined")), // Unbound identifier
        ])]);
        let body = Expression::atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unbound identifier: 'undefined'")
        );
    }
}
