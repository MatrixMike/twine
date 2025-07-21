# 🧵 Twine

A minimalist Scheme interpreter written in Rust with fiber-based concurrency and asynchronous I/O.

## Overview

Twine implements a functional subset of R7RS-small Scheme with four core design principles:

- **Asynchronous I/O**: All I/O operations are async with fiber yielding, appearing synchronous to Scheme code
- **Fiber-based Concurrency**: Lightweight fiber scheduler with multi-threaded execution and no Global Interpreter Lock
- **Strict Immutability**: All data structures are immutable after creation (side effects like I/O are still supported)
- **Minimalism**: Essential language features only for maintainability and simplicity

## Features

- Interactive REPL and file execution
- Fiber scheduler with automatic I/O yielding
- Two-layer concurrency: low-level fibers and high-level async tasks
- Immutable data types: numbers, booleans, strings, symbols, lists, procedures
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

;; Async tasks with fiber concurrency
(define task1 (async (lambda () (+ 1 2))))
(define task2 (async (lambda () (* 3 4))))
(+ (task-wait task1) (task-wait task2))  ; => 15
```

## Architecture

Twine uses a fiber scheduler built on the `smol` async runtime, allowing multiple fibers to execute concurrently across CPU cores. The interpreter provides both low-level fiber management (`spawn-fiber`) and high-level task abstraction (`async`, `task-wait`) for different concurrency needs.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.