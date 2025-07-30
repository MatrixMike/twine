//! Integration tests for define procedure syntax
//!
//! This file contains integration tests for define procedure functionality:
//! - Basic procedure definition with define syntax
//! - Multiple parameters and no parameters
//! - Complex procedure bodies with conditionals and expressions
//! - Procedure closure behavior and environment capture
//! - Parameter shadowing and nested calls
//! - Error handling and syntax validation
//! - Interaction with lambda expressions
//! - Procedure definitions with lambda expressions
//! - Complex procedure definitions and calls
//! - Error handling for procedure definitions

mod common;

use common::eval_source;
use twine_scheme::Error;
use twine_scheme::runtime::Environment;
use twine_scheme::types::{Symbol, Value};

#[test]
fn test_integration_define_procedure_basic() {
    let mut env = Environment::new();

    // Define a simple procedure: (define (square x) (* x x))
    let define_result = eval_source("(define (square x) (* x x))", &mut env).unwrap();
    assert_eq!(define_result, Value::Nil);

    // Verify the procedure was created correctly
    let square_proc = env.lookup(&Symbol::new("square")).unwrap();
    assert!(square_proc.is_procedure());
    let proc = square_proc.as_procedure().unwrap();
    assert!(proc.is_lambda());
    assert_eq!(proc.arity(), Some(1));

    // Test calling the procedure
    let result = eval_source("(square 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(25.0));

    let result = eval_source("(square -3)", &mut env).unwrap();
    assert_eq!(result, Value::number(9.0));

    // Test with decimal
    let result = eval_source("(square 2.5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.25));
}

#[test]
fn test_integration_define_procedure_multiple_parameters() {
    let mut env = Environment::new();

    // Define procedure with multiple parameters: (define (add-three x y z) (+ x y z))
    let define_result = eval_source("(define (add-three x y z) (+ x y z))", &mut env).unwrap();
    assert_eq!(define_result, Value::Nil);

    // Verify procedure properties
    let proc_value = env.lookup(&Symbol::new("add-three")).unwrap();
    let proc = proc_value.as_procedure().unwrap();
    assert_eq!(proc.arity(), Some(3));

    // Test calling the procedure
    let result = eval_source("(add-three 1 2 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0));

    let result = eval_source("(add-three 10 20 30)", &mut env).unwrap();
    assert_eq!(result, Value::number(60.0));

    // Define procedure with two parameters
    eval_source("(define (multiply x y) (* x y))", &mut env).unwrap();
    let result = eval_source("(multiply 6 7)", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));
}

#[test]
fn test_integration_define_procedure_no_parameters() {
    let mut env = Environment::new();

    // Define procedure with no parameters: (define (get-answer) 42)
    let define_result = eval_source("(define (get-answer) 42)", &mut env).unwrap();
    assert_eq!(define_result, Value::Nil);

    // Verify procedure properties
    let proc_value = env.lookup(&Symbol::new("get-answer")).unwrap();
    let proc = proc_value.as_procedure().unwrap();
    assert_eq!(proc.arity(), Some(0));

    // Test calling the procedure
    let result = eval_source("(get-answer)", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));

    // Define constant string procedure
    eval_source("(define (get-greeting) \"Hello, World!\")", &mut env).unwrap();
    let result = eval_source("(get-greeting)", &mut env).unwrap();
    assert_eq!(result, Value::string("Hello, World!"));
}

#[test]
fn test_integration_define_procedure_complex_body() {
    let mut env = Environment::new();

    // Define procedure with conditional body
    let define_result = eval_source("(define (abs x) (if (< x 0) (- x) x))", &mut env).unwrap();
    assert_eq!(define_result, Value::Nil);

    // Test with positive number
    let result = eval_source("(abs 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0));

    // Test with negative number
    let result = eval_source("(abs -7)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0));

    // Test with zero
    let result = eval_source("(abs 0)", &mut env).unwrap();
    assert_eq!(result, Value::number(0.0));

    // Define procedure with multiple operations
    eval_source(
        "(define (complex-calc x y) (+ (* x x) (* y y) (* 2 x y)))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(complex-calc 3 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(49.0)); // 9 + 16 + 24 = 49
}

#[test]
fn test_integration_define_procedure_closure() {
    let mut env = Environment::new();

    // Define outer variable
    eval_source("(define base 100)", &mut env).unwrap();

    // Define procedure that captures outer variable
    let define_result = eval_source("(define (add-to-base x) (+ x base))", &mut env).unwrap();
    assert_eq!(define_result, Value::Nil);

    // Test closure behavior - should use captured base
    let result = eval_source("(add-to-base 42)", &mut env).unwrap();
    assert_eq!(result, Value::number(142.0));

    // Change base and test again - with lexical scoping, procedure should still use captured base (100)
    eval_source("(define base 200)", &mut env).unwrap();
    let result = eval_source("(add-to-base 42)", &mut env).unwrap();
    assert_eq!(result, Value::number(142.0)); // Should use original captured base (100)

    // Test nested closure
    eval_source("(define multiplier 5)", &mut env).unwrap();
    eval_source(
        "(define (scale-and-add x) (+ (* x multiplier) base))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(scale-and-add 10)", &mut env).unwrap();
    assert_eq!(result, Value::number(250.0)); // (10 * 5) + 200
}

#[test]
fn test_integration_define_procedure_parameter_shadowing() {
    let mut env = Environment::new();

    // Define outer variable
    eval_source("(define x 999)", &mut env).unwrap();

    // Define procedure where parameter shadows outer variable
    let define_result = eval_source("(define (test-shadow x) (+ x 1))", &mut env).unwrap();
    assert_eq!(define_result, Value::Nil);

    // Parameter x should shadow outer x
    let result = eval_source("(test-shadow 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0));

    // Outer x should still be accessible outside procedure
    let result = eval_source("x", &mut env).unwrap();
    assert_eq!(result, Value::number(999.0));

    // Test multiple parameter shadowing
    eval_source("(define y 888)", &mut env).unwrap();
    eval_source("(define (shadow-both x y) (* x y))", &mut env).unwrap();
    let result = eval_source("(shadow-both 3 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0));

    // Outer variables should remain unchanged
    let result = eval_source("(+ x y)", &mut env).unwrap();
    assert_eq!(result, Value::number(1887.0)); // 999 + 888
}

#[test]
fn test_integration_define_procedure_nested_calls() {
    let mut env = Environment::new();

    // Define multiple procedures
    eval_source("(define (double x) (* x 2))", &mut env).unwrap();
    eval_source("(define (add-one x) (+ x 1))", &mut env).unwrap();
    eval_source("(define (compose-ops x) (double (add-one x)))", &mut env).unwrap();

    // Test nested calls
    let result = eval_source("(compose-ops 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(8.0)); // double(add-one(3)) = double(4) = 8

    // Test direct nested calls
    let result = eval_source("(double (double 5))", &mut env).unwrap();
    assert_eq!(result, Value::number(20.0)); // double(double(5)) = double(10) = 20

    // Test chaining multiple procedures
    eval_source("(define (subtract-two x) (- x 2))", &mut env).unwrap();
    let result = eval_source("(subtract-two (double (add-one 10)))", &mut env).unwrap();
    assert_eq!(result, Value::number(20.0)); // subtract-two(double(add-one(10))) = subtract-two(double(11)) = subtract-two(22) = 20
}

#[test]
fn test_integration_define_procedure_with_lists() {
    let mut env = Environment::new();

    // Define procedure that works with lists
    let define_result = eval_source(
        "(define (sum-first-two lst) (+ (car lst) (car (cdr lst))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(define_result, Value::Nil);

    // Test with number list
    let result = eval_source("(sum-first-two '(5 10 15))", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0));

    // Define procedure that constructs lists
    eval_source("(define (make-triple x) (list x x x))", &mut env).unwrap();
    let result = eval_source("(make-triple 'a)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "a");
    assert_eq!(list.get(2).unwrap().as_symbol().unwrap(), "a");

    // Define procedure with cons operations
    eval_source("(define (prepend-hello lst) (cons 'hello lst))", &mut env).unwrap();
    let result = eval_source("(prepend-hello '(world))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "hello");
    assert_eq!(list.get(1).unwrap().as_symbol().unwrap(), "world");

    // Define procedure that checks list properties
    eval_source(
        "(define (list-info lst) (if (null? lst) 'empty (length lst)))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(list-info '())", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "empty");

    let result = eval_source("(list-info '(a b c d))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);
}

#[test]
fn test_integration_define_procedure_with_let() {
    let mut env = Environment::new();

    // Define procedure that uses let
    let define_result = eval_source(
        "(define (compute x) (let ((temp (* x 2))) (+ temp 1)))",
        &mut env,
    )
    .unwrap();
    assert_eq!(define_result, Value::Nil);

    // Test the procedure
    let result = eval_source("(compute 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(11.0)); // (5 * 2) + 1

    // Define procedure with multiple let bindings
    eval_source(
        "(define (complex-let x y) (let ((a (* x 2)) (b (+ y 3))) (- a b)))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(complex-let 10 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0)); // (10 * 2) - (5 + 3) = 20 - 8 = 12

    // Define procedure with nested let
    eval_source(
        r#"(define (nested-let x)
             (let ((outer (* x 2)))
               (let ((inner (+ outer 1)))
                 (+ outer inner))))"#,
        &mut env,
    )
    .unwrap();
    let result = eval_source("(nested-let 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(13.0)); // outer=6, inner=7, result=6+7=13
}

#[test]
fn test_integration_define_procedure_shadowing_builtins() {
    let mut env = Environment::new();

    // Test that builtins work initially
    let result = eval_source("(+ 2 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(5.0));

    // Shadow the builtin with user-defined procedure
    let define_result = eval_source("(define (+ x y) (* x y))", &mut env).unwrap();
    assert_eq!(define_result, Value::Nil);

    // Now + should refer to the user-defined procedure
    let result = eval_source("(+ 2 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(6.0)); // Should multiply, not add

    // Test shadowing other builtins
    eval_source("(define (car lst) (cdr lst))", &mut env).unwrap();
    let result = eval_source("(car '(a b c))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2); // Should return (b c), not 'a

    // Non-shadowed builtins should still work
    let result = eval_source("(- 10 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0));
}

#[test]
fn test_integration_define_procedure_arity_errors() {
    let mut env = Environment::new();

    // Define procedure expecting 2 parameters
    eval_source("(define (add-two x y) (+ x y))", &mut env).unwrap();

    // Test too few arguments
    let result = eval_source("(add-two 5)", &mut env);
    assert!(result.is_err());
    if let Err(Error::ArityError {
        expected, actual, ..
    }) = result
    {
        assert_eq!(expected, 2);
        assert_eq!(actual, 1);
    } else {
        panic!("Expected ArityError for too few arguments");
    }

    // Test too many arguments
    let result = eval_source("(add-two 1 2 3)", &mut env);
    assert!(result.is_err());
    if let Err(Error::ArityError {
        expected, actual, ..
    }) = result
    {
        assert_eq!(expected, 2);
        assert_eq!(actual, 3);
    } else {
        panic!("Expected ArityError for too many arguments");
    }

    // Test zero-arity procedure with arguments
    eval_source("(define (get-constant) 42)", &mut env).unwrap();
    let result = eval_source("(get-constant 1)", &mut env);
    assert!(result.is_err());
    if let Err(Error::ArityError {
        expected, actual, ..
    }) = result
    {
        assert_eq!(expected, 0);
        assert_eq!(actual, 1);
    } else {
        panic!("Expected ArityError for wrong arity");
    }
}

#[test]
fn test_integration_define_procedure_syntax_errors() {
    let mut env = Environment::new();

    // Test empty parameter list (no procedure name)
    let result = eval_source("(define () 42)", &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("requires non-empty parameter list")
    );

    // Test non-symbol procedure name
    let result = eval_source("(define (42 x) x)", &mut env);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be a symbol"));

    // Test non-symbol parameter
    let result = eval_source("(define (proc 42) x)", &mut env);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be a symbol"));

    // Test duplicate parameters
    let result = eval_source("(define (proc x x) x)", &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("duplicate parameter")
    );

    // Test no body
    let result = eval_source("(define (proc x))", &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("requires at least one body expression")
    );

    // Test invalid parameter list structure
    let result = eval_source("(define proc x)", &mut env);
    assert!(result.is_err()); // Should be variable definition, not procedure

    // Test too few arguments to define
    let result = eval_source("(define)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(define (proc))", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_define_procedure_recursive_pattern() {
    let mut env = Environment::new();

    // Define a procedure that could be used recursively (simple counter factory)
    eval_source(
        "(define (make-incrementer start) (lambda (x) (+ start x)))",
        &mut env,
    )
    .unwrap();

    // Create an incrementer
    eval_source("(define add-ten (make-incrementer 10))", &mut env).unwrap();

    // Use the incrementer
    let result = eval_source("(add-ten 5)", &mut env).unwrap();
    assert_eq!(result, Value::number(15.0));

    // Create another incrementer
    eval_source("(define add-hundred (make-incrementer 100))", &mut env).unwrap();
    let result = eval_source("(add-hundred 23)", &mut env).unwrap();
    assert_eq!(result, Value::number(123.0));

    // Define procedure that returns different procedures based on condition
    eval_source(
        r#"(define (make-operation op)
             (if (eq? op 'add)
               (lambda (x y) (+ x y))
               (lambda (x y) (* x y))))"#,
        &mut env,
    )
    .unwrap();

    eval_source("(define adder (make-operation 'add))", &mut env).unwrap();
    let result = eval_source("(adder 3 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(7.0));

    eval_source("(define multiplier (make-operation 'mult))", &mut env).unwrap();
    let result = eval_source("(multiplier 3 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0));
}

#[test]
fn test_integration_define_procedure_lambda_interaction() {
    let mut env = Environment::new();

    // Define procedure using define syntax
    eval_source("(define (square x) (* x x))", &mut env).unwrap();

    // Define procedure using lambda syntax
    eval_source("(define cube (lambda (x) (* x x x)))", &mut env).unwrap();

    // Both should work the same way
    let square_result = eval_source("(square 4)", &mut env).unwrap();
    assert_eq!(square_result, Value::number(16.0));

    let cube_result = eval_source("(cube 3)", &mut env).unwrap();
    assert_eq!(cube_result, Value::number(27.0));

    // Both should have procedure properties
    let square_proc = env.lookup(&Symbol::new("square")).unwrap();
    assert!(square_proc.is_procedure());

    let cube_proc = env.lookup(&Symbol::new("cube")).unwrap();
    assert!(cube_proc.is_procedure());

    // Test that define syntax is equivalent to lambda
    eval_source("(define add1-define (lambda (x) (+ x 1)))", &mut env).unwrap();
    eval_source("(define (add1-syntax x) (+ x 1))", &mut env).unwrap();

    let result1 = eval_source("(add1-define 10)", &mut env).unwrap();
    let result2 = eval_source("(add1-syntax 10)", &mut env).unwrap();
    assert_eq!(result1, result2);

    // Test using one in the definition of the other
    eval_source("(define (power4 x) (square (square x)))", &mut env).unwrap();
    let result = eval_source("(power4 2)", &mut env).unwrap();
    assert_eq!(result, Value::number(16.0)); // 2^4 = 16

    eval_source(
        "(define sixth-power (lambda (x) (* (square x) (cube x))))",
        &mut env,
    )
    .unwrap();
    let result = eval_source("(sixth-power 2)", &mut env).unwrap();
    assert_eq!(result, Value::number(32.0)); // square(2) * cube(2) = 4 * 8 = 32
}

#[test]
fn test_integration_define_procedure_complex_evaluation() {
    let mut env = Environment::new();

    // Define helper procedures
    eval_source("(define (positive? x) (> x 0))", &mut env).unwrap();
    eval_source("(define (negative? x) (< x 0))", &mut env).unwrap();

    // Define complex procedure with nested conditionals and calls
    eval_source(
        r#"(define (classify x)
             (if (positive? x)
               'positive
               (if (negative? x)
                 'negative
                 'zero)))"#,
        &mut env,
    )
    .unwrap();

    // Test classification
    let result = eval_source("(classify 5)", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "positive");

    let result = eval_source("(classify -3)", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "negative");

    let result = eval_source("(classify 0)", &mut env).unwrap();
    assert_eq!(result.as_symbol().unwrap(), "zero");

    // Define procedure that uses multiple features
    eval_source(
        r#"(define (process-number x)
             (let ((class (classify x))
                   (abs-val (if (negative? x) (- x) x)))
               (list class abs-val (* abs-val abs-val))))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(process-number -4)", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "negative");
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 4.0); // abs(-4)
    assert_eq!(list.get(2).unwrap().as_number().unwrap(), 16.0); // 4^2
}

#[test]
fn test_integration_define_procedure_comprehensive_integration() {
    let mut env = Environment::new();

    // Define procedure that combines multiple features
    eval_source(
        r#"(define (process-list lst)
             (let ((first (car lst))
                   (rest (cdr lst)))
               (if (null? rest)
                 first
                 (+ first (car rest)))))"#,
        &mut env,
    )
    .unwrap();

    // Test with two-element list
    let result = eval_source("(process-list '(10 20))", &mut env).unwrap();
    assert_eq!(result, Value::number(30.0));

    // Test with single-element list
    let result = eval_source("(process-list '(42))", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));

    // Define procedure that works with other procedures
    eval_source("(define (apply-twice f x) (f (f x)))", &mut env).unwrap();
    eval_source("(define (double x) (* x 2))", &mut env).unwrap();

    let result = eval_source("(apply-twice double 3)", &mut env).unwrap();
    assert_eq!(result, Value::number(12.0)); // double(double(3)) = double(6) = 12

    // Test comprehensive procedure interaction
    eval_source(
        r#"(define (comprehensive-test data)
             (let ((processed (process-list data))
                   (doubled (apply-twice double (car data))))
               (if (> processed doubled)
                 'first-wins
                 'second-wins)))"#,
        &mut env,
    )
    .unwrap();

    let result = eval_source("(comprehensive-test '(3 10))", &mut env).unwrap();
    // processed = 3 + 10 = 13, doubled = double(double(3)) = 12
    assert_eq!(result.as_symbol().unwrap(), "first-wins");

    let result = eval_source("(comprehensive-test '(2 5))", &mut env).unwrap();
    // processed = 2 + 5 = 7, doubled = double(double(2)) = 8
    assert_eq!(result.as_symbol().unwrap(), "second-wins");
}

#[test]
fn test_integration_let_binding_basic() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test basic let binding: (let ((x 42)) x)
    let result = eval_source("(let ((x 42)) x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Test multiple bindings: (let ((x 10) (y 20)) y)
    let result = eval_source("(let ((x 10) (y 20)) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0);

    // Test let with arithmetic in body
    let result = eval_source("(let ((x 10) (y 5)) (+ x y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);

    // Test let with different data types
    let result = eval_source("(let ((name \"Alice\") (age 30)) name)", &mut env).unwrap();
    assert_eq!(result.as_string().unwrap(), "Alice");

    let result = eval_source("(let ((flag #t) (count 0)) flag)", &mut env).unwrap();
    assert!(result.as_boolean().unwrap());
}

#[test]
fn test_integration_let_binding_lexical_scoping() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Define x in outer environment
    env.define_str("x", Value::number(100.0));

    // Test lexical scoping - inner x should shadow outer x
    let result = eval_source("(let ((x 42)) x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // After let, outer x should still be accessible
    let result = eval_source("x", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 100.0);

    // Test that let doesn't affect outer environment
    let result = eval_source("(let ((y 999)) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 999.0);

    // y should not be accessible outside let
    let result = eval_source("y", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_let_binding_simultaneous() {
    use twine_scheme::runtime::environment::Environment;
    use twine_scheme::types::Value;

    let mut env = Environment::new();

    // Define x in outer environment
    env.define_str("x", Value::number(10.0));

    // Test simultaneous binding - y should see outer x, not inner x
    let result = eval_source("(let ((x 42) (y x)) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0); // y sees outer x

    // Test that both bindings work
    let result = eval_source("(let ((x 42) (y x)) x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0); // inner x

    // Test simultaneous binding with expressions
    let result = eval_source("(let ((a (+ 1 2)) (b (* 3 4))) (+ a b))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0); // 3 + 12
}

#[test]
fn test_integration_let_binding_multiple_body() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test multiple body expressions - should return last one
    let result = eval_source("(let ((x 5)) 1 2 3 x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0);

    // Test with define in let body - should be scoped to let
    let result = eval_source("(let ((x 10)) (define y x) y)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test multiple expressions in let body (only last expression returned)
    let result = eval_source(
        "(let ((x 1)) (display \"side effect\") (+ x 2) (+ x 3))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0); // x + 3 = 1 + 3 = 4

    // Test with mixed expression types
    let result = eval_source("(let ((x 10)) \"ignored\" #f x)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);
}

#[test]
fn test_integration_let_binding_with_lists() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with list operations
    let result = eval_source("(let ((lst '(1 2 3))) (car lst))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    // Test let with list construction
    let result = eval_source("(let ((x 42)) (list x x))", &mut env).unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list.get(0).unwrap().as_number().unwrap(), 42.0);
    assert_eq!(list.get(1).unwrap().as_number().unwrap(), 42.0);

    // Test let with nested list operations
    let result = eval_source(
        "(let ((data '((a 1) (b 2)))) (car (cdr (car data))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);

    // Test let with list length
    let result = eval_source("(let ((items '(a b c d))) (length items))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0);

    // Test let with cons operations
    let result = eval_source(
        "(let ((head 'first) (tail '(second third))) (cons head tail))",
        &mut env,
    )
    .unwrap();
    assert!(result.is_list());
    let list = result.as_list().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().as_symbol().unwrap(), "first");
}

#[test]
fn test_integration_let_binding_nested() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test nested let expressions
    let result = eval_source("(let ((x 10)) (let ((y 20)) (+ x y)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Test nested let with shadowing
    let result = eval_source("(let ((x 10)) (let ((x 20)) x))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 20.0);

    // Test access to outer binding in nested let
    let result = eval_source("(let ((x 10)) (let ((y 20)) (+ x y)))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 30.0);

    // Test deeply nested let
    let result = eval_source(
        "(let ((a 1)) (let ((b 2)) (let ((c 3)) (+ a b c))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 6.0);

    // Test nested let with different scoping
    let result = eval_source(
        "(let ((x 1)) (let ((x 2) (y x)) (let ((x 3)) (+ x y))))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 4.0); // 3 + 1 (y sees outer x)
}

#[test]
fn test_integration_let_binding_with_conditionals() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with if expressions
    let result = eval_source("(let ((x 10) (y 20)) (if (< x y) x y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test let binding boolean values
    let result = eval_source("(let ((flag #t)) (if flag 'yes 'no))", &mut env).unwrap();
    assert!(result.is_symbol());
    assert_eq!(result.as_symbol().unwrap(), "yes");

    // Test conditional let bindings
    let result = eval_source("(let ((value (if (> 5 3) 100 200))) value)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 100.0);

    // Test let in conditional branches
    let result = eval_source("(if #t (let ((x 42)) x) (let ((y 99)) y))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Test complex conditional with let
    let result = eval_source(
        r#"(let ((threshold 50) (value 75))
             (if (> value threshold)
               (let ((bonus 10)) (+ value bonus))
               (let ((penalty 5)) (- value penalty))))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 85.0);
}

#[test]
fn test_integration_let_binding_with_define() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with define - bindings defined in let body stay in let scope
    let result = eval_source("(let ((x 42)) (define local-var x) local-var)", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // The binding defined in let should NOT be accessible in outer scope
    assert!(env.lookup_str("local-var").is_err());

    // Test define procedure in let
    let result = eval_source(
        "(let ((factor 2)) (define (double x) (* x factor)) (double 21))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // The procedure should not be accessible outside let
    let result = eval_source("(double 5)", &mut env);
    assert!(result.is_err());

    // Test multiple defines in let
    let result = eval_source(
        r#"(let ((base 10))
             (define increment 5)
             (define (add-increment x) (+ x increment))
             (add-increment base))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 15.0);
}

#[test]
fn test_integration_let_binding_error_cases() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with undefined identifier in binding expression
    let result = eval_source("(let ((x undefined-var)) x)", &mut env);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unbound identifier")
    );

    // Test malformed let syntax - no bindings
    let result = eval_source("(let)", &mut env);
    assert!(result.is_err());

    // Test malformed let syntax - no body
    let result = eval_source("(let ((x 1)))", &mut env);
    assert!(result.is_err());

    // Test malformed binding - not a list
    let result = eval_source("(let (x) x)", &mut env);
    assert!(result.is_err());

    // Test malformed binding - wrong number of elements
    let result = eval_source("(let ((x)) x)", &mut env);
    assert!(result.is_err());

    let result = eval_source("(let ((x 1 2)) x)", &mut env);
    assert!(result.is_err());

    // Test binding to non-identifier
    let result = eval_source("(let ((42 1)) 42)", &mut env);
    assert!(result.is_err());

    // Test error in binding expression
    let result = eval_source("(let ((x (+ 1 \"not-number\"))) x)", &mut env);
    assert!(result.is_err());

    // Test error in body expression
    let result = eval_source("(let ((x 1)) (+ x \"not-number\"))", &mut env);
    assert!(result.is_err());
}

#[test]
fn test_integration_comprehensive_binding_behavior() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test complex scenario combining define and let
    eval_source("(define base 10)", &mut env).unwrap();

    let result = eval_source(
        "(let ((multiplier 3) (offset 5))
           (define computed (* base multiplier))
           (define result (+ computed offset))
           result)",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 35.0); // (10 * 3) + 5

    // Verify that let-scoped definitions don't leak
    assert!(env.lookup_str("multiplier").is_err());
    assert!(env.lookup_str("offset").is_err());
    assert!(env.lookup_str("computed").is_err());
    assert!(env.lookup_str("result").is_err());

    // But base should still be accessible
    let result = eval_source("base", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 10.0);

    // Test let with lambda and closure
    eval_source("(define global-factor 100)", &mut env).unwrap();

    let result = eval_source(
        r#"(let ((local-factor 2))
             (define make-multiplier
               (lambda (x)
                 (* x local-factor global-factor)))
             (make-multiplier 3))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 600.0); // 3 * 2 * 100

    // Test complex nested scoping
    let result = eval_source(
        r#"(let ((x 1))
             (let ((x 2) (y x))
               (let ((x 3) (z y))
                 (+ x y z))))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 5.0); // 3 + 1 + 1 (y and z both see original x)
}

#[test]
fn test_integration_let_with_procedures() {
    use twine_scheme::runtime::environment::Environment;

    let mut env = Environment::new();

    // Test let with lambda expressions
    let result = eval_source(
        "(let ((add-one (lambda (x) (+ x 1)))) (add-one 41))",
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 42.0);

    // Test let with procedure definitions
    eval_source("(define (square x) (* x x))", &mut env).unwrap();

    let result = eval_source("(let ((value 7)) (square value))", &mut env).unwrap();
    assert_eq!(result.as_number().unwrap(), 49.0);

    // Test let capturing environment for procedures
    let result = eval_source(
        r#"(let ((base 100))
             (define (add-to-base x) (+ x base))
             (let ((base 200))
               (add-to-base 50)))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 150.0); // Uses captured base = 100

    // Test let with higher-order procedures
    let result = eval_source(
        r#"(let ((apply-twice (lambda (f x) (f (f x))))
                   (add-one (lambda (x) (+ x 1))))
             (apply-twice add-one 5))"#,
        &mut env,
    )
    .unwrap();
    assert_eq!(result.as_number().unwrap(), 7.0); // add-one applied twice to 5
}
#[test]
fn test_integration_letrec_basic() {
    let mut env = Environment::new();

    // Simple letrec with one binding
    let result = eval_source("(letrec ((x 42)) x)", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));
}

#[test]
fn test_integration_letrec_recursive_factorial() {
    let mut env = Environment::new();

    // Recursive factorial function
    let source = r#"
        (letrec ((factorial (lambda (n)
                              (if (= n 0)
                                  1
                                  (* n (factorial (- n 1)))))))
          (factorial 5))
    "#;

    let result = eval_source(source, &mut env).unwrap();
    assert_eq!(result, Value::number(120.0));
}

#[test]
fn test_integration_letrec_mutually_recursive() {
    let mut env = Environment::new();

    // Mutually recursive even/odd functions
    let source = r#"
        (letrec ((even? (lambda (n)
                          (if (= n 0)
                              #t
                              (odd? (- n 1)))))
                 (odd? (lambda (n)
                         (if (= n 0)
                             #f
                             (even? (- n 1))))))
          (even? 4))
    "#;

    let result = eval_source(source, &mut env).unwrap();
    assert_eq!(result, Value::boolean(true));

    // Test odd case
    let source2 = r#"
        (letrec ((even? (lambda (n)
                          (if (= n 0)
                              #t
                              (odd? (- n 1)))))
                 (odd? (lambda (n)
                         (if (= n 0)
                             #f
                             (even? (- n 1))))))
          (odd? 3))
    "#;

    let result2 = eval_source(source2, &mut env).unwrap();
    assert_eq!(result2, Value::boolean(true));
}

#[test]
fn test_integration_letrec_mixed_bindings() {
    let mut env = Environment::new();

    // Mix of lambda and non-lambda bindings
    let source = r#"
        (letrec ((x 10)
                 (double (lambda (n) (* n 2)))
                 (y 20))
          (+ x (double y)))
    "#;

    let result = eval_source(source, &mut env).unwrap();
    assert_eq!(result, Value::number(50.0));
}

#[test]
fn test_integration_letrec_nested() {
    let mut env = Environment::new();

    // Nested letrec expressions
    let source = r#"
        (letrec ((outer (lambda (x)
                          (letrec ((inner (lambda (y) (+ x y))))
                            (inner 10)))))
          (outer 5))
    "#;

    let result = eval_source(source, &mut env).unwrap();
    assert_eq!(result, Value::number(15.0));
}

#[test]
fn test_integration_letrec_lexical_scoping() {
    let mut env = Environment::new();

    // Test lexical scoping with letrec
    env.define("x".into(), Value::number(100.0));

    let source = r#"
        (letrec ((x 1)
                 (f (lambda () x)))
          (f))
    "#;

    let result = eval_source(source, &mut env).unwrap();
    assert_eq!(result, Value::number(1.0)); // Should use letrec's x, not global x
}

#[test]
fn test_integration_letrec_multiple_body_expressions() {
    let mut env = Environment::new();

    // Multiple body expressions in letrec
    let source = r#"
        (letrec ((x 10)
                 (y 20))
          (display "Computing sum...")
          (newline)
          (+ x y))
    "#;

    let result = eval_source(source, &mut env).unwrap();
    assert_eq!(result, Value::number(30.0));
}

#[test]
fn test_integration_letrec_tail_recursion() {
    let mut env = Environment::new();

    // Test tail recursion optimization with letrec
    let source = r#"
        (letrec ((countdown (lambda (n)
                              (if (= n 0)
                                  'done
                                  (countdown (- n 1))))))
          (countdown 100))
    "#;

    let result = eval_source(source, &mut env).unwrap();
    assert_eq!(result, Value::symbol("done"));
}

#[test]
fn test_integration_letrec_complex_mutual_recursion() {
    let mut env = Environment::new();

    // More complex mutually recursive functions
    let source = r#"
        (letrec ((is-even? (lambda (n)
                             (if (= n 0)
                                 #t
                                 (is-odd? (- n 1)))))
                 (is-odd? (lambda (n)
                            (if (= n 0)
                                #f
                                (is-even? (- n 1)))))
                 (test-both (lambda (x y)
                              (list (is-even? x) (is-odd? y)))))
          (test-both 6 7))
    "#;

    let result = eval_source(source, &mut env).unwrap();

    // Should return (list #t #t)
    if let Value::List(list) = result {
        let values = list.as_slice();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], Value::boolean(true));
        assert_eq!(values[1], Value::boolean(true));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_integration_letrec_error_cases() {
    let mut env = Environment::new();

    // Test various error conditions

    // Missing body
    assert!(eval_source("(letrec ((x 1)))", &mut env).is_err());

    // Invalid binding syntax
    assert!(eval_source("(letrec (x 1) x)", &mut env).is_err());

    // Duplicate identifiers
    assert!(eval_source("(letrec ((x 1) (x 2)) x)", &mut env).is_err());

    // Invalid identifier (not a symbol)
    assert!(eval_source("(letrec ((42 1)) 42)", &mut env).is_err());
}

#[test]
fn test_integration_letrec_empty_bindings() {
    let mut env = Environment::new();

    // Empty binding list should work
    let result = eval_source("(letrec () 42)", &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));
}

#[test]
fn test_integration_letrec_vs_let_difference() {
    let mut env = Environment::new();

    // This should work with letrec but fail with let
    let letrec_source = r#"
        (letrec ((f (lambda () (g)))
                 (g (lambda () 42)))
          (f))
    "#;

    let result = eval_source(letrec_source, &mut env).unwrap();
    assert_eq!(result, Value::number(42.0));

    // The equivalent let would fail because g is not in scope when f is defined
    let let_source = r#"
        (let ((f (lambda () (g)))
              (g (lambda () 42)))
          (f))
    "#;

    assert!(eval_source(let_source, &mut env).is_err());
}

#[test]
fn test_integration_recursive_define_compatibility() {
    let mut env = Environment::new();

    // Test that recursive define also works with our WeakLambda implementation
    let define_source = r#"
        (define factorial
          (lambda (n)
            (if (= n 0)
                1
                (* n (factorial (- n 1))))))
    "#;

    eval_source(define_source, &mut env).unwrap();

    let result = eval_source("(factorial 4)", &mut env).unwrap();
    assert_eq!(result, Value::number(24.0));
}
