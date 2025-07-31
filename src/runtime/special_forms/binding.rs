//! Binding and definition special forms
//!
//! This module implements binding special forms like `define` and `let`.
//!
//! ## Organization
//! Functions are organized in logical groups:
//! 1. **Main special forms** - Public evaluation functions for each form
//! 2. **Helper functions** - Internal utilities for processing bindings
//! 3. **Tests** - Comprehensive test coverage

use crate::Error;
use crate::error::Result;
use crate::parser::Expression;
use crate::runtime::special_forms::lambda::create_lambda_procedure;
use crate::runtime::utils::{
    eval_sequence, is_lambda_expression, parse_parameters, validate_unique_binding_identifiers,
    validate_unique_parameters,
};
use crate::runtime::{environment::Environment, eval::eval};
use crate::types::{Procedure, Symbol, Value};
use std::sync::Arc;

// ============================================================================
// MAIN SPECIAL FORMS
// ============================================================================

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
        return Err(Error::arity_error("define", 2, 0));
    }

    let first_arg = Arc::clone(&args[0]);
    match first_arg.as_ref() {
        // Binding definition: (define identifier expression)
        Expression::Atom(Value::Symbol(identifier)) => {
            eval_define_binding(identifier, &args[1..], env)
        }

        // Procedure definition: (define (name param...) body...)
        Expression::List(param_elements) => eval_define_procedure(param_elements, &args[1..], env),

        _ => Err(Error::runtime_error(
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
pub fn eval_let(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(Error::arity_error("let", 1, 0));
    }

    // First argument must be the binding list
    let bindings_expr = Arc::clone(&args[0]);
    let body_exprs = &args[1..];

    if body_exprs.is_empty() {
        return Err(Error::runtime_error(
            "let: requires at least one body expression",
        ));
    }

    // Parse binding list using shared helper
    let (identifiers, expressions) = parse_bindings(bindings_expr.as_ref(), "let")?;

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

    // Evaluate body expressions sequentially in new environment using shared helper
    eval_sequence(body_exprs, &mut let_env)
}

/// Evaluate a letrec special form
///
/// Syntax: (letrec ((id1 expr1) (id2 expr2) ...) body1 body2 ...)
///
/// Letrec enables recursive and mutually recursive bindings by:
/// 1. Creating WeakLambda placeholders for all identifiers first
/// 2. Evaluating expressions in an environment containing all placeholders
/// 3. Initializing WeakLambdas with actual lambdas
/// 4. Replacing WeakLambdas with actual Lambda procedures
/// 5. Evaluating body expressions in the final environment
///
/// Returns the value of the last body expression.
pub fn eval_letrec(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(Error::arity_error("letrec", 1, 0));
    }

    // First argument must be the binding list
    let bindings_expr = Arc::clone(&args[0]);
    let body_exprs = &args[1..];

    if body_exprs.is_empty() {
        return Err(Error::arity_error("letrec", 2, args.len()));
    }

    // Parse binding list
    let binding_elements = match bindings_expr.as_ref() {
        Expression::List(elements) => elements,
        other => {
            return Err(Error::binding_list_must_be_list_error(
                "letrec",
                other.type_name(),
            ));
        }
    };

    // Parse and validate individual bindings
    let mut identifiers = Vec::with_capacity(binding_elements.len());
    let mut value_exprs = Vec::with_capacity(binding_elements.len());

    for binding_expr in binding_elements {
        let binding_elements = match binding_expr.as_ref() {
            Expression::List(elements) => elements,
            other => {
                return Err(Error::binding_must_be_list_error(
                    "letrec",
                    other.type_name(),
                ));
            }
        };

        if binding_elements.len() != 2 {
            return Err(Error::binding_wrong_arity_error(
                "letrec",
                2,
                binding_elements.len(),
            ));
        }

        // Extract identifier
        let identifier = match binding_elements[0].as_ref() {
            Expression::Atom(Value::Symbol(id)) => id.clone(),
            other => {
                return Err(Error::identifier_must_be_symbol_error(
                    "letrec",
                    other.type_name(),
                ));
            }
        };

        // Extract value expression
        let value_expr = Arc::clone(&binding_elements[1]);

        identifiers.push(identifier);
        value_exprs.push(value_expr);
    }

    // Check for duplicate identifiers using shared utility
    validate_unique_binding_identifiers(&identifiers, "letrec")?;

    // Create new environment extending the current one
    let mut letrec_env = Environment::new_scope(env);

    // Phase 1: Identify lambda and non-lambda expressions
    let mut is_lambda_expr = Vec::with_capacity(value_exprs.len());
    let mut lambda_indices = Vec::new();
    let mut non_lambda_indices = Vec::new();

    for (i, value_expr) in value_exprs.iter().enumerate() {
        let is_lambda = is_lambda_expression(value_expr);
        is_lambda_expr.push(is_lambda);

        if is_lambda {
            lambda_indices.push(i);
        } else {
            non_lambda_indices.push(i);
        }
    }

    // Phase 2: Evaluate non-lambda expressions first and add them to environment
    for &i in &non_lambda_indices {
        let value = eval(Arc::clone(&value_exprs[i]), &mut letrec_env)?;
        letrec_env.define(identifiers[i].clone(), value);
    }

    // Phase 3: Create WeakLambda placeholders for lambda expressions
    let mut weak_lambdas = Vec::new();
    for &i in &lambda_indices {
        let weak_lambda = Procedure::weak_lambda();
        weak_lambdas.push(weak_lambda.clone());
        letrec_env.define(identifiers[i].clone(), Value::Procedure(weak_lambda));
    }

    // Phase 4: Evaluate lambda expressions (they can now see all non-lambda values and other WeakLambdas)
    let mut lambda_values = Vec::new();
    for &i in &lambda_indices {
        let lambda_value = eval(Arc::clone(&value_exprs[i]), &mut letrec_env)?;
        lambda_values.push(lambda_value);
    }

    // Phase 5: Initialize WeakLambdas with actual lambdas
    for (weak_lambda, lambda_value) in weak_lambdas.iter().zip(lambda_values.iter()) {
        if let Value::Procedure(Procedure::Lambda(actual_lambda)) = lambda_value {
            weak_lambda
                .set_weak_lambda(actual_lambda)
                .map_err(|_| Error::runtime_error("Failed to initialize WeakLambda"))?;
        }
    }

    // Phase 6: Replace WeakLambdas with actual lambdas in the environment
    for (&i, lambda_value) in lambda_indices.iter().zip(lambda_values.iter()) {
        letrec_env.define(identifiers[i].clone(), lambda_value.clone());
    }

    // Evaluate body expressions sequentially in the final environment
    eval_sequence(body_exprs, &mut letrec_env)
}

/// Evaluate a let* special form
///
/// Syntax: (let* ((id1 expr1) (id2 expr2) ...) body1 body2 ...)
///
/// Semantics:
/// 1. Create a new environment with current environment as parent
/// 2. Bind identifiers sequentially: each binding can see previous bindings
/// 3. Evaluate body expressions sequentially in the final environment
pub fn eval_let_star(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(Error::arity_error("let*", 1, 0));
    }

    // First argument must be the binding list
    let bindings_expr = Arc::clone(&args[0]);
    let body_exprs = &args[1..];

    if body_exprs.is_empty() {
        return Err(Error::runtime_error(
            "let*: requires at least one body expression",
        ));
    }

    // Parse binding list
    let (identifiers, expressions) = parse_bindings(bindings_expr.as_ref(), "let*")?;

    // Create new environment with current environment as parent
    let mut letstar_env = Environment::new_scope(env);

    // Bind identifiers sequentially - each can see previous bindings
    for (identifier, expression) in identifiers.into_iter().zip(expressions.into_iter()) {
        let value = eval(expression, &mut letstar_env)?;
        letstar_env.define(identifier, value);
    }

    // Evaluate body expressions sequentially in new environment
    eval_sequence(body_exprs, &mut letstar_env)
}

