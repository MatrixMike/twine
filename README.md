<div align="center">
  <h1>🧵 twine</h1>
  <p>
    <strong>
        A minimalist Scheme with fiber-based concurrency and async I/O
    </strong>
  </p>
</div>

## Overview

Twine implements a functional subset of R7RS-small Scheme including macro support with four core design principles:

- **Fiber-based Concurrency**: Lightweight fiber scheduler with multi-threaded execution and no Global Interpreter Lock
- **Asynchronous I/O**: All I/O operations are async with fiber yielding, appearing synchronous to Scheme code
- **Strict Immutability**: All data structures are immutable after creation (side effects like I/O are still supported)
- **Macro System**: R7RS-small macro support with `define-syntax` and `syntax-rules` for compile-time code transformation
- **Minimalism**: Essential language features only for maintainability and simplicity

## Features

- Interactive REPL and file execution
- Fiber scheduler with automatic I/O yielding
- Two-layer concurrency: low-level fibers and high-level async tasks
- Immutable data types: numbers, booleans, strings, symbols, lists, procedures
- R7RS-small macro system with `define-syntax` and `syntax-rules`
- Lexical scoping with closures
- Built-in arithmetic, list operations, and conditionals
- Hierarchical task management with parent-child relationships

## Quick Start

```scheme
;; Basic arithmetic and functions
(define square (lambda (x) (* x x)))
(square 5)  ; => 25

;; List operations
(define numbers '(1 2 3 4 5))
(map square numbers)  ; => (1 4 9 16 25)

;; Async tasks with fiber concurrency - both forms supported
(define task1 (async (+ 1 2)))                    ; Simple expression
(define task2 (async (lambda () (* 3 4))))        ; Explicit thunk
(+ (task-wait task1) (task-wait task2))  ; => 15

;; Macros for code transformation
(define-syntax when
  (syntax-rules ()
    ((when condition body ...)
     (if condition (begin body ...)))))
(when #t (display "Hello World"))
```

## Architecture

Twine uses a fiber scheduler built on the `smol` async runtime, allowing multiple fibers to execute concurrently across CPU cores. The interpreter provides both low-level fiber management (`spawn-fiber`) and high-level task abstraction (via the `async` macro and `task-wait` builtin) for different concurrency needs. The `async` macro supports both simple expressions `(async expr)` and explicit thunks `(async (lambda () body))`, expanding to appropriate `spawn-fiber` calls for convenient task creation.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
