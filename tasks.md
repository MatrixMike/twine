# Twine Scheme Interpreter - Implementation Tasks

## Overview

This document provides a detailed implementation plan for the Twine Scheme interpreter, based on the specifications in `requirements.md` and technical design in `design.md`. Tasks are organized incrementally to allow for a minimal working interpreter that gradually adds features.

## Task Organization

- **Phase 1**: Core Language Foundation
- **Phase 2**: Basic Interpreter Functionality  
- **Phase 3**: Advanced Language Features
- **Phase 4**: Concurrency and Async Features
- **Phase 5**: Macro System and Polish

Each task includes:
- Clear description and expected outcome
- References to requirements (FR-X) and design sections
- Dependencies on other tasks
- Test requirements
- All-tests-passing validation

## Task Implementation Guidelines

### Minimal Implementation Principle
Each task should implement **ONLY** the features explicitly described in that task. Do not implement functionality that belongs to future tasks, even if it seems related or convenient to add. This ensures:
- Clear progress tracking
- Incremental testing and validation
- Easier debugging and rollback
- Proper dependency management

### Dependency Management
- Keep project dependencies to an **absolute minimum**
- When required, choose alternatives that are minimal and have as few dependencies as possible
- All async-related crates **must** come from the smol ecosystem: https://github.com/smol-rs
- Add Rust dependencies **only when they are actually needed** for a specific task
- Do not add all dependencies at once in the initial setup
- Each dependency addition should be justified by the current task requirements
- Update `Cargo.toml` incrementally as features are implemented
- Avoid dependencies that pull in large dependency trees

### Implementation Constraints
- **No forward implementation**: Don't add stubs, placeholder code, or partial implementations for future features
- **No premature optimization**: Implement the simplest solution that works for the current task
- **Minimal viable feature**: Each task should produce the smallest working implementation of its described functionality
- **Test-driven**: Write tests for the current task only, not for future functionality

### Testing Requirements
Each task must include comprehensive testing that validates:
- **Unit tests**: Test individual functions and methods in isolation
- **Integration tests**: Test interactions between components where applicable
- **Error condition tests**: Test all error paths and edge cases
- **Property tests**: Verify invariants and expected behaviors

### All-Tests-Passing Constraint
**CRITICAL**: After completing each task, ALL tests in the project must pass. This includes:
- All existing tests from previous tasks
- All new tests for the current task
- No test should be commented out, ignored, or marked as "todo"
- Tests must run successfully with `cargo test`
- If any test fails, the task is not considered complete

This constraint ensures:
- No regression in previously implemented functionality
- Quality assurance at every step
- Continuous integration readiness
- Incremental validation of the entire system

---

## Phase 1: Core Language Foundation

### 1.1 Project Setup and Infrastructure

- [x] **T1.1.1**: Initialize Rust project structure
  - Create basic `Cargo.toml` with project metadata (name, version, edition)
  - DO NOT add any external dependencies yet - add them only when needed
  - Create basic `src/main.rs` and `src/lib.rs` files
  - DO NOT create module directories yet - create them when implementing specific components
  - **Ref**: Design Section "Dependencies"
  - **Tests**: Create basic test infrastructure in `src/lib.rs`, verify project compiles with `cargo check`
  - **All tests must pass**: `cargo test` should run successfully (even if no tests exist yet)
  - **Constraint**: Minimal project setup only, no premature module structure

- [x] **T1.1.2**: Set up basic error handling infrastructure
  - Add `thiserror` dependency to `Cargo.toml` (first external dependency - chosen for its minimal footprint and zero dependencies)
  - Implement basic `Error` enum with only essential variants needed for Phase 1: `SyntaxError`, `ParseError`
  - DO NOT implement all error types from design.md - add them when needed
  - Implement `Display` trait for current error types only
  - Create `Result<T>` type alias
  - **Ref**: FR-12, Design Section "Error Handling"
  - **Tests**: Unit tests for error creation, Display implementation, and Result type usage
  - **All tests must pass**: `cargo test` should run successfully with all error handling tests
  - **Constraint**: Only implement error types needed for current phase

- [x] **T1.1.3**: Set up local dependency source management
  - Create `deps/` directory structure: `deps/vendor/`, `deps/docs/`, `deps/registry/`
  - Add `deps/` to `.gitignore` to prevent committing dependency sources
  - Set up vendor management with `cargo vendor deps/vendor`
  - Generate comprehensive documentation with `cargo doc --all-features --document-private-items`
  - Copy generated docs to `deps/docs/` directory
  - Create maintenance script or document commands for updating vendored sources
  - **Ref**: Design Section "Local Dependency Source Management"
  - **Tests**: Verify `deps/` directories exist, `.gitignore` excludes them, and documentation is generated
  - **All tests must pass**: Basic project structure tests should still pass
  - **Constraint**: Set up infrastructure for AI agent dependency access

