# Requirements for Twine Scheme Interpreter

## Overview

This document captures the functional and non-functional requirements for the Twine Scheme interpreter using the EARS (Easy Approach to Requirements Syntax) format. The interpreter SHALL implement a subset of the R7RS-small Scheme specification to provide a purely functional programming environment with **immutable data structures only** as a core design principle.

## Key Design Principles

### Principle 1: Immutability
**PRIORITY: HIGH** - The interpreter SHALL enforce complete immutability of all data structures. No mutable operations SHALL be supported. This is a fundamental design constraint that affects all aspects of the system.

### Principle 2: Asynchronous IO
**PRIORITY: HIGH** - All IO operations SHALL be asynchronous in the context of the entire runtime. The interpreter SHALL use async/await patterns for all input/output operations to ensure non-blocking execution and proper resource management.

### Principle 3: Parallel Execution
**PRIORITY: HIGH** - The interpreter SHALL support parallel execution of fibers/tasks via a thread pool without a Global Interpreter Lock (GIL). Multiple Scheme computations SHALL execute concurrently across CPU cores for maximum performance.

### Principle 4: Simplicity and Minimalism
**PRIORITY: HIGH** - The interpreter SHALL prioritize simple implementation and minimal syntax. Only the essential subset of R7RS-small SHALL be implemented to maintain code clarity, reduce complexity, and ensure maintainability.

## User Stories

### Story 1: Basic Expression Evaluation
**As a** developer learning functional programming  
**I want to** evaluate basic Scheme expressions  
**So that** I can perform arithmetic and logical operations interactively

### Story 2: Function Definition and Application
**As a** Scheme programmer  
**I want to** define and call custom functions  
**So that** I can create reusable code modules

### Story 3: Interactive REPL
**As a** Scheme developer  
**I want to** interact with the interpreter through a read-eval-print loop  
**So that** I can experiment with code and debug programs interactively

### Story 4: File Execution
**As a** Scheme programmer  
**I want to** execute Scheme programs from files  
**So that** I can run complete programs and scripts

### Story 5: Error Handling
**As a** developer using the interpreter  
**I want to** receive clear error messages when my code has issues  
**So that** I can quickly identify and fix problems

## Functional Requirements

### FR-1: Lexical Analysis
**WHEN** the interpreter receives Scheme source code  
**THEN** the system SHALL tokenize the input into atoms, numbers, strings, and delimiters  
**AND** the system SHALL handle comments by ignoring text after semicolons until end of line

### FR-2: Syntactic Analysis
**WHEN** tokens are parsed  
**THEN** the system SHALL construct an abstract syntax tree (AST) from S-expressions  
**AND** the system SHALL validate proper parentheses matching  
**AND** the system SHALL report syntax errors with line and column information

