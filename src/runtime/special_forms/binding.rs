//! Binding and definition special forms
//!
//! This module implements binding special forms like `define` and `let`.

use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::special_forms::lambda::{
    create_lambda_procedure, parse_parameters, validate_parameters,
};
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::{Symbol, Value};
use std::sync::Arc;

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
pub fn eval_define(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::Error::arity_error("define", 2, 0));
    }

    let first_arg = Arc::clone(&args[0]);
    match first_arg.as_ref() {
        // Binding definition: (define identifier expression)
        Expression::Atom(Value::Symbol(identifier)) => {
            eval_define_binding(identifier, &args[1..], env)
        }

        // Procedure definition: (define (name param...) body...)
        Expression::List(param_elements) => eval_define_procedure(param_elements, &args[1..], env),

        _ => Err(crate::Error::runtime_error(
            "define: first argument must be a symbol or parameter list",
        )),
    }
}

/// Handle variable binding: (define identifier expression)
fn eval_define_binding(
    identifier: &Symbol,
    value_exprs: &[Arc<Expression>],
    env: &mut Environment,
) -> Result<Value> {
    if value_exprs.len() != 1 {
        return Err(crate::Error::arity_error(
            "define",
            2,
            value_exprs.len() + 1,
        ));
    }

    // Evaluate the expression and bind it to the identifier
    let value_expr = Arc::clone(&value_exprs[0]);
    let value = eval(value_expr, env)?;
    env.define(identifier.clone(), value);
    Ok(Value::Nil)
}