- [ ] **T1.1.4**: Create basic test framework structure
  - Create `tests/` directory for integration tests
  - Add basic unit test setup in `src/lib.rs`
  - DO NOT create test utilities yet - add them when specific tests need them
  - **Ref**: Design Section "Testing Strategy"
  - **Tests**: Verify test framework setup works with a simple dummy test
  - **All tests must pass**: `cargo test` should discover and run tests successfully
  - **Constraint**: Minimal test setup only, no premature utilities

### 1.2 Core Data Types and Value System

- [ ] **T1.2.1**: Implement basic `Value` enum
  - Create `src/types.rs` module
  - Create `Value` enum with Number, Boolean, String, Symbol, Nil variants ONLY
  - DO NOT add List, Procedure, TaskHandle variants yet - these belong to future tasks
  - Implement `Clone`, `Debug`, `PartialEq` traits
  - Add basic constructor methods for current variants only
  - **Ref**: FR-3, Design Section "Value System"
  - **Tests**: Unit tests for Value creation, Debug output, PartialEq behavior, and Clone functionality
  - **All tests must pass**: All Value enum tests plus existing error handling tests
  - **Constraint**: Only implement Value variants needed for lexer/parser phases

- [ ] **T1.2.2**: Implement immutable number type
  - Define `SchemeNumber` type (f64 wrapper)
  - DO NOT implement arithmetic operations yet - these belong to evaluation phase
  - Add only number parsing and basic formatting for display
  - **Ref**: FR-4, Design Section "Immutable Value Design"
  - **Tests**: Unit tests for number parsing, formatting, equality, and edge cases (infinity, NaN)
  - **All tests must pass**: All number type tests plus all previous tests (Value, Error, etc.)
  - **Constraint**: Only implement what's needed for lexer number token creation

- [ ] **T1.2.3**: Implement immutable string and symbol types
  - Define `SchemeString` and `SchemeSymbol` types as simple wrappers
  - DO NOT implement symbol interning yet - use simple String storage for now
  - Add basic equality and hashing
  - DO NOT implement string operations - only basic construction and display
  - **Ref**: FR-3, Design Section "Immutable Value Design"
  - **Tests**: Unit tests for string/symbol creation, equality, hashing, and display formatting
  - **All tests must pass**: All string/symbol tests plus all previous tests
  - **Constraint**: Minimal types for lexer/parser needs only

- [ ] **T1.2.4**: Implement immutable list type
  - Define `SchemeList` using simple `Vec<Value>` (no Arc yet)
  - DO NOT implement list operations (car, cdr, cons) - these belong to evaluation phase
  - DO NOT add structural sharing - implement basic version first
  - Add only basic construction and display
  - **Ref**: FR-5, Design Section "List Operations and Structural Sharing"
  - **Tests**: Unit tests for list creation, basic display, equality, and Vec operations
  - **All tests must pass**: All list type tests plus all previous tests
  - **Constraint**: Basic list type for parser AST only, no operations yet

- [ ] **T1.2.5**: Add comprehensive value system tests
  - Test basic data type construction and display
  - Test equality and basic properties
  - DO NOT test operations that haven't been implemented yet
  - DO NOT add performance benchmarks yet - focus on correctness
  - **Tests**: Comprehensive test suite covering all Value variants, edge cases, and error conditions
  - **All tests must pass**: Complete value system test suite (20+ tests expected)
  - **Constraint**: Test only the minimal functionality implemented so far

### 1.3 Lexical Analysis

- [ ] **T1.3.1**: Implement `Token` enum
  - Create `src/lexer.rs` module
  - Create token types from `design.md`: LeftParen, RightParen, Quote, Number, String, Symbol, Boolean, EOF
  - Add position tracking (line, column)
  - Implement `Debug` and `PartialEq` traits
  - **Tests**: Unit tests for Lexer creation, position tracking, and basic iteration setup
  - **All tests must pass**: All Lexer struct tests plus all previous tests
  - **Ref**: FR-1, Design Section "Lexer"
  - **Tests**: Unit tests for Token creation, Debug output, PartialEq, and position tracking
  - **All tests must pass**: All Token tests plus all previous tests (Value system, etc.)
  - **Constraint**: Token definition only, no lexer logic yet

- [ ] **T1.3.2**: Implement `Lexer` struct
  - Create lexer with input, position, line, column fields
  - Implement character-by-character scanning
  - Add whitespace and comment handling
  - **Ref**: FR-1, Design Section "Lexer"

- [ ] **T1.3.3**: Implement token recognition
  - Add number parsing (integers and floats)
  - Add string parsing with escape sequences
  - Add symbol and boolean recognition
  - Handle parentheses and quotes
  - **Tests**: Unit tests for each token type recognition, number parsing, string parsing, symbol parsing
  - **All tests must pass**: Comprehensive lexer functionality tests plus all previous tests
  - **Ref**: FR-1

- [ ] **T1.3.4**: Add lexer error handling
  - Implement detailed error messages with position
  - Handle invalid characters and malformed tokens
  - Add recovery strategies for continued parsing
  - **Tests**: Unit tests for error conditions, invalid input handling, and error message formatting
  - **All tests must pass**: All lexer error handling tests plus all previous tests
  - **Tests**: Unit tests for syntax errors, unmatched parentheses, and error recovery
  - **All tests must pass**: All parser error handling tests plus all previous tests
  - **Tests**: Unit tests for unbound variables, error message quality, and shadowing detection
  - **All tests must pass**: All environment error tests plus all previous tests
  - **Ref**: FR-12

- [ ] **T1.3.5**: Create comprehensive lexer tests
  - Test all token types
  - Test error conditions and recovery
  - Test position tracking accuracy
  - Performance tests for large inputs
  - **All tests must pass**: Complete lexer test suite (30+ tests expected) plus all previous tests

---

## Phase 2: Basic Interpreter Functionality

### 2.1 Syntactic Analysis

- [ ] **T2.1.1**: Implement `Expr` enum
  - Create expression types: Atom, List, Quote
  - Add position information for error reporting
  - Implement `Debug` and `Clone` traits
  - **Tests**: Unit tests for Expr creation, Debug output, Clone functionality, and position tracking
  - **All tests must pass**: All Expr enum tests plus all previous tests (Lexer, Value system, etc.)
  - **Tests**: Unit tests for Parser creation, token consumption, and basic parsing setup
  - **All tests must pass**: All Parser struct tests plus all previous tests
  - **Ref**: FR-2, Design Section "Parser"

- [ ] **T2.1.2**: Implement `Parser` struct
  - Create parser with tokens and current position
  - Implement recursive descent parsing
  - Add expression parsing methods
  - **Ref**: FR-2, Design Section "Parser"

- [ ] **T2.1.3**: Implement expression parsing
  - Parse atoms (numbers, strings, symbols, booleans)
  - Parse lists and nested expressions
  - Handle quote expressions
  - **Tests**: Unit tests for atom parsing, list parsing, quote parsing, and nested expressions
  - **All tests must pass**: Comprehensive parser functionality tests plus all previous tests
  - **Ref**: FR-2

- [ ] **T2.1.4**: Add parser error handling
  - Implement syntax error reporting with position
  - Handle unmatched parentheses
  - Add error recovery for partial expressions
  - **Ref**: FR-12

- [ ] **T2.1.5**: Create comprehensive parser tests
  - Test all expression types
  - Test nested and complex expressions
  - Test error conditions and recovery
  - Performance tests for deeply nested expressions
  - **All tests must pass**: Complete parser test suite (25+ tests expected) plus all previous tests

### 2.2 Environment Management

- [ ] **T2.2.1**: Implement `Environment` struct
  - Create environment with bindings HashMap and optional parent
  - Implement lexical scoping chain
  - Add thread-safe sharing with `Arc<RwLock<Environment>>`
  - **Tests**: Unit tests for Environment creation, thread safety, and basic structure
  - **All tests must pass**: All Environment struct tests plus all previous tests (Parser, Lexer, etc.)
  - **Ref**: FR-7, FR-13, Design Section "Environment Management"

- [ ] **T2.2.2**: Implement environment operations
  - Add `new()`, `with_parent()`, `define()`, `lookup()` methods
  - Implement variable binding and lookup
  - Add environment extension for function calls
  - **Tests**: Unit tests for all environment operations, scoping behavior, and variable binding
  - **All tests must pass**: All environment operation tests plus all previous tests
  - **Tests**: Unit tests for let bindings, lexical scoping, and binding evaluation order
  - **All tests must pass**: All let binding tests plus all previous tests
  - **Ref**: FR-7, FR-13

- [ ] **T2.2.3**: Add environment error handling
  - Handle unbound variable errors
  - Implement detailed error messages
  - Add variable shadowing detection
  - **Tests**: Unit tests for error recovery, graceful handling, and continued operation
  - **All tests must pass**: All REPL error handling tests plus all previous tests
  - **Ref**: FR-12

- [ ] **T2.2.4**: Create environment tests
  - Test variable binding and lookup
  - Test lexical scoping behavior
  - Test environment chaining
  - Test thread safety
  - **All tests must pass**: Complete environment test suite (15+ tests expected) plus all previous tests

### 2.3 Basic Evaluation Engine

- [ ] **T2.3.1**: Implement basic `eval` function
  - Create evaluation for atoms (self-evaluating values)
  - Implement symbol lookup in environment
  - Add basic list evaluation framework
  - **Tests**: Unit tests for atom evaluation, symbol lookup, and basic evaluation framework
  - **All tests must pass**: All basic eval tests plus all previous tests (Environment, Parser, etc.)
  - **Ref**: Design Section "Evaluation Model"

- [ ] **T2.3.2**: Implement arithmetic operations
  - Add built-in procedures: +, -, *, /, =, <, >, <=, >=
  - Implement proper arity checking
  - Add type checking for numeric operations
  - **Tests**: Unit tests for each arithmetic operation, arity checking, and type validation
  - **All tests must pass**: All arithmetic operation tests plus all previous tests
  - **Ref**: FR-4, AC-1

- [ ] **T2.3.3**: Implement conditional expressions
  - Add `if` special form evaluation
  - Implement boolean evaluation logic
  - Add proper conditional flow control
  - **Tests**: Unit tests for if expressions, boolean evaluation, and conditional flow
  - **All tests must pass**: All conditional expression tests plus all previous tests
  - **Ref**: FR-6, AC-4

- [ ] **T2.3.4**: Implement basic list operations
  - Add built-in procedures: car, cdr, cons, list, null?
  - Implement proper list type checking
  - Add list construction and deconstruction
  - **Tests**: Unit tests for list operations, type checking, and list manipulation
  - **All tests must pass**: All list operation tests plus all previous tests
  - **Ref**: FR-5, AC-3

- [ ] **T2.3.5**: Create basic evaluation tests
  - Test arithmetic operations
  - Test conditional expressions
  - Test list operations
  - Test error handling for type mismatches
  - **All tests must pass**: Complete evaluation test suite (20+ tests expected) plus all previous tests

### 2.4 Variable Binding and Definition

- [ ] **T2.4.1**: Implement `define` special form
  - Add variable definition in current environment
  - Handle function definition syntax sugar
  - Implement proper scoping for definitions
  - **Tests**: Unit tests for variable definition, function definition syntax, and scoping
  - **All tests must pass**: All define form tests plus all previous tests
  - **Ref**: FR-7, AC-2

- [ ] **T2.4.2**: Implement `let` binding forms
  - Add `let` for local variable binding
  - Implement proper lexical scoping
  - Add binding evaluation order
  - **Ref**: FR-7, FR-13

- [ ] **T2.4.3**: Create variable binding tests
  - Test define functionality
  - Test let bindings and scoping
  - Test variable shadowing
  - Test binding error conditions
  - **All tests must pass**: Complete variable binding test suite (10+ tests expected) plus all previous tests

---

## Phase 3: Advanced Language Features

### 3.1 Function Definition and Application

- [ ] **T3.1.1**: Implement `Procedure` enum
  - Create Builtin and Lambda variants
  - Add parameter lists and body storage
  - Implement closure capture
  - **Tests**: Unit tests for Procedure enum, closure capture, and parameter handling
  - **All tests must pass**: All Procedure tests plus all previous tests (requires updating Value enum)
  - **Ref**: FR-8, Design Section "Value System"

- [ ] **T3.1.2**: Implement `lambda` special form
  - Add lambda expression parsing and evaluation
  - Implement closure creation with environment capture
  - Add parameter binding logic
  - **Tests**: Unit tests for lambda creation, closure behavior, and parameter binding
  - **All tests must pass**: All lambda tests plus all previous tests
  - **Ref**: FR-8, AC-2

- [ ] **T3.1.3**: Implement function application
  - Add procedure call evaluation
  - Implement argument evaluation and binding
  - Add arity checking for all procedure types
  - **Tests**: Unit tests for function calls, argument evaluation, and arity checking
  - **All tests must pass**: All function application tests plus all previous tests
  - **Ref**: FR-8, Design Section "Execution Engine"

- [ ] **T3.1.4**: Implement tail call optimization
  - Add tail position detection
  - Implement tail call elimination
  - Optimize recursive function calls
  - **Tests**: Unit tests for tail call detection, recursive optimization, and stack usage
  - **All tests must pass**: All tail call optimization tests plus all previous tests
  - **Ref**: Design Section "Tail Call Optimization"

- [ ] **T3.1.5**: Create function system tests
  - Test lambda creation and application
  - Test closure behavior and variable capture
  - Test recursive functions and tail calls
  - Test function definition with define
  - **Tests**: Integration tests covering all function scenarios from acceptance criteria
  - **All tests must pass**: Complete function system test suite (25+ tests expected) plus all previous tests
  - **Ref**: AC-2, AC-5

### 3.2 Advanced Built-in Procedures

- [ ] **T3.2.1**: Implement type checking procedures
  - Add: number?, boolean?, string?, symbol?, list?, procedure?, null?
  - Implement proper type predicate logic
  - Add comprehensive type checking
  - **Tests**: Unit tests for each type predicate, comprehensive type checking scenarios
  - **All tests must pass**: All type checking procedure tests plus all previous tests
  - **Tests**: Unit tests for I/O procedures, output formatting, and input handling
  - **All tests must pass**: All I/O procedure tests plus all previous tests
  - **Ref**: FR-11

- [ ] **T3.2.2**: Implement advanced list operations
  - Add: length, append, reverse, member, assoc
  - Implement list transformation procedures
  - Add proper error handling for list operations
  - **Tests**: Unit tests for advanced list operations, edge cases, and error conditions
  - **All tests must pass**: All advanced list operation tests plus all previous tests
  - **Ref**: FR-5, FR-11

- [ ] **T3.2.3**: Implement I/O procedures (synchronous)
  - Add: display, newline, read (basic)
  - Implement string output formatting
  - Add basic input reading
  - **Ref**: FR-11

- [ ] **T3.2.4**: Create built-in procedure tests
  - Test all type checking procedures
  - Test advanced list operations
  - Test I/O procedures
  - Test error conditions
  - **All tests must pass**: Complete built-in procedures test suite (20+ tests expected) plus all previous tests