### FR-3: Immutable Data Types
**WHEN** evaluating expressions  
**THEN** the system SHALL support the following immutable data types:
- Numbers (integers and floating-point) - immutable by nature
- Booleans (#t and #f) - immutable by nature
- Strings - immutable after creation
- Symbols - immutable by nature
- Lists (proper and improper) - immutable after creation
- Procedures (built-in and user-defined) - immutable after creation
**AND** the system SHALL NOT provide any mutation operations for these data types

### FR-4: Arithmetic Operations
**WHEN** arithmetic expressions are evaluated  
**THEN** the system SHALL support the operators: +, -, *, /, =, <, >, <=, >=  
**AND** the system SHALL handle both integer and floating-point arithmetic  
**AND** the system SHALL support variadic arithmetic functions

### FR-5: List Operations
**WHEN** list manipulation is required  
**THEN** the system SHALL provide: car, cdr, cons, list, null?, pair?  
**AND** the system SHALL support proper list construction and deconstruction

### FR-6: Conditional Expressions
**WHEN** conditional logic is needed  
**THEN** the system SHALL support the `if` special form with condition, then-clause, and optional else-clause  
**AND** the system SHALL treat any non-#f value as true in conditional contexts

### FR-7: Variable Binding
**WHEN** variables need to be defined  
**THEN** the system SHALL support `define` for immutable global variable binding  
**AND** the system SHALL support `lambda` for function definition  
**AND** the system SHALL NOT support mutable variable assignment or rebinding

### FR-8: Function Application
**WHEN** functions are called  
**THEN** the system SHALL evaluate arguments left-to-right  
**AND** the system SHALL apply functions with proper argument binding  
**AND** the system SHALL support tail-call optimization for recursive functions

### FR-9: REPL Functionality
**WHEN** the interpreter starts in interactive mode  
**THEN** the system SHALL display a prompt for user input  
**AND** the system SHALL read complete S-expressions  
**AND** the system SHALL evaluate expressions and print results  
**AND** the system SHALL continue the loop until user exits

### FR-10: File Execution
**WHEN** a Scheme file is provided as input  
**THEN** the system SHALL read and evaluate all expressions in the file sequentially  
**AND** the system SHALL report the final expression's result or any errors encountered

### FR-11: Built-in Procedures
**WHEN** standard Scheme procedures are called  
**THEN** the system SHALL provide implementations for:
- Type predicates: number?, boolean?, string?, symbol?, list?, procedure?
- Async I/O operations: display, newline
- List operations: length, append, reverse
- Higher-order functions: map, apply
**AND** the system SHALL implement all I/O operations asynchronously

### FR-12: Error Handling
**WHEN** errors occur during execution  
**THEN** the system SHALL provide descriptive error messages  
**AND** the system SHALL indicate the location of syntax errors  
**AND** the system SHALL distinguish between syntax errors, runtime errors, and type errors  
**AND** the system SHALL continue REPL operation after handling errors

### FR-13: Lexical Scoping and Strict Immutability
**WHEN** variables are referenced  
**THEN** the system SHALL use lexical scoping rules  
**AND** the system SHALL maintain proper environment chains  
**AND** the system SHALL support closures that capture their defining environment  
**AND** the system SHALL enforce strict immutability - all data structures SHALL be immutable after creation  
**AND** the system SHALL NOT provide any assignment or mutation operators whatsoever  
**AND** the system SHALL reject any attempt to modify existing data structures

### FR-14: Asynchronous Runtime
**WHEN** IO operations are performed  
**THEN** the system SHALL execute all IO operations asynchronously  
**AND** the system SHALL maintain an async runtime for proper resource management  
**AND** the system SHALL ensure REPL and file execution work seamlessly with async operations  
**AND** the system SHALL provide async-compatible error handling for IO operations

### FR-15: Parallel Task Execution
**WHEN** multiple computations can be parallelized  
**THEN** the system SHALL execute independent tasks across multiple threads  
**AND** the system SHALL use a thread pool for efficient resource management  
**AND** the system SHALL NOT use a Global Interpreter Lock (GIL)  
**AND** the system SHALL ensure thread-safe access to immutable data structures  
**AND** the system SHALL provide fiber/task primitives for parallel computation

### FR-16: Minimal Language Subset
**WHEN** language features are implemented  
**THEN** the system SHALL include only essential R7RS-small constructs  
**AND** the system SHALL prioritize core functionality over comprehensive feature coverage  
**AND** the system SHALL maintain simple, readable implementation code  
**AND** the system SHALL avoid complex language features that increase implementation complexity  
**AND** the system SHALL focus on the minimal viable subset for functional programming

## Non-Functional Requirements

### NFR-1: Performance
**WHEN** the interpreter evaluates expressions  
**THEN** the system SHALL complete simple arithmetic operations within 1ms  
**AND** the system SHALL handle recursive functions with reasonable stack depth (>1000 calls)  
**AND** the system SHALL maintain responsive async IO without blocking the runtime  
**AND** the system SHALL utilize multiple CPU cores for parallel computation  
**AND** the system SHALL scale performance with available hardware threads

### NFR-2: Memory Management
**WHEN** the interpreter runs  
**THEN** the system SHALL manage memory efficiently without manual intervention  
**AND** the system SHALL handle garbage collection automatically (leveraging Rust's ownership)

### NFR-3: Usability
**WHEN** users interact with the REPL  
**THEN** the system SHALL provide clear prompts and readable output formatting  
**AND** the system SHALL support multi-line input for complex expressions

### NFR-4: Portability
**WHEN** the interpreter is compiled  
**THEN** the system SHALL run on major operating systems (Windows, macOS, Linux)  
**AND** the system SHALL require minimal external dependencies (async runtime like tokio)  
**AND** the system SHALL maintain cross-platform async IO compatibility  
**AND** the system SHALL provide consistent parallel execution across platforms

### NFR-5: Maintainability and Simplicity
**WHEN** the codebase is modified  
**THEN** the system SHALL follow Rust best practices and conventions  
**AND** the system SHALL maintain modular architecture with clear separation of concerns  
**AND** the system SHALL prioritize code simplicity over feature completeness  
**AND** the system SHALL keep implementation complexity minimal  
**AND** the system SHALL prefer straightforward solutions over optimized complex ones

### NFR-6: Standards Compliance and Subset Focus
**WHEN** Scheme features are implemented  
**THEN** the system SHALL follow R7RS-small specification where applicable  
**AND** the system SHALL document any deviations from standard behavior  
**AND** the system SHALL prioritize R7RS-small syntax and semantics over earlier standards  
**AND** the system SHALL implement only the most essential subset of R7RS-small  
**AND** the system SHALL justify each included feature against the simplicity principle

## Acceptance Criteria

### AC-1: Basic Arithmetic
```scheme
> (+ 1 2 3)
6
> (* 4 5)
20
> (/ 10 2)
5
```

### AC-2: Function Definition and Application
```scheme
> (define square (lambda (x) (* x x)))
square
> (square 4)
16
```

### AC-3: List Operations
```scheme
> (cons 1 (cons 2 '()))
(1 2)
> (car '(a b c))
a
> (cdr '(a b c))
(b c)
```

### AC-4: Conditional Logic
```scheme
> (if (> 5 3) 'yes 'no)
yes
> (if #f 'true 'false)
false
```

### AC-5: Function Parameters as Local Binding
```scheme
> ((lambda (x y) (+ x y)) 10 20)
30
> (define add-ten (lambda (x) (+ x 10)))
add-ten
> (add-ten 5)
15
```

### AC-6: Error Handling
```scheme
> (+ 1 'symbol)
Error: Type error - expected number, got symbol
> (car 42)
Error: Type error - expected pair, got number
```

### AC-7: Asynchronous IO
```scheme
> (display "Hello, World!")
Hello, World!
> (newline)

```
*Note: IO operations execute asynchronously but appear synchronous to the user*

### AC-8: Parallel Execution
```scheme
> (define fib (lambda (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2))))))
fib
> (parallel-map fib '(35 36 37 38))
(9227465 14930352 24157817 39088169)
```
*Note: Tasks execute in parallel across multiple threads for improved performance*

### AC-9: Minimal Syntax Example
```scheme
> (define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))
factorial
> (factorial 5)
120
```
*Note: Simple, essential syntax covering define, lambda, if, arithmetic, and recursion*

## Out of Scope

The following R7RS-small features are explicitly out of scope to maintain simplicity:

### Complex Language Features (Simplicity Principle)
- Macros and syntax transformation (define-syntax, syntax-rules)
- Continuations and call/cc
- Module system (define-library, import, export)
- Exception handling (guard, raise) - async-compatible error handling will be implemented instead
- Dynamic binding
- Quasiquote and unquote syntax
- Multiple return values
- Eval procedure

### Advanced Data Types (Minimalism Principle)
- Full numeric tower (complex numbers, rationals, exact/inexact)
- Vector operations
- Bytevector operations
- Record types
- Parameter objects
- Character and string manipulation beyond basics

### I/O and System Features
- Synchronous file I/O (open-input-file, with-input-from-file, etc.) - async alternatives will be considered
- Port operations beyond basic display
- Environment variable access

### Concurrency and Debugging
- Manual threading and concurrency management - handled by async runtime and thread pool
- Debugging facilities
- Performance profiling tools

### Excluded by Other Principles
- **ALL mutable operations** - This is a core design constraint
- Local variable binding forms (let, let*, letrec) - conflicts with immutability principle
- Any form of data structure modification after creation

## Glossary

- **S-expression**: Symbolic expression, the fundamental syntax of Scheme
- **REPL**: Read-Eval-Print Loop, interactive interpreter interface
- **AST**: Abstract Syntax Tree, internal representation of parsed code
- **Tail-call optimization**: Optimization that allows recursive calls in tail position to reuse stack frames
- **Lexical scoping**: Variable binding rules where variables refer to bindings in enclosing lexical scope
- **Closure**: Function object that captures variables from its defining environment
- **Async Runtime**: Event-driven execution environment that manages asynchronous operations
- **Non-blocking IO**: Input/output operations that don't halt program execution while waiting for completion
- **Fiber/Task**: Lightweight unit of computation that can be executed in parallel
- **Thread Pool**: Collection of worker threads that execute tasks without the overhead of thread creation
- **GIL-free**: Architecture without a Global Interpreter Lock, allowing true parallel execution
- **Essential Subset**: Minimal set of language features required for functional programming
- **Implementation Simplicity**: Design principle favoring straightforward code over complex optimizations