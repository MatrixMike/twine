<div align="center">
  <h1>🧵 twine</h1>
  <p>
    <strong>
      Scheme with fiber-based concurrency and async I/O
    </strong>
  </p>
</div>

## Overview

Twine is a [Scheme](https://en.wikipedia.org/wiki/Scheme_(programming_language)) interpreter implementing:

- **Fiber-based Concurrency**: Lightweight fiber scheduler with multi-threaded parallel execution
- **Asynchronous I/O**: All I/O operations are async with fiber yielding
- **Strict Immutability**: All data structures are immutable after creation
- **Minimalism**: Essential language features only
- **Async Special Form**: `(async <expr> ...)` spawns new fibers for concurrent execution

## Educational Goals

This project serves as a comprehensive learning platform for understanding AI agent development workflows, interpreter design, and modern systems programming concepts.

- **AI Agent Development**: Learning to work with AI coding agents
- **Interpreter Implementation**: Understanding lexing, parsing, evaluation, and runtime systems
- **Async I/O and Parallelism**: Hands-on experience with Rust's async ecosystem
- **Concurrency Models**: Implementing fiber-based scheduling and multi-threaded execution
- **Functional Programming**: Exploring Scheme and immutable data structures

## Key Documentation

For detailed project information, see:

- [`REQUIREMENTS.md`](REQUIREMENTS.md) - Functional requirements, user stories, and acceptance criteria
- [`DESIGN.md`](DESIGN.md) - Technical architecture and implementation details
- [`TASKS.md`](TASKS.md) - Structured implementation plan and task dependencies
- [`AGENT.md`](AGENT.md) - AI agent development guidelines and workflow

## Current Features

### ✅ Implemented
- **Lexical Analysis**: Complete tokenization with position tracking for numbers, strings, symbols, booleans, and delimiters
- **Syntactic Analysis**: Recursive descent parser for S-expressions, atoms, lists, and quoted expressions
- **Immutable Data Types**: Numbers, booleans, strings, symbols, and lists with reference counting
- **Environment Management**: Lexical scoping with identifier binding and closure support
- **Basic Evaluation Engine**: Expression evaluation with special forms and procedure application
- **Function System**: Lambda procedures with lexical closures and tail call optimization
- **Built-in Procedures**: Arithmetic operations, comparisons, list operations, and I/O (`display`, `newline`)
- **Special Forms**: `define`, `lambda`, `let`, `if`, and `quote`
- **Interactive REPL**: Read-eval-print loop with clear prompts and error handling
- **Error Handling**: Comprehensive syntax error reporting with precise position information
- **Fiber Infrastructure**: `Fiber` struct with state management, continuation tracking, and parent-child relationships
- **Fiber Scheduler**: `FiberScheduler` struct with ready queue, fiber management, and thread pool infrastructure
- **Fiber Lifecycle Management**: Complete spawn, yield, resume, and cleanup operations with state transitions
- **Async Special Form**: `(async <expr> ...)` for spawning concurrent fibers directly from Scheme code
- **Scheduler Main Loop**: Event-driven scheduler with thread pool execution and cooperative multitasking
- **Test Coverage**: 423+ tests covering all implemented features with verified I/O output

### 🚧 In Progress
- Comprehensive fiber scheduler testing (T4.1.5)
- File execution capabilities

### 📋 Planned
- Asynchronous I/O integration with automatic fiber yielding
- R7RS-small macro system
- Built-in fiber management procedures

## Quick Start

### Interactive REPL

```bash
cargo run
```

The REPL supports multi-line input with automatic bracket matching. Expressions are evaluated when all brackets are properly balanced:

```scheme
twine> (+ 1 2)
3
twine> (define factorial
  (lambda (n)
    (if (= n 0)
        1
        (* n (factorial (- n 1))))))
()
twine> (factorial 5)
120
twine> (display "Hello, World!")
Hello, World!
twine> (display "foobar")(newline)
foobar
()
twine> (define x 10)(define y 20)(+ x y)
30
twine> 
```

### Example Programs

```scheme
;; Basic arithmetic and functions
(define square (lambda (x) (* x x)))
(square 5)  ; => 25

;; List operations
(define factorial 
  (lambda (n) 
    (if (= n 0) 
        1 
        (* n (factorial (- n 1))))))
(factorial 5)  ; => 120

;; I/O operations (currently implemented)
(display "Hello, ")
(display "World!")
(newline)
(display (+ 2 3))  ; outputs: 5
(newline)

;; Conditional expressions
(if (> 10 5)
    (display "10 is greater than 5")
    (display "This won't print"))
(newline)
```

## Architecture

Twine uses a fiber scheduler built on `smol`, allowing multiple fibers to execute concurrently across CPU cores. The codebase is organized into modules that reflect interpreter concepts:

- `lexer/` - Input tokenization with position tracking and error reporting
- `parser/` - S-expression parsing and AST construction
- `types/` - Immutable Scheme data types
- `runtime/` - Core evaluation engine and environment management
- `scheduler/` - Fiber infrastructure with state management and concurrency primitives
- `repl/` - Interactive interface

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