### 3.3 REPL Implementation

- [ ] **T3.3.1**: Implement basic REPL loop
  - Create read-eval-print loop structure
  - Add input reading and parsing
  - Implement expression evaluation and output
  - **Tests**: Integration tests for REPL loop, input/output, and basic interaction
  - **All tests must pass**: All basic REPL tests plus all previous tests
  - **Ref**: FR-9, AC-3

- [ ] **T3.3.2**: Add REPL error handling
  - Implement graceful error recovery
  - Add detailed error reporting
  - Continue REPL operation after errors
  - **Ref**: FR-12

- [ ] **T3.3.3**: Add REPL enhancements
  - Implement multi-line input support
  - Add basic command history
  - Add help and exit commands
  - **Tests**: Integration tests for multi-line input, command history, and help system
  - **All tests must pass**: All REPL enhancement tests plus all previous tests
  - **Ref**: NFR-3

- [ ] **T3.3.4**: Create REPL tests
  - Test basic REPL functionality
  - Test error recovery
  - Test multi-line input
  - Integration tests for user scenarios
  - **All tests must pass**: Complete REPL test suite (15+ tests expected) plus all previous tests

### 3.4 File Execution

- [ ] **T3.4.1**: Implement file reading and parsing
  - Add file input handling
  - Implement multi-expression parsing
  - Add proper file error handling
  - **Tests**: Unit tests for file reading, parsing multiple expressions, and file error handling
  - **All tests must pass**: All file parsing tests plus all previous tests
  - **Tests**: Unit tests for sequential evaluation, environment handling, and result collection
  - **All tests must pass**: All batch evaluation tests plus all previous tests
  - **Tests**: Integration tests for command-line interface, argument parsing, and execution modes
  - **All tests must pass**: All CLI tests plus all previous tests
  - **Ref**: FR-10

- [ ] **T3.4.2**: Implement batch evaluation
  - Add sequential expression evaluation
  - Implement proper environment handling
  - Add result collection and reporting
  - **Ref**: FR-10

- [ ] **T3.4.3**: Add command-line interface
  - Implement file execution from command line
  - Add command-line argument parsing
  - Add execution mode selection (REPL vs file)
  - **Ref**: FR-10

- [ ] **T3.4.4**: Create file execution tests
  - Test file parsing and execution
  - Test command-line interface
  - Test error handling for file operations
  - Integration tests with sample Scheme files
  - **All tests must pass**: Complete file execution test suite (12+ tests expected) plus all previous tests

---

## Phase 4: Concurrency and Async Features

### 4.1 Fiber Scheduler Infrastructure

- [ ] **T4.1.1**: Implement `Fiber` struct
  - Add async dependencies: `smol`, `futures-lite`, `async-channel` to `Cargo.toml` (smol ecosystem only)
  - Create `src/runtime/` module directory
  - Create fiber with id, state, continuation, parent fields
  - Implement `FiberState` enum (Ready, Running, Suspended, Completed)
  - Add `SuspendReason` enum (IoOperation, WaitingForTask, Yielded)
  - **Ref**: FR-14, Design Section "Fiber Scheduler"
  - **Tests**: Unit tests for Fiber creation, state transitions, and data structure integrity
  - **All tests must pass**: All Fiber struct tests plus all previous tests
  - **Constraint**: Data structures only, no scheduler logic yet

- [ ] **T4.1.2**: Implement `FiberScheduler` struct
  - Add `polling` and `async-task` dependencies to `Cargo.toml` for scheduler operations
  - Create scheduler with ready queue, fiber map, runtime, thread pool
  - Add main fiber management
  - DO NOT implement scheduling algorithms yet - just the data structure
  - **Ref**: FR-14, Design Section "Fiber Scheduler Architecture"
  - **Tests**: Unit tests for FiberScheduler creation, data structure setup, and basic state
  - **All tests must pass**: All FiberScheduler struct tests plus all previous tests
  - **Constraint**: Scheduler structure only, no execution logic yet

- [ ] **T4.1.3**: Implement fiber lifecycle management
  - Add `spawn_fiber()`, `yield_current()`, `resume_fiber()` methods
  - Implement fiber state transitions
  - Add fiber cleanup and resource management
  - **Tests**: Unit tests for fiber lifecycle methods, state management, and resource cleanup
  - **All tests must pass**: All fiber lifecycle tests plus all previous tests
  - **Ref**: FR-14, Design Section "Fiber and Task Lifecycle"

- [ ] **T4.1.4**: Implement scheduler main loop
  - Add `run_scheduler()` method with event loop
  - Implement fiber dispatch and execution
  - Add cooperative multitasking support
  - **Tests**: Integration tests for scheduler main loop, fiber dispatch, and cooperative multitasking
  - **All tests must pass**: All scheduler main loop tests plus all previous tests
  - **Ref**: FR-14

- [ ] **T4.1.5**: Create fiber scheduler tests
  - Test fiber creation and scheduling
  - Test state transitions and lifecycle
  - Test cooperative multitasking
  - Performance tests for scheduler overhead
  - **All tests must pass**: Complete fiber scheduler test suite (20+ tests expected) plus all previous tests

