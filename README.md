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

- **Fiber-based Concurrency**: Lightweight fiber scheduler with multi-threaded execution
- **Asynchronous I/O**: All I/O operations are async with fiber yielding
- **Strict Immutability**: All data structures are immutable after creation
- **Macro System**: R7RS-small macro support with `define-syntax` and `syntax-rules`
- **Minimalism**: Essential language features only

## Educational Goals

This project serves as a comprehensive learning platform for understanding AI agent development workflows, interpreter design, and modern systems programming concepts.

- **AI Agent Development**: Learning to work with AI coding agents
- **Interpreter Implementation**: Understanding lexing, parsing, evaluation, and runtime systems
- **Async I/O and Parallelism**: Hands-on experience with Rust's async ecosystem
- **Concurrency Models**: Implementing fiber-based scheduling and multi-threaded execution
- **Functional Programming**: Exploring Scheme and immutable data structures

## Features

- Interactive REPL and file execution
- Fiber scheduler with automatic I/O yielding
- Two-layer concurrency: low-level fibers and high-level async tasks
- Immutable data types: numbers, booleans, strings, symbols, lists, procedures
- R7RS-small macro system
- Lexical scoping with closures
- Built-in arithmetic, list operations, and conditionals

## Quick Start

```scheme
;; Basic arithmetic and functions
(define square (lambda (x) (* x x)))
(square 5)  ; => 25

;; List operations
(define numbers '(1 2 3 4 5))
(map square numbers)  ; => (1 4 9 16 25)

;; Async tasks with fiber concurrency
(define task1 (async (+ 1 2)))
(define task2 (async (lambda () (* 3 4))))
(+ (task-wait task1) (task-wait task2))  ; => 15

;; Macros for code transformation
(define-syntax when
  (syntax-rules ()
    ((when condition body ...)
     (if condition (begin body ...)))))
(when #t (display "Hello World"))
```

## Architecture

Twine uses a fiber scheduler built on `smol`, allowing multiple fibers to execute concurrently across CPU cores. The codebase is organized into modules that reflect interpreter concepts:

- `lexer/` - Input tokenization
- `parser/` - S-expression parsing and AST construction
- `types/` - Immutable Scheme data types
- `interpreter/` - Core evaluation engine and environment management
- `runtime/` - Fiber scheduler, task system, and async I/O
- `repl/` - Interactive interface

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