/// Evaluate a letrec* special form
///
/// Syntax: (letrec* ((id1 expr1) (id2 expr2) ...) body1 body2 ...)
///
/// Semantics:
/// 1. Create a new environment with current environment as parent
/// 2. Bind identifiers sequentially with recursive capability
/// 3. Each binding can refer to earlier bindings and can be recursive
/// 4. Evaluate body expressions sequentially in the final environment
pub fn eval_letrec_star(args: &[Arc<Expression>], env: &mut Environment) -> Result<Value> {
    if args.is_empty() {
        return Err(Error::arity_error("letrec*", 1, 0));
    }

    // First argument must be the binding list
    let bindings_expr = Arc::clone(&args[0]);
    let body_exprs = &args[1..];

    if body_exprs.is_empty() {
        return Err(Error::arity_error("letrec*", 2, args.len()));
    }

    // Parse binding list
    let (identifiers, expressions) = parse_bindings(bindings_expr.as_ref(), "letrec*")?;

    // Create new environment with current environment as parent
    let mut letrecstar_env = Environment::new_scope(env);

    // Bind identifiers sequentially with recursive capability
    for (identifier, expression) in identifiers.into_iter().zip(expressions.into_iter()) {
        // Use recursive binding helper for each binding
        eval_recursive_binding(&identifier, expression, &mut letrecstar_env)?;
    }

    // Evaluate body expressions sequentially in new environment
    eval_sequence(body_exprs, &mut letrecstar_env)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Handles recursive binding for lambda expressions using WeakLambda approach
fn eval_recursive_binding(
    identifier: &Symbol,
    value_expr: Arc<Expression>,
    env: &mut Environment,
) -> Result<Value> {
    if is_lambda_expression(&value_expr) {
        // For lambda expressions, use WeakLambda approach for potential recursion
        // 1. Create WeakLambda placeholder
        let weak_lambda = Procedure::weak_lambda();

        // 2. Create environment with the WeakLambda placeholder for recursive reference
        let mut recursive_env = env.flatten();
        recursive_env.define(identifier.clone(), Value::Procedure(weak_lambda.clone()));

        // 3. Evaluate the lambda in the environment with WeakLambda
        let lambda_value = eval(value_expr, &mut recursive_env)?;

        // 4. Initialize WeakLambda with actual lambda
        if let Value::Procedure(Procedure::Lambda(actual_lambda)) = &lambda_value {
            weak_lambda
                .set_weak_lambda(actual_lambda)
                .map_err(|_| Error::runtime_error("Failed to initialize WeakLambda"))?;
        }

        // 5. Add the final lambda to the environment
        env.define(identifier.clone(), lambda_value);
    } else {
        // For non-lambda expressions, use standard evaluation
        let value = eval(value_expr, env)?;
        env.define(identifier.clone(), value);
    }

    Ok(Value::Nil)
}

fn eval_define_binding(
    identifier: &Symbol,
    value_exprs: &[Arc<Expression>],
    env: &mut Environment,
) -> Result<Value> {
    if value_exprs.len() != 1 {
        return Err(Error::arity_error("define", 2, value_exprs.len() + 1));
    }

    let value_expr = Arc::clone(&value_exprs[0]);
    eval_recursive_binding(identifier, value_expr, env)
}

fn eval_define_procedure(
    param_elements: &[Arc<Expression>],
    args: &[Arc<Expression>],
    env: &mut Environment,
) -> Result<Value> {
    if param_elements.is_empty() {
        return Err(Error::runtime_error(
            "define: procedure definition requires non-empty parameter list",
        ));
    }

    // Extract procedure name
    let identifier = match param_elements[0].as_ref() {
        Expression::Atom(Value::Symbol(name)) => name.clone(),
        other => {
            return Err(Error::procedure_name_must_be_symbol_error(
                "define",
                other.type_name(),
            ));
        }
    };

    // Extract and validate parameters
    let params = parse_parameters(&param_elements[1..], "define")?;
    validate_unique_parameters(&params, "define")?;

    // Validate procedure body
    if args.is_empty() {
        return Err(Error::runtime_error(
            "define: procedure definition requires at least one body expression",
        ));
    }

    // Create lambda procedure directly using shared logic
    let body_exprs = args.iter().map(Arc::clone).collect();
    let lambda_value = create_lambda_procedure(params, body_exprs, env);

    // Use the shared recursive binding logic
    eval_recursive_binding(&identifier, Arc::new(Expression::Atom(lambda_value)), env)
}

/// Helper function to parse binding list into identifiers and expressions
fn parse_bindings(
    bindings_expr: &Expression,
    form_name: &str,
) -> Result<(Vec<Symbol>, Vec<Arc<Expression>>)> {
    let binding_pairs = match bindings_expr {
        Expression::List(pairs) => pairs,
        other => {
            return Err(Error::first_argument_must_be_list_of_bindings_error(
                form_name,
                other.type_name(),
            ));
        }
    };

    let mut identifiers = Vec::with_capacity(binding_pairs.len());
    let mut expressions = Vec::with_capacity(binding_pairs.len());

    for pair in binding_pairs {
        match pair.as_ref() {
            Expression::List(elements) => {
                if elements.len() != 2 {
                    return Err(Error::binding_elements_wrong_arity_error(form_name));
                }

                // First element must be a symbol (identifier name)
                let identifier = match elements[0].as_ref() {
                    Expression::Atom(Value::Symbol(sym)) => sym.clone(),
                    Expression::Atom(atom) => {
                        return Err(Error::identifier_must_be_symbol_error(
                            form_name,
                            atom.type_name(),
                        ));
                    }
                    other => {
                        return Err(Error::identifier_must_be_symbol_error(
                            form_name,
                            other.type_name(),
                        ));
                    }
                };

                // Second element is the expression to evaluate
                let expression = Arc::clone(&elements[1]);

                identifiers.push(identifier);
                expressions.push(expression);
            }
            _ => {
                return Err(Error::each_binding_must_be_list_error(form_name));
            }
        }
    }

    Ok((identifiers, expressions))
}

// ============================================================================
// TESTS
// ============================================================================

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
                .contains("let: identifier must be a symbol, got number")
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

    #[test]
    fn test_eval_letrec_basic() {
        let mut env = Environment::new();

        // (letrec ((x 42)) x)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ])]);
        let body = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_letrec(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_letrec_recursive_lambda() {
        let mut env = Environment::new();

        // (letrec ((factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1)))))))
        //   (factorial 5))
        let factorial_lambda = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("lambda")),
            Expression::arc_list(vec![Expression::arc_atom(Value::symbol("n"))]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("if")),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("=")),
                    Expression::arc_atom(Value::symbol("n")),
                    Expression::arc_atom(Value::number(0.0)),
                ]),
                Expression::arc_atom(Value::number(1.0)),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("*")),
                    Expression::arc_atom(Value::symbol("n")),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("factorial")),
                        Expression::arc_list(vec![
                            Expression::arc_atom(Value::symbol("-")),
                            Expression::arc_atom(Value::symbol("n")),
                            Expression::arc_atom(Value::number(1.0)),
                        ]),
                    ]),
                ]),
            ]),
        ]);

        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("factorial")),
            factorial_lambda,
        ])]);

        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("factorial")),
            Expression::arc_atom(Value::number(5.0)),
        ]);

        let args = vec![bindings, body];
        let result = eval_letrec(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(120.0));
    }

    #[test]
    fn test_eval_letrec_mutually_recursive() {
        let mut env = Environment::new();

        // (letrec ((even? (lambda (n) (if (= n 0) #t (odd? (- n 1)))))
        //          (odd? (lambda (n) (if (= n 0) #f (even? (- n 1))))))
        //   (even? 4))
        let even_lambda = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("lambda")),
            Expression::arc_list(vec![Expression::arc_atom(Value::symbol("n"))]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("if")),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("=")),
                    Expression::arc_atom(Value::symbol("n")),
                    Expression::arc_atom(Value::number(0.0)),
                ]),
                Expression::arc_atom(Value::boolean(true)),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("odd?")),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("-")),
                        Expression::arc_atom(Value::symbol("n")),
                        Expression::arc_atom(Value::number(1.0)),
                    ]),
                ]),
            ]),
        ]);

        let odd_lambda = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("lambda")),
            Expression::arc_list(vec![Expression::arc_atom(Value::symbol("n"))]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("if")),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("=")),
                    Expression::arc_atom(Value::symbol("n")),
                    Expression::arc_atom(Value::number(0.0)),
                ]),
                Expression::arc_atom(Value::boolean(false)),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("even?")),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("-")),
                        Expression::arc_atom(Value::symbol("n")),
                        Expression::arc_atom(Value::number(1.0)),
                    ]),
                ]),
            ]),
        ]);

        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("even?")),
                even_lambda,
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("odd?")),
                odd_lambda,
            ]),
        ]);

        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("even?")),
            Expression::arc_atom(Value::number(4.0)),
        ]);

        let args = vec![bindings, body];
        let result = eval_letrec(&args, &mut env).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_eval_letrec_multiple_bindings() {
        let mut env = Environment::new();

        // (letrec ((x 10) (y 20)) (+ x y))
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

        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("+")),
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::symbol("y")),
        ]);

        let args = vec![bindings, body];
        let result = eval_letrec(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(30.0));
    }

    #[test]
    fn test_eval_letrec_empty_bindings() {
        let mut env = Environment::new();

        // (letrec () 42)
        let bindings = Expression::arc_list(vec![]);
        let body = Expression::arc_atom(Value::number(42.0));

        let args = vec![bindings, body];
        let result = eval_letrec(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_letrec_errors() {
        let mut env = Environment::new();

        // Test no arguments
        assert!(eval_letrec(&[], &mut env).is_err());

        // Test no body
        let bindings = Expression::arc_list(vec![]);
        assert!(eval_letrec(&[bindings], &mut env).is_err());

        // Test invalid binding list (not a list)
        let invalid_bindings = Expression::arc_atom(Value::number(42.0));
        let body = Expression::arc_atom(Value::number(1.0));
        assert!(eval_letrec(&[invalid_bindings, body], &mut env).is_err());

        // Test invalid binding (not a list)
        let bindings = Expression::arc_list(vec![Expression::arc_atom(Value::number(42.0))]);
        let body = Expression::arc_atom(Value::number(1.0));
        assert!(eval_letrec(&[bindings, body], &mut env).is_err());

        // Test invalid binding (wrong length)
        let bindings =
            Expression::arc_list(vec![Expression::arc_list(vec![Expression::arc_atom(
                Value::symbol("x"),
            )])]);
        let body = Expression::arc_atom(Value::number(1.0));
        assert!(eval_letrec(&[bindings, body], &mut env).is_err());

        // Test invalid identifier (not a symbol)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::number(42.0)),
            Expression::arc_atom(Value::number(1.0)),
        ])]);
        let body = Expression::arc_atom(Value::number(1.0));
        assert!(eval_letrec(&[bindings, body], &mut env).is_err());

        // Test duplicate identifiers
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(1.0)),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(2.0)),
            ]),
        ]);
        let body = Expression::arc_atom(Value::symbol("x"));
        assert!(eval_letrec(&[bindings, body], &mut env).is_err());
    }

    #[test]
    fn test_eval_let_star_basic() {
        let mut env = Environment::new();

        // Test basic let*: (let* ((x 42)) x)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ])]);
        let body = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_let_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_let_star_sequential_binding() {
        let mut env = Environment::new();

        // Test sequential binding: (let* ((x 10) (y x)) y)
        // y should bind to the inner x (10), not any outer x
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(10.0)),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("y")),
                Expression::arc_atom(Value::symbol("x")), // Should see the x bound above
            ]),
        ]);
        let body = Expression::arc_atom(Value::symbol("y"));

        let args = vec![bindings, body];
        let result = eval_let_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(10.0));
    }

    #[test]
    fn test_eval_let_star_sequential_computation() {
        let mut env = Environment::new();

        // Test sequential computation: (let* ((x 5) (y (+ x 3))) y)
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("x")),
                Expression::arc_atom(Value::number(5.0)),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("y")),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("+")),
                    Expression::arc_atom(Value::symbol("x")),
                    Expression::arc_atom(Value::number(3.0)),
                ]),
            ]),
        ]);
        let body = Expression::arc_atom(Value::symbol("y"));

        let args = vec![bindings, body];
        let result = eval_let_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(8.0));
    }

    #[test]
    fn test_eval_let_star_empty_bindings() {
        let mut env = Environment::new();

        // Test empty bindings: (let* () 42)
        let bindings = Expression::arc_list(vec![]);
        let body = Expression::arc_atom(Value::number(42.0));

        let args = vec![bindings, body];
        let result = eval_let_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_let_star_multiple_body_expressions() {
        let mut env = Environment::new();

        // Test multiple body expressions: (let* ((x 5)) 1 2 x)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(5.0)),
        ])]);

        let body1 = Expression::arc_atom(Value::number(1.0));
        let body2 = Expression::arc_atom(Value::number(2.0));
        let body3 = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body1, body2, body3];
        let result = eval_let_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(5.0)); // Should return last expression
    }

    #[test]
    fn test_eval_let_star_errors() {
        let mut env = Environment::new();

        // Test no arguments
        let result = eval_let_star(&[], &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let*: expected 1 argument, got 0")
        );

        // Test no body expressions
        let bindings = Expression::arc_list(vec![]);
        let args = vec![bindings];
        let result = eval_let_star(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("let*: requires at least one body expression")
        );
    }

    #[test]
    fn test_eval_letrec_star_basic() {
        let mut env = Environment::new();

        // Test basic letrec*: (letrec* ((x 42)) x)
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("x")),
            Expression::arc_atom(Value::number(42.0)),
        ])]);
        let body = Expression::arc_atom(Value::symbol("x"));

        let args = vec![bindings, body];
        let result = eval_letrec_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_letrec_star_sequential_recursive() {
        let mut env = Environment::new();

        // Test sequential recursive: (letrec* ((factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))) (factorial 5))
        // This demonstrates self-recursion which should work in letrec*
        let bindings = Expression::arc_list(vec![Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("factorial")),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("lambda")),
                Expression::arc_list(vec![Expression::arc_atom(Value::symbol("n"))]),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("if")),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("=")),
                        Expression::arc_atom(Value::symbol("n")),
                        Expression::arc_atom(Value::number(0.0)),
                    ]),
                    Expression::arc_atom(Value::number(1.0)),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("*")),
                        Expression::arc_atom(Value::symbol("n")),
                        Expression::arc_list(vec![
                            Expression::arc_atom(Value::symbol("factorial")),
                            Expression::arc_list(vec![
                                Expression::arc_atom(Value::symbol("-")),
                                Expression::arc_atom(Value::symbol("n")),
                                Expression::arc_atom(Value::number(1.0)),
                            ]),
                        ]),
                    ]),
                ]),
            ]),
        ])]);
        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("factorial")),
            Expression::arc_atom(Value::number(5.0)),
        ]);

        let args = vec![bindings, body];
        let result = eval_letrec_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(120.0)); // 5! = 120
    }

    #[test]
    fn test_eval_letrec_star_sequential_binding() {
        let mut env = Environment::new();

        // Test sequential binding with recursion: (letrec* ((f (lambda (x) x)) (g (lambda (y) (f y)))) (g 42))
        let bindings = Expression::arc_list(vec![
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("f")),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("lambda")),
                    Expression::arc_list(vec![Expression::arc_atom(Value::symbol("x"))]),
                    Expression::arc_atom(Value::symbol("x")),
                ]),
            ]),
            Expression::arc_list(vec![
                Expression::arc_atom(Value::symbol("g")),
                Expression::arc_list(vec![
                    Expression::arc_atom(Value::symbol("lambda")),
                    Expression::arc_list(vec![Expression::arc_atom(Value::symbol("y"))]),
                    Expression::arc_list(vec![
                        Expression::arc_atom(Value::symbol("f")),
                        Expression::arc_atom(Value::symbol("y")),
                    ]),
                ]),
            ]),
        ]);
        let body = Expression::arc_list(vec![
            Expression::arc_atom(Value::symbol("g")),
            Expression::arc_atom(Value::number(42.0)),
        ]);

        let args = vec![bindings, body];
        let result = eval_letrec_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_letrec_star_empty_bindings() {
        let mut env = Environment::new();

        // Test empty bindings: (letrec* () 42)
        let bindings = Expression::arc_list(vec![]);
        let body = Expression::arc_atom(Value::number(42.0));

        let args = vec![bindings, body];
        let result = eval_letrec_star(&args, &mut env).unwrap();
        assert_eq!(result, Value::number(42.0));
    }

    #[test]
    fn test_eval_letrec_star_errors() {
        let mut env = Environment::new();

        // Test no arguments
        let result = eval_letrec_star(&[], &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("letrec*: expected 1 argument, got 0")
        );

        // Test no body expressions
        let bindings = Expression::arc_list(vec![]);
        let args = vec![bindings];
        let result = eval_letrec_star(&args, &mut env);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("letrec*: expected 2 arguments, got 1")
        );
    }
}