### 4.2 Async Task System

- [ ] **T4.2.1**: Implement `Task` and `TaskHandle` structs
  - Create task with handle, fiber_id, parent/child relationships, result
  - Implement `TaskHandle` with id and control methods
  - Add task hierarchy management
  - **Tests**: Unit tests for Task/TaskHandle creation, hierarchy management, and data integrity
  - **All tests must pass**: All Task system struct tests plus all previous tests
  - **Ref**: FR-15, Design Section "Async Task System"

- [ ] **T4.2.2**: Implement task operations
  - Add `wait()`, `is_finished()`, `cancel()` methods on TaskHandle
  - Implement task completion and result propagation
  - Add hierarchical task cancellation
  - **Tests**: Unit tests for task operations, completion handling, and hierarchical cancellation
  - **All tests must pass**: All task operation tests plus all previous tests
  - **Tests**: Integration tests for task-fiber coordination, scheduling, and prioritization
  - **All tests must pass**: All task-fiber integration tests plus all previous tests
  - **Ref**: FR-15

- [ ] **T4.2.3**: Integrate tasks with fiber scheduler
  - Connect task lifecycle to fiber execution
  - Implement task-fiber coordination
  - Add task scheduling and prioritization
  - **Ref**: FR-15

- [ ] **T4.2.4**: Create task system tests
  - Test task creation and execution
  - Test task hierarchy and cancellation
  - Test task-fiber integration
  - **Tests**: Comprehensive task system tests covering acceptance criteria scenarios
  - **All tests must pass**: Complete task system test suite (18+ tests expected) plus all previous tests
  - **Ref**: AC-8, AC-10

### 4.3 Asynchronous I/O Integration

- [ ] **T4.3.1**: Implement async I/O infrastructure
  - Create async I/O module with `smol` integration
  - Add `yield_for_io()` fiber suspension
  - Implement I/O operation queuing
  - **Tests**: Unit tests for async I/O setup, fiber suspension, and I/O operation queuing
  - **All tests must pass**: All async I/O infrastructure tests plus all previous tests
  - **Ref**: FR-14, Design Section "Asynchronous I/O and Fiber Integration"

- [ ] **T4.3.2**: Implement async built-in procedures
  - Add `display-async`, `read-line-async` procedures
  - Implement fiber-yielding I/O operations
  - Add proper error handling for async operations
  - **Tests**: Unit tests for async procedures, fiber yielding, and async error handling
  - **All tests must pass**: All async built-in procedure tests plus all previous tests
  - **Ref**: FR-14, AC-7

- [ ] **T4.3.3**: Implement async evaluation context
  - Modify eval to support async operations
  - Add async procedure call handling
  - Implement proper async error propagation
  - **Tests**: Integration tests for async evaluation, procedure calls, and error propagation
  - **All tests must pass**: All async evaluation context tests plus all previous tests
  - **Ref**: Design Section "Error Propagation in Async Context"

- [ ] **T4.3.4**: Create async I/O tests
  - Test async I/O operations
  - Test fiber yielding and resumption
  - Test concurrent I/O operations
  - **Tests**: Integration tests for async I/O scenarios from acceptance criteria
  - **All tests must pass**: Complete async I/O test suite (15+ tests expected) plus all previous tests
  - **Ref**: AC-7

### 4.4 Built-in Fiber and Task Procedures

- [ ] **T4.4.1**: Implement fiber management procedures
  - Add: `spawn-fiber`, `yield`, `current-fiber`, `fiber-status`
  - Implement fiber control and introspection
  - Add proper error handling for fiber operations
  - **Tests**: Unit tests for each fiber management procedure and error handling
  - **All tests must pass**: All fiber management procedure tests plus all previous tests
  - **Ref**: Design Section "Built-in Fiber and Task Procedures", AC-11

- [ ] **T4.4.2**: Implement task management procedures
  - Add: `spawn-task`, `task-wait`, `task-cancel`, `task-result`
  - Implement task creation and control
  - Add task hierarchy management procedures
  - **Tests**: Unit tests for task management procedures and hierarchy operations
  - **All tests must pass**: All task management procedure tests plus all previous tests
  - **Ref**: Design Section "Built-in Fiber and Task Procedures"

- [ ] **T4.4.3**: Implement coordination procedures
  - Add: `parallel`, `sequential`, `race`, `timeout`
  - Implement high-level concurrency patterns
  - Add proper resource cleanup
  - **Tests**: Integration tests for coordination patterns, resource cleanup, and high-level concurrency
  - **All tests must pass**: All coordination procedure tests plus all previous tests
  - **Ref**: AC-10

- [ ] **T4.4.4**: Create concurrency procedure tests
  - Test all fiber and task procedures
  - Test coordination patterns
  - Test error handling and cleanup
  - **Tests**: Comprehensive concurrency procedure tests covering all acceptance criteria
  - **All tests must pass**: Complete concurrency procedures test suite (25+ tests expected) plus all previous tests
  - **Ref**: AC-8, AC-10, AC-11

---