/// Handle procedure definition: (define (name param...) body...)
fn eval_define_procedure(
    param_elements: &[Arc<Expression>],
    args: &[Arc<Expression>],
    env: &mut Environment,
) -> Result<Value> {
    if param_elements.is_empty() {
        return Err(crate::Error::runtime_error(
            "define: procedure definition requires non-empty parameter list",
        ));
    }

    // Extract procedure name
    let identifier = match param_elements[0].as_ref() {
        Expression::Atom(Value::Symbol(name)) => name.clone(),
        _ => {
            return Err(crate::Error::runtime_error(
                "define: procedure name must be a symbol",
            ));
        }
    };

    // Extract and validate parameters
    let params = parse_parameters(&param_elements[1..])?;
    validate_parameters(&params)?;

    // Validate procedure body
    if args.is_empty() {
        return Err(crate::Error::runtime_error(
            "define: procedure definition requires at least one body expression",
        ));
    }

    // Create body expression - wrap multiple expressions in a list if needed
    let body_expr = if args.len() == 1 {
        Arc::clone(&args[0])
    } else {
        Expression::arc_list(args.iter().map(Arc::clone).collect())
    };

    // Create lambda procedure and bind it to the identifier
    let lambda_proc = create_lambda_procedure(params, body_expr, env);
    env.define(identifier, lambda_proc);
    Ok(Value::Nil)
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
pub fn eval_let(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(crate::Error::arity_error("let", 1, 0));
    }

    // First argument must be the binding list
    let bindings_expr = Arc::clone(&args[0]);
    let body_exprs = &args[1..];

    if body_exprs.is_empty() {
        return Err(crate::Error::runtime_error(
            "let: requires at least one body expression",
        ));
    }

    // Parse binding list: ((var1 expr1) (var2 expr2) ...)
    let binding_pairs = match bindings_expr.as_ref() {
        Expression::List(pairs) => pairs,
        _ => {
            return Err(crate::Error::runtime_error(
                "let: first argument must be a list of bindings",
            ));
        }
    };

    // Parse and validate each binding pair
    let mut identifiers = Vec::with_capacity(binding_pairs.len());
    let mut expressions = Vec::with_capacity(binding_pairs.len());

    for pair in binding_pairs {
        match pair.as_ref() {
            Expression::List(elements) => {
                if elements.len() != 2 {
                    return Err(crate::Error::runtime_error(
                        "let: each binding must be a list of exactly 2 elements (identifier expression)",
                    ));
                }

                // First element must be a symbol (identifier name)
                let identifier = match elements[0].as_ref() {
                    Expression::Atom(Value::Symbol(sym)) => sym.clone(),
                    _ => {
                        return Err(crate::Error::runtime_error(
                            "let: binding identifier must be a symbol",
                        ));
                    }
                };

                // Second element is the expression to evaluate
                let expression = Arc::clone(&elements[1]);

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
    let mut values = Vec::with_capacity(expressions.len());
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

    // Evaluate body expressions sequentially in new environment
    let mut result = Value::Nil;
    for body_expr in body_exprs {
        result = eval(Arc::clone(body_expr), &mut let_env)?;
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
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ];

        let result = eval_define(&args, &mut env).unwrap();
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
            Expression::arc_atom(Value::symbol("z")),
            Expression::arc_atom(Value::symbol("y")), // Should evaluate to 10.0
        ];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the binding was created with the evaluated value
        assert_eq!(env.lookup(&Symbol::new("z")).unwrap(), Value::number(10.0));
    }

    #[test]
    fn test_eval_define_procedure_syntax() {
        let mut env = Environment::new();

        // Test procedure definition syntax: (define (square x) (* x x))
        let procedure_def = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("square")),
            Expression::arc_atom(Value::symbol("x")),
        ]);

        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("*")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("x")),
        ]);

        let args = vec![procedure_def, body];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the procedure binding was created
        let binding = env.lookup(&Symbol::new("square")).unwrap();
        assert!(binding.is_procedure());
        let procedure = binding.as_procedure().unwrap();
        assert!(procedure.is_lambda());
        assert_eq!(procedure.arity(), Some(1));
    }

    #[test]
    fn test_eval_define_procedure_multiple_params() {
        let mut env = Environment::new();

        // Test procedure with multiple parameters: (define (add x y) (+ x y))
        let procedure_def = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("add")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ]);

        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ]);

        let args = vec![procedure_def, body];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the procedure binding was created
        let binding = env.lookup(&Symbol::new("add")).unwrap();
        assert!(binding.is_procedure());
        let procedure = binding.as_procedure().unwrap();
        assert!(procedure.is_lambda());
        assert_eq!(procedure.arity(), Some(2));
    }

    #[test]
    fn test_eval_define_procedure_multiple_body_expressions() {
        let mut env = Environment::new();

        // Test procedure with multiple body expressions
        let procedure_def =
            Expression::arc_list(vec![Expression::arc_atom(Value::symbol("test-fn"))]);

        let body1 = Expression::arc_atom(Value::number(1.0));
        let body2 = Expression::arc_atom(Value::number(2.0));

        let args = vec![procedure_def, body1, body2];

        let result = eval_define(&args, &mut env).unwrap();
        assert_eq!(result, Value::Nil);

        // Verify the procedure binding was created
        let binding = env.lookup(&Symbol::new("test-fn")).unwrap();
        assert!(binding.is_procedure());
        let procedure = binding.as_procedure().unwrap();
        assert!(procedure.is_lambda());
        assert_eq!(procedure.arity(), Some(0));
    }

    #[test]
    fn test_eval_define_errors() {
        let mut env = Environment::new();

        // Test no arguments
        let result = eval_define(&[], &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("define: expected 2 arguments, got 0")
        );

        // Test binding definition with wrong arity
        let args = vec![Expression::arc_atom(Value::symbol("x"))];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("define: expected 2 arguments, got 1")
        );

        // Test procedure definition with non-symbol name
        let procedure_def = Expression::arc_list(vec![
            Expression::arc_atom(Value::number(42.0)), // Not a symbol
        ]);
        let args = vec![procedure_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("procedure name must be a symbol")
        );

        // Test procedure definition with non-symbol parameter
        let procedure_def = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("fn")),
            Expression::arc_atom(Value::number(42.0)), // Parameter must be symbol
        ]);
        let args = vec![procedure_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("parameter must be a symbol")
        );

        // Test procedure definition with empty parameter list
        let procedure_def = Expression::arc_list(vec![]);
        let args = vec![procedure_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("requires non-empty parameter list")
        );

        // Test procedure definition without body
        let procedure_def = Expression::arc_list(vec![Expression::arc_atom(Value::symbol("fn"))]);
        let args = vec![procedure_def];
        let result = eval_define(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("procedure definition requires at least one body expression")
        );

        // Test invalid first argument
        let args = vec![Expression::arc_atom(Value::number(42.0))];
        let result = eval_define(&args, &mut env);
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
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ];
        eval_define(&args1, &mut env).unwrap();
        assert_eq!(env.lookup(&Symbol::new("x")).unwrap(), Value::number(42.0));

        // Redefine the same identifier (should shadow)
        let args2 = vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::string("hello")),
        ];
        eval_define(&args2, &mut env).unwrap();
        assert_eq!(
            env.lookup(&Symbol::new("x")).unwrap(),
            Value::string("hello")
        );
    }

    #[test]
    fn test_eval_let_basic() {
        let mut env = Environment::new();

        // Test basic let: (let ((x 42)) x)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ])]);
        let body = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));

        // Verify the original environment is unchanged
        assert!(env.lookup(&Symbol::new("x")).is_err());
    }

    #[test]
    fn test_eval_let_multiple_bindings() {
        let mut env = Environment::new();

        // Test multiple bindings: (let ((x 10) (y 20)) (+ x y))
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(10.0)),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("y")),
                Expression::arc_atom(Value::number(20.0)),
            ]),
        ]);

        // Body: (+ x y) - but since + isn't implemented yet, just return y
        let body = Expression::arc_atom(Value::symbol("y"));

        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(20.0));
    }

    #[test]
    fn test_eval_let_multiple_body_expressions() {
        let mut env = Environment::new();

        // Test multiple body expressions: (let ((x 5)) 1 2 x)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(5.0)),
        ])]);

        let body1 = Expression::arc_atom(Value::number(1.0));
        let body2 = Expression::arc_atom(Value::number(2.0));
        let body3 = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body1, body2, body3];
        let result = eval_let(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0)); // Should return last expression
    }

    #[test]
    fn test_eval_let_lexical_scoping() {
        let mut env = Environment::new();

        // Define x in outer environment
        env.define(Symbol::new("x"), Value::number(100.0));

        // Test lexical scoping: (let ((x 42)) x)
        // Inner x should shadow outer x
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ])]);
        let body = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env).unwrap();
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
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(42.0)),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("y")),
                Expression::arc_atom(Value::symbol("x")), // This should reference outer x
            ]),
        ]);
        let body = Expression::arc_atom(Value::symbol("y"));

        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(10.0)); // y should be 10 (outer x), not 42
    }

    #[test]
    fn test_eval_let_empty_bindings() {
        let mut env = Environment::new();

        // Test empty bindings: (let () 42)
        let bindings = Expression::arc_list(vec![]);
        let body = Expression::arc_atom(Value::number(42.0));

        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_let_with_expressions() {
        let mut env = Environment::new();

        // Define an identifier to use in binding expressions
        env.define(Symbol::new("a"), Value::number(5.0));

        // Test: (let ((x a) (y 10)) y)
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::symbol("a")), // Should evaluate to 5
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("y")),
                Expression::arc_atom(Value::number(10.0)),
            ]),
        ]);
        let body = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0));
    }

    #[test]
    fn test_eval_let_errors() {
        let mut env = Environment::new();

        // Test no arguments
        let result = eval_let(&[], &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: expected 1 argument, got 0")
        );

        // Test no body expressions
        let bindings = Expression::arc_list(vec![]);
        let args = vec![bindings];
        let result = eval_let(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: requires at least one body expression")
        );

        // Test non-list bindings
        let bindings = Expression::arc_atom(Value::number(42.0));
        let body = Expression::arc_atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: first argument must be a list of bindings")
        );

        // Test invalid binding format (not a list)
        let bindings = Expression::arc_list(vec![Expression::arc_atom(Value::number(42.0))]);
        let body = Expression::arc_atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: each binding must be a list")
        );

        // Test binding with wrong arity
        let bindings =
            Expression::arc_list(vec![Expression::arc_list(vec![Expression::arc_atom(
                Value::symbol("x"),
            )])]);
        let body = Expression::arc_atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let: each binding must be a list of exactly 2 elements")
        );

        // Test binding with non-symbol identifier
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::number(42.0)), // Not a symbol
            Expression::arc_atom(Value::number(1.0)),
        ])]);
        let body = Expression::arc_atom(Value::number(1.0));
        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env);
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
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("undefined")), // Unbound identifier
        ])]);
        let body = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unbound identifier: 'undefined'")
        );
    }
}