## Phase 5: Macro System and Polish

### 5.1 Macro System Infrastructure

- [ ] **T5.1.1**: Implement pattern matching system
  - Create `Pattern` enum (Literal, Variable, List, Ellipsis)
  - Implement pattern matching algorithms
  - Add pattern variable binding
  - **Tests**: Unit tests for pattern matching algorithms, variable binding, and edge cases
  - **All tests must pass**: All pattern matching tests plus all previous tests
  - **Ref**: FR-16, Design Section "Macro System"

- [ ] **T5.1.2**: Implement template system
  - Create `Template` enum (Literal, Variable, List, Substitution)
  - Implement template expansion with substitution
  - Add proper variable substitution logic
  - **Tests**: Unit tests for template expansion, substitution logic, and template validation
  - **All tests must pass**: All template system tests plus all previous tests
  - **Tests**: Unit tests for macro rule application, macro expansion, and rule conflict handling
  - **All tests must pass**: All macro rule tests plus all previous tests
  - **Tests**: Unit tests for define-syntax parsing, macro registration, and disambiguation
  - **All tests must pass**: All define-syntax tests plus all previous tests
  - **Tests**: Integration tests for macro expansion phase, recursive expansion, and context management
  - **All tests must pass**: All macro integration tests plus all previous tests
  - **Ref**: FR-16

- [ ] **T5.1.3**: Implement `MacroRule` and `Macro` structs
  - Create macro rule with pattern and template
  - Implement macro with name and rules list
  - Add macro expansion logic
  - **Ref**: FR-16

- [ ] **T5.1.4**: Create macro system tests
  - Test pattern matching
  - Test template expansion
  - Test macro rule application
  - Test error handling
  - **All tests must pass**: Complete macro system infrastructure test suite (15+ tests expected) plus all previous tests

### 5.2 Macro Integration

- [ ] **T5.2.1**: Implement `define-syntax` special form
  - Add macro definition parsing
  - Implement macro registration in environment
  - Add macro vs procedure disambiguation
  - **Ref**: FR-16

- [ ] **T5.2.2**: Integrate macros with evaluation
  - Add macro expansion phase before evaluation
  - Implement recursive macro expansion
  - Add expansion context management
  - **Ref**: FR-16

- [ ] **T5.2.3**: Implement standard macros
  - Add `when`, `unless`, `cond` macros
  - Implement `let*`, `letrec` binding macros
  - Add `async` macro for task creation
  - **Tests**: Unit tests for each standard macro, expansion correctness, and usage scenarios
  - **All tests must pass**: All standard macro tests plus all previous tests
  - **Ref**: AC-12, Design examples

- [ ] **T5.2.4**: Create macro integration tests
  - Test define-syntax functionality
  - Test macro expansion and evaluation
  - Test standard macro library
  - **Tests**: Integration tests for macro integration scenarios from acceptance criteria
  - **All tests must pass**: Complete macro integration test suite (12+ tests expected) plus all previous tests
  - **Ref**: AC-12

### 5.3 Performance Optimization and Polish

- [ ] **T5.3.1**: Implement performance optimizations
  - Add tail call optimization improvements
  - Implement symbol interning for faster comparisons
  - Add memory pool allocation for common objects
  - **Tests**: Performance tests for optimization effectiveness and correctness validation
  - **All tests must pass**: All performance optimization tests plus all previous tests (no regression)
  - **Ref**: NFR-1, Design Section "Performance Optimizations"

- [ ] **T5.3.2**: Add resource management and limits
  - Implement `ResourceLimits` struct
  - Add stack depth and memory usage tracking
  - Implement execution timeouts and limits
  - **Tests**: Unit tests for resource limits, tracking accuracy, and limit enforcement
  - **All tests must pass**: All resource management tests plus all previous tests
  - **Ref**: Design Section "Security Considerations"

- [ ] **T5.3.3**: Improve error messages and debugging
  - Add detailed stack traces for errors
  - Implement better error message formatting
  - Add debugging information and introspection
  - **Tests**: Integration tests for error message quality, stack traces, and debugging features
  - **All tests must pass**: All error/debugging improvement tests plus all previous tests
  - **Tests**: Documentation examples should compile and run correctly as tests
  - **All tests must pass**: Documentation validation tests plus all previous tests
  - **Ref**: NFR-3

- [ ] **T5.3.4**: Create comprehensive performance tests
  - Add benchmarks for all major operations
  - Test memory usage and performance characteristics
  - Profile and optimize critical paths
  - **Tests**: Comprehensive performance benchmarks and memory usage validation
  - **All tests must pass**: All performance tests plus all previous tests (complete test suite)
  - **Ref**: Design Section "Performance Characteristics"

### 5.4 Final Integration and Testing

- [ ] **T5.4.1**: Create comprehensive integration tests
  - Test all acceptance criteria scenarios
  - Create complex multi-feature test cases
  - Add regression test suite
  - **Tests**: Complete integration test suite covering every acceptance criteria scenario
  - **All tests must pass**: Full acceptance test suite (50+ tests expected) plus all previous tests
  - **Ref**: All AC-* requirements

- [ ] **T5.4.2**: Add documentation and examples
  - Add `clap` dependency for command-line interface (final dependency)
  - Create user documentation and examples
  - Add API documentation for all modules
  - Create tutorial and getting started guide
  - **Ref**: NFR-3

- [ ] **T5.4.3**: Final performance validation
  - Run complete benchmark suite
  - Validate all non-functional requirements
  - Profile memory usage and optimize
  - **Tests**: Final validation tests for all non-functional requirements
  - **All tests must pass**: Complete final validation test suite plus all previous tests
  - **Ref**: All NFR-* requirements

- [ ] **T5.4.4**: Release preparation
  - Clean up code and remove debug artifacts
  - Finalize configuration and build scripts
  - Prepare distribution package
  - **Tests**: Release validation tests, build verification, and distribution testing
  - **All tests must pass**: Final release validation tests - COMPLETE PROJECT TEST SUITE (300+ tests expected)
  - **Ref**: NFR-4, NFR-5

---

## Testing Strategy

### Unit Tests
Each module should have comprehensive unit tests covering:
- Normal operation cases
- Edge cases and error conditions  
- Performance characteristics
- Memory usage patterns
- **Required**: All unit tests must pass before task completion

### Integration Tests
- End-to-end scenarios matching acceptance criteria
- REPL interaction tests
- File execution tests
- Concurrency and async operation tests
- **Required**: All integration tests must pass before task completion

### Performance Tests
- Benchmarks for core operations
- Memory usage profiling
- Concurrency overhead measurement
- Regression detection
- **Required**: Performance tests must validate no regression from previous tasks

### Acceptance Tests
Each acceptance criteria (AC-1 through AC-12) should have corresponding automated tests that verify the exact scenarios described in `requirements.md`.
- **Required**: All acceptance tests must pass when their corresponding features are implemented

### Test Execution Validation
After each task completion, run the complete test suite:
```bash
cargo test                    # All tests must pass
cargo test --release         # Release mode tests must pass
cargo bench                  # Benchmarks must run without errors (when applicable)
```

### Test Coverage Expectations
- **Phase 1**: 50+ tests (foundation)
- **Phase 2**: 120+ tests (basic interpreter)
- **Phase 3**: 200+ tests (advanced features)
- **Phase 4**: 280+ tests (concurrency)
- **Phase 5**: 300+ tests (complete system)

---

## Dependencies and References

### External Dependencies (Add Only When Needed)

**Core Dependencies (minimal, essential only):**
- `thiserror` - Error handling with zero dependencies (Phase 1)
- `smol` - Main async runtime from smol ecosystem (Phase 4)
- `futures-lite` - Async utilities from smol ecosystem (Phase 4)
- `async-channel` - Async communication from smol ecosystem (Phase 4)
- `polling` - Event polling for I/O from smol ecosystem (Phase 4)
- `async-task` - Task management from smol ecosystem (Phase 4)

**Non-core Dependencies (add only if absolutely necessary):**
- `clap` - Command line parsing (Phase 5, only if CLI features are implemented)

**Dependency Philosophy:**
- All async-related crates must come from the smol ecosystem: https://github.com/smol-rs
- Prefer single-purpose, minimal crates over feature-rich alternatives
- Each dependency must be justified and have minimal sub-dependencies
- Remove non-smol async crates from consideration

**Important**: Each dependency should be added to `Cargo.toml` only when the task specifically requires it. Do not add all dependencies at project initialization.

### Local Dependency Source Management

**Purpose**: Enable AI agents to access accurate source code and documentation for all project dependencies without network access.

**Setup Commands**:
```bash
# Initial setup
mkdir -p deps/{vendor,docs,registry}
echo "deps/" >> .gitignore
cargo vendor deps/vendor
cargo doc --all-features --document-private-items --workspace
cp -r target/doc/* deps/docs/

# Maintenance (run after Cargo.toml changes)
cargo vendor deps/vendor --sync Cargo.toml
cargo doc --all-features --document-private-items --workspace --force-rebuild
cp -r target/doc/* deps/docs/
```

**Benefits**:
- Complete offline access to dependency source code for AI analysis
- Version-locked sources ensure consistency with Cargo.lock
- Generated documentation includes private implementation details
- No network dependency during development or AI assistance

### Internal References
- **Requirements**: See `requirements.md` for FR-* and NFR-* specifications
- **Design**: See `design.md` for detailed technical specifications
- **Acceptance Criteria**: See `requirements.md` AC-* sections for validation requirements

---

## Task Status Legend

- [ ] **Not Started** - Task not yet begun
- [x] **Completed** - Task finished and tested
- [🔄] **In Progress** - Task currently being worked on
- [⚠️] **Blocked** - Task waiting on dependencies or issues

## Progress Tracking

**Phase 1**: ☐ Not Started  
**Phase 2**: ☐ Not Started  
**Phase 3**: ☐ Not Started  
**Phase 4**: ☐ Not Started  
**Phase 5**: ☐ Not Started  

**Overall Progress**: 0% (0/87 tasks completed)