# Twine Scheme Interpreter - Implementation Tasks

## Quick Reference

### Current Status
- **Phase 1.1**: ✅ **COMPLETE** (4/4 tasks) - Project Setup
- **Phase 1.2**: ☐ Not Started (0/5 tasks) - Core Data Types
- **Overall Progress**: 5% (4/81 tasks completed)

### Next Priority
**→ T1.2.1**: Implement basic `Value` enum

### Phase Overview
| Phase | Focus | Tasks | Est. Duration |
|-------|-------|-------|---------------|
| **Phase 1** | Foundation | 14 tasks | 2-3 weeks |
| **Phase 2** | Basic Interpreter | 20 tasks | 3-4 weeks |
| **Phase 3** | Advanced Features | 20 tasks | 3-4 weeks |
| **Phase 4** | Concurrency | 16 tasks | 3-4 weeks |
| **Phase 5** | Polish & Macros | 11 tasks | 2-3 weeks |

### Critical Rules
- ✅ **All tests must pass** after each task
- ⚠️ **Minimal implementation** - only current task features
- 📦 **Add dependencies only when needed** - not all at once
- 🧪 **smol ecosystem only** for async dependencies

---

## Task Guidelines

### Critical Constraints

#### Minimal Implementation Principle
- ⚠️ **ONLY** implement features described in current task
- 🚫 **NO** forward-looking code or stubs for future features
- ✅ **YES** to simple, working implementations
- 📝 **DOCUMENT** all design decisions for learning

#### All-Tests-Passing Requirement
```bash
# After EVERY task completion:
cargo test                 # All tests must pass
cargo test --release      # Release mode must pass
cargo bench              # Benchmarks must run (when applicable)
```

#### Dependency Management
- **Add dependencies ONLY when task requires them**
- **All async crates MUST be from smol ecosystem**
- **Justify each dependency** with task requirements
- **Update `./scripts/update-deps.sh`** after any Cargo.toml changes

#### Documentation Compliance (MANDATORY)
- ✅ **ALWAYS** read and follow `requirements.md` for functional requirements
- ✅ **ALWAYS** read and follow `design.md` for technical architecture
- ✅ **VERIFY** all implementations align with documented specifications
- ✅ **REFERENCE** specific requirement numbers (FR-X, NFR-X) in task completion
- ✅ **CHECK** acceptance criteria (AC-X) before marking tasks complete
- ⚠️ **NO DEVIATIONS** from documented specifications without explicit approval
- 📋 **CITE** relevant requirement/design sections when implementing features

```bash
# Before starting ANY task:
1. Read relevant sections in requirements.md
2. Read relevant sections in design.md
3. Understand acceptance criteria
4. Verify task dependencies are satisfied
5. Implement according to specifications
```

### Module Organization Principles
- **Single Responsibility**: Each module has one clear purpose
- **Domain Alignment**: Structure reflects interpreter concepts
- **Educational Clarity**: Code organization supports learning
- **Minimal Dependencies**: Clean interfaces between modules

---

## Phase 1: Core Language Foundation

### 1.1 Project Setup and Infrastructure ✅ COMPLETE

#### T1.1.1: Initialize Rust project structure ✅
**Status**: ✅ Complete
- Created basic `Cargo.toml` with project metadata
- Basic `src/main.rs` and `src/lib.rs` files
- **Constraint**: No external dependencies, no premature modules

#### T1.1.2: Set up basic error handling infrastructure ✅
**Status**: ✅ Complete
- Added `thiserror` dependency (minimal, zero-dependency choice)
- Implemented basic `Error` enum with essential variants
- Created `Result<T>` type alias
- **Tests**: Unit tests for error creation and Display implementation

#### T1.1.3: Set up local dependency source management ✅
**Status**: ✅ Complete
- Created `deps/` directory structure for AI agent access
- Set up vendor management and documentation generation
- Added to `.gitignore` to prevent committing large sources
- **Purpose**: Enable AI agents to reference accurate dependency code

#### T1.1.4: Create basic test framework structure ✅
**Status**: ✅ Complete
- Created `tests/` directory for integration tests
- Added basic unit test setup in `src/lib.rs`
- **Tests**: Created `tests/basic_integration.rs` with validation tests

### 1.2 Core Data Types and Value System

#### T1.2.1: Implement basic `Value` enum ✅
**Priority**: ✅ **COMPLETE**
**Prerequisites**: Error handling infrastructure ✅
**Deliverables**: ✅ **ALL COMPLETE**
- ✅ Create `src/types.rs` module
- ✅ Implement `Value` enum with: `Number`, `Boolean`, `String`, `Symbol`, `Nil`
- ✅ **DID NOT add** `List`, `Procedure`, `TaskHandle` variants yet
- ✅ Implement `Clone`, `Debug`, `PartialEq` traits
- ✅ Add basic constructor methods

**Tests Required**:
```rust
#[test] fn test_value_creation()
#[test] fn test_value_debug_output()
#[test] fn test_value_equality()
#[test] fn test_value_cloning()
```

**Acceptance**: ✅ All Value enum tests + all previous tests pass
**References**: FR-3, Design Section "Value System"

#### T1.2.2: Implement immutable number type ✅
**Priority**: ✅ **COMPLETE**
**Prerequisites**: Basic Value enum ✅
**Deliverables**: ✅ **ALL COMPLETE**
- ✅ Define `SchemeNumber` type (f64 wrapper)
- ✅ **DID NOT implement** arithmetic operations yet
- ✅ Add number parsing and basic formatting only

**Tests Required**:
```rust
#[test] fn test_number_parsing()
#[test] fn test_number_formatting()
#[test] fn test_number_equality()
#[test] fn test_number_edge_cases() // infinity, NaN
```

**References**: FR-4, Design Section "Immutable Value Design"

#### T1.2.3: Implement immutable string and symbol types ✅
**Prerequisites**: Number type implementation
**Deliverables**:
- Define `String` and `Symbol` types
- ⚠️ **DO NOT implement** symbol interning yet
- Basic equality and hashing only

**Tests Required**:
```rust
#[test] fn test_string_creation()
#[test] fn test_symbol_creation()
#[test] fn test_string_symbol_equality()
#[test] fn test_string_symbol_hashing()
```

#### T1.2.4: Implement immutable list type ✅
**Prerequisites**: String and symbol types
**Deliverables**:
- Define `List` using simple `Vec<Value>`
- ⚠️ **DO NOT implement** list operations (car, cdr, cons)
- ⚠️ **DO NOT add** structural sharing (Arc) yet
- Basic construction and display only

**Tests Required**:
```rust
#[test] fn test_list_creation()
#[test] fn test_list_display()
#[test] fn test_list_equality()
#[test] fn test_empty_list()
```

#### T1.2.5: Add comprehensive value system tests ✅
**Prerequisites**: All basic types implemented
**Deliverables**:
- Comprehensive test suite for all Value variants
- Edge case testing
- Error condition testing

**Acceptance**: 20+ tests covering complete value system

### 1.3 Lexical Analysis

#### T1.3.1: Implement `Token` enum ✅
**Prerequisites**: Value system complete
**Deliverables**:
- Create `src/lexer.rs` module
- Token types: `LeftParen`, `RightParen`, `Quote`, `Number`, `String`, `Symbol`, `Boolean`, `EOF`
- Add position tracking (line, column)
- Implement `Debug` and `PartialEq` traits

**Educational Note**: Uses owned `String` types for simplicity and learning focus. Zero-copy optimization with `&str` slices will be implemented in T5.3.1a for performance learning.

**Tests Required**:
```rust
#[test] fn test_token_creation()
#[test] fn test_token_debug_output()
#[test] fn test_token_equality()
#[test] fn test_position_tracking()
```

#### T1.3.2: Implement `Lexer` struct ✅
**Prerequisites**: Token enum
**Deliverables**:
- Create lexer with input, position, line, column fields
- Character-by-character scanning infrastructure
- Whitespace and comment handling

**Tests Required**:
```rust
#[test] fn test_lexer_creation()
#[test] fn test_position_tracking()
#[test] fn test_whitespace_handling()
#[test] fn test_comment_handling()
```

#### T1.3.3: Implement token recognition ✅
**Prerequisites**: Lexer struct
**Deliverables**:
- Number parsing (integers and floats)
- String parsing with escape sequences
- Symbol and boolean recognition
- Parentheses and quote handling

**Tests Required**:
```rust
#[test] fn test_number_tokenization()
#[test] fn test_string_tokenization()
#[test] fn test_symbol_tokenization()
#[test] fn test_boolean_tokenization()
#[test] fn test_delimiter_tokenization()
```

#### T1.3.4: Add lexer error handling ✅
**Prerequisites**: Token recognition
**Deliverables**:
- Detailed error messages with position
- Invalid character handling
- Error recovery strategies

**Tests Required**:
```rust
#[test] fn test_syntax_errors()
#[test] fn test_invalid_characters()
#[test] fn test_error_recovery()
#[test] fn test_error_positions()
```

#### T1.3.5: Create comprehensive lexer tests ✅
**Prerequisites**: Complete lexer implementation
**Acceptance**: 30+ tests covering all token types and error conditions
**Status**: ✅ COMPLETE - 34 tests implemented covering all token types, error conditions, edge cases, and position tracking

---

## Phase 2: Basic Interpreter Functionality

### 2.1 Syntactic Analysis

#### T2.1.1: Implement `Expr` enum ✅
**Prerequisites**: Complete lexer
**Deliverables**:
- Create `src/parser.rs` module
- Expression types: `Atom`, `List`, `Quote`
- Position information for error reporting
- `Debug` and `Clone` traits

**Compliance**: FR-2 (Syntactic Analysis)
**Tests**: 13 passing parser tests + 5 doctests
**Completed**: AST foundation with `Expression` and `PositionedExpression` types

#### T2.1.2: Implement `Parser` struct
**Deliverables**:
- Parser with tokens and current position
- Recursive descent parsing infrastructure
- Expression parsing methods

#### T2.1.3: Implement expression parsing
**Deliverables**:
- Parse atoms (numbers, strings, symbols, booleans)
- Parse lists and nested expressions
- Handle quote expressions

#### T2.1.4: Add parser error handling
**Deliverables**:
- Syntax error reporting with position
- Unmatched parentheses handling
- Error recovery for partial expressions

#### T2.1.5: Create comprehensive parser tests
**Acceptance**: 25+ tests covering all expression types and error conditions

### 2.2 Environment Management

#### T2.2.1: Implement `Environment` struct
**Prerequisites**: Parser complete
**Deliverables**:
- Create `src/interpreter/environment.rs`
- Environment with bindings HashMap and optional parent
- Thread-safe sharing with appropriate synchronization

#### T2.2.2: Implement environment operations
**Deliverables**:
- `new()`, `with_parent()`, `define()`, `lookup()` methods
- Variable binding and lookup
- Environment extension for function calls

#### T2.2.3: Add environment error handling
**Deliverables**:
- Unbound variable errors
- Detailed error messages
- Variable shadowing detection

#### T2.2.4: Create environment tests
**Acceptance**: 15+ tests covering scoping, binding, and thread safety

### 2.3 Basic Evaluation Engine

#### T2.3.1: Implement basic `eval` function
**Prerequisites**: Environment system complete
**Deliverables**:
- Create `src/interpreter/mod.rs`
- Evaluation for atoms (self-evaluating values)
- Symbol lookup in environment
- Basic list evaluation framework

#### T2.3.2: Implement arithmetic operations
**Deliverables**:
- Built-in procedures: `+`, `-`, `*`, `/`, `=`, `<`, `>`, `<=`, `>=`
- Arity checking
- Type checking for numeric operations

#### T2.3.3: Implement conditional expressions
**Deliverables**:
- `if` special form evaluation
- Boolean evaluation logic
- Conditional flow control

#### T2.3.4: Implement basic list operations
**Deliverables**:
- Built-in procedures: `car`, `cdr`, `cons`, `list`, `null?`
- List type checking
- List construction and deconstruction

#### T2.3.5: Create basic evaluation tests
**Acceptance**: 20+ tests covering arithmetic, conditionals, and list operations

### 2.4 Variable Binding and Definition

#### T2.4.1: Implement `define` special form
**Prerequisites**: Basic evaluation complete
**Deliverables**:
- Variable definition in current environment
- Function definition syntax sugar
- Proper scoping for definitions

#### T2.4.2: Implement `let` binding forms
**Deliverables**:
- `let` for local variable binding
- Lexical scoping implementation
- Binding evaluation order

#### T2.4.3: Create variable binding tests
**Acceptance**: 10+ tests covering define, let, and scoping behavior

---

## Phase 3: Advanced Language Features

### 3.1 Function Definition and Application

#### T3.1.1: Implement `Procedure` enum
**Prerequisites**: Variable binding complete
**Deliverables**:
- Create `Builtin` and `Lambda` variants
- Parameter lists and body storage
- Closure capture implementation

#### T3.1.2: Implement `lambda` special form
**Deliverables**:
- Lambda expression parsing and evaluation
- Closure creation with environment capture
- Parameter binding logic

#### T3.1.3: Implement function application
**Deliverables**:
- Procedure call evaluation
- Argument evaluation and binding
- Arity checking for all procedure types

#### T3.1.4: Implement tail call optimization
**Deliverables**:
- Tail position detection
- Tail call elimination
- Recursive function optimization

#### T3.1.5: Create function system tests
**Acceptance**: 25+ tests covering lambda creation, application, recursion, and tail calls

### 3.2 Advanced Built-in Procedures

#### T3.2.1: Implement type checking procedures
**Prerequisites**: Function system complete
**Deliverables**:
- `number?`, `boolean?`, `string?`, `symbol?`, `list?`, `procedure?`, `null?`
- Comprehensive type checking logic

#### T3.2.2: Implement advanced list operations
**Deliverables**:
- `length`, `append`, `reverse`, `member`, `assoc`
- List transformation procedures
- Error handling for list operations

#### T3.2.3: Implement I/O procedures (synchronous)
**Deliverables**:
- `display`, `newline`, `read` (basic)
- String output formatting
- Basic input reading

#### T3.2.4: Create built-in procedure tests
**Acceptance**: 20+ tests covering type checking, list operations, and I/O

### 3.3 REPL Implementation

#### T3.3.1: Implement basic REPL loop
**Prerequisites**: Built-in procedures complete
**Deliverables**:
- Read-eval-print loop structure
- Input reading and parsing
- Expression evaluation and output

#### T3.3.2: Add REPL error handling
**Deliverables**:
- Graceful error recovery
- Detailed error reporting
- Continued operation after errors

#### T3.3.3: Add REPL enhancements
**Deliverables**:
- Multi-line input support
- Basic command history
- Help and exit commands

#### T3.3.4: Create REPL tests
**Acceptance**: 15+ tests covering REPL functionality and error recovery

### 3.4 File Execution

#### T3.4.1: Implement file reading and parsing
**Prerequisites**: REPL complete
**Deliverables**:
- File input handling
- Multi-expression parsing
- File error handling

#### T3.4.2: Implement batch evaluation
**Deliverables**:
- Sequential expression evaluation
- Environment handling
- Result collection and reporting

#### T3.4.3: Add command-line interface
**Deliverables**:
- File execution from command line
- Command-line argument parsing
- Execution mode selection

#### T3.4.4: Create file execution tests
**Acceptance**: 12+ tests covering file parsing, execution, and CLI

---

## Phase 4: Concurrency and Async Features

### 4.1 Fiber Scheduler Infrastructure

#### T4.1.1: Implement `Fiber` struct
**Prerequisites**: File execution complete
**Dependencies**: Add `smol`, `futures-lite`, `async-channel` to Cargo.toml
**Deliverables**:
- Create `src/runtime/` module
- Fiber with id, state, continuation, parent fields
- `FiberState` enum (Ready, Running, Suspended, Completed)
- `SuspendReason` enum (IoOperation, WaitingForTask, Yielded)

#### T4.1.2: Implement `FiberScheduler` struct
**Dependencies**: Add `polling`, `async-task` to Cargo.toml
**Deliverables**:
- Scheduler with ready queue, fiber map, runtime, thread pool
- Main fiber management
- ⚠️ Data structure only, no scheduling logic yet

#### T4.1.3: Implement fiber lifecycle management
**Deliverables**:
- `spawn_fiber()`, `yield_current()`, `resume_fiber()` methods
- Fiber state transitions
- Resource cleanup

#### T4.1.4: Implement scheduler main loop
**Deliverables**:
- `run_scheduler()` method with event loop
- Fiber dispatch and execution
- Cooperative multitasking

#### T4.1.5: Create fiber scheduler tests
**Acceptance**: 20+ tests covering fiber creation, scheduling, and multitasking

### 4.2 Async Task System

#### T4.2.1: Implement `Task` and `TaskHandle` structs
**Prerequisites**: Fiber scheduler complete
**Deliverables**:
- Task with handle, fiber_id, parent/child relationships
- TaskHandle with control methods
- Task hierarchy management

#### T4.2.2: Implement task operations
**Deliverables**:
- `wait()`, `is_finished()`, `cancel()` methods
- Task completion and result propagation
- Hierarchical task cancellation

#### T4.2.3: Integrate tasks with fiber scheduler
**Deliverables**:
- Task lifecycle to fiber execution connection
- Task-fiber coordination
- Task scheduling and prioritization

#### T4.2.4: Create task system tests
**Acceptance**: 18+ tests covering task creation, hierarchy, and cancellation

### 4.3 Asynchronous I/O Integration

#### T4.3.1: Implement async I/O infrastructure
**Prerequisites**: Task system complete
**Deliverables**:
- Async I/O module with `smol` integration
- `yield_for_io()` fiber suspension
- I/O operation queuing

#### T4.3.2: Implement async built-in procedures
**Deliverables**:
- `display-async`, `read-line-async` procedures
- Fiber-yielding I/O operations
- Async error handling

#### T4.3.3: Implement async evaluation context
**Deliverables**:
- Modify eval to support async operations
- Async procedure call handling
- Async error propagation

#### T4.3.4: Create async I/O tests
**Acceptance**: 15+ tests covering async I/O and fiber yielding

### 4.4 Built-in Fiber and Task Procedures

#### T4.4.1: Implement fiber management procedures
**Prerequisites**: Async I/O complete
**Deliverables**:
- `spawn-fiber`, `yield`, `current-fiber`, `fiber-status`
- Fiber control and introspection
- Error handling for fiber operations

#### T4.4.2: Implement task management procedures
**Deliverables**:
- `spawn-task`, `task-wait`, `task-cancel`, `task-result`
- Task creation and control
- Task hierarchy management

#### T4.4.3: Implement coordination procedures
**Deliverables**:
- `parallel`, `sequential`, `race`, `timeout`
- High-level concurrency patterns
- Resource cleanup

#### T4.4.4: Create concurrency procedure tests
**Acceptance**: 25+ tests covering all fiber and task procedures

---

## Phase 5: Macro System and Polish

**Educational Focus**: This phase emphasizes learning advanced language features (macros) and performance optimization techniques. Earlier phases prioritized simplicity and learning core interpreter concepts, while Phase 5 demonstrates real-world optimization strategies including zero-copy parsing, memory management, and performance measurement.

### 5.1 Macro System Infrastructure

#### T5.1.1: Implement pattern matching system
**Prerequisites**: Concurrency system complete
**Deliverables**:
- Create `src/interpreter/macros.rs`
- `Pattern` enum (Literal, Variable, List, Ellipsis)
- Pattern matching algorithms
- Pattern variable binding

#### T5.1.2: Implement template system
**Deliverables**:
- `Template` enum (Literal, Variable, List, Substitution)
- Template expansion with substitution
- Variable substitution logic

#### T5.1.3: Implement `MacroRule` and `Macro` structs
**Deliverables**:
- Macro rule with pattern and template
- Macro with name and rules list
- Macro expansion logic

#### T5.1.4: Create macro system tests
**Acceptance**: 15+ tests covering pattern matching and template expansion

### 5.2 Macro Integration

#### T5.2.1: Implement `define-syntax` special form
**Prerequisites**: Macro system infrastructure complete
**Deliverables**:
- Macro definition parsing
- Macro registration in environment
- Macro vs procedure disambiguation

#### T5.2.2: Integrate macros with evaluation
**Deliverables**:
- Macro expansion phase before evaluation
- Recursive macro expansion
- Expansion context management

#### T5.2.3: Implement standard macros
**Deliverables**:
- `when`, `unless`, `cond` macros
- `let*`, `letrec` binding macros
- `async` macro for task creation

#### T5.2.4: Create macro integration tests
**Acceptance**: 12+ tests covering macro definition and expansion

### 5.3 Performance Optimization and Polish

#### T5.3.1: Implement performance optimizations
**Prerequisites**: Macro system complete
**Deliverables**:
- Symbol interning for faster comparisons
- Tail call optimization improvements
- Memory pool allocation for reduced GC pressure

#### T5.3.1a: Zero-copy lexing optimization
**Prerequisites**: T5.3.1
**Deliverables**:
- Convert `Token` enum to use `&str` slices instead of owned `String`
- Add lifetime parameters: `Token<'a>`, `PositionedToken<'a>`, `Lexer<'a>`
- Implement `Cow<str>` for tokens requiring escape sequence processing
- Update parser and all downstream components for lifetime compatibility
- Benchmark memory allocation reduction and tokenization speed improvements
- Maintain API compatibility through careful lifetime design

**Performance Targets**:
- Reduce lexer memory allocations by >90%
- Improve tokenization speed by 2-5x
- Measure impact on overall parsing performance

**Tests Required**:
```rust
#[test] fn test_zero_copy_tokenization()
#[test] fn test_lifetime_correctness()
#[test] fn test_performance_benchmarks()
#[test] fn test_escape_sequence_processing()
```

#### T5.3.2: Add resource management and limits
**Deliverables**:
- `ResourceLimits` struct
- Stack depth and memory tracking
- Execution timeouts and limits

#### T5.3.3: Improve error messages and debugging
**Deliverables**:
- Detailed stack traces
- Better error message formatting
- Debugging information and introspection

#### T5.3.4: Create comprehensive performance tests
**Acceptance**: Performance benchmarks and memory validation

### 5.4 Final Integration and Testing

#### T5.4.1: Create comprehensive integration tests
**Prerequisites**: All features complete
**Deliverables**:
- Test all acceptance criteria scenarios
- Complex multi-feature test cases
- Regression test suite

#### T5.4.2: Add documentation and examples
**Dependencies**: Add `clap` for CLI (final dependency)
**Deliverables**:
- User documentation and examples
- API documentation for all modules
- Tutorial and getting started guide

#### T5.4.3: Final performance validation
**Deliverables**:
- Complete benchmark suite
- Non-functional requirements validation
- Memory usage profiling

#### T5.4.4: Release preparation
**Deliverables**:
- Code cleanup and optimization
- Configuration and build scripts
- Distribution package preparation

**Final Acceptance**: 300+ tests covering complete system

---

## Testing Strategy

### Test Requirements by Phase
| Phase | Expected Tests | Focus Areas |
|-------|---------------|-------------|
| **Phase 1** | 50+ tests | Foundation components |
| **Phase 2** | 120+ tests | Basic interpreter functionality |
| **Phase 3** | 200+ tests | Advanced language features |
| **Phase 4** | 280+ tests | Concurrency and async operations |
| **Phase 5** | 300+ tests | Complete system integration |

### Test Categories

#### Unit Tests (Per Module)
```rust
// Example test structure
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_operation() { /* ... */ }

    #[test]
    fn test_edge_cases() { /* ... */ }

    #[test]
    fn test_error_conditions() { /* ... */ }

    #[test]
    fn test_performance_characteristics() { /* ... */ }
}
```

#### Integration Tests
- End-to-end scenarios matching acceptance criteria
- REPL interaction testing
- File execution testing
- Multi-component interactions

#### Acceptance Tests
Each acceptance criteria (AC-1 through AC-12) must have corresponding automated tests:

```rust
// Example acceptance test
#[test]
fn test_ac_1_basic_arithmetic() {
    let result = eval_string("(+ 1 2 3)");
    assert_eq!(result, Ok(Value::Number(6.0)));
}
```

### Test Execution Validation
```bash
# Required after each task:
cargo test                    # All tests must pass
cargo test --release         # Release mode validation
cargo clippy                  # Code quality checks
cargo fmt --check            # Formatting verification
```

---

## Dependencies and Constraints

### Dependency Addition Rules
1. **Add ONLY when task requires** - no bulk additions
2. **Justify each dependency** with specific task needs
3. **Prefer minimal alternatives** - avoid feature-rich crates
4. **Update local sources** with `./scripts/update-deps.sh`

### Approved Dependencies
| Dependency | Phase | Purpose | Ecosystem |
|------------|-------|---------|-----------|
| `thiserror` | 1 | Error handling | Minimal |
| `smol` | 4 | Async runtime | smol |
| `futures-lite` | 4 | Async utilities | smol |
| `async-channel` | 4 | Async communication | smol |
| `polling` | 4 | Event polling | smol |
| `async-task` | 4 | Task management | smol |
| `clap` | 5 | CLI parsing | Optional |

### Critical Constraints
- **All async dependencies MUST be from smol ecosystem**
- **No tokio, async-std, or other async runtimes**
- **Minimal dependency trees preferred**
- **Local source management required for AI agents**

---

## Progress Tracking

### Overall Status
**Current Phase**: Phase 1 (Foundation)
**Overall Progress**: 18% (15/82 tasks completed)
**Estimated Completion**: 12-16 weeks

### Phase Progress
- **Phase 1**: ✅ 100% (14/14 tasks) - Foundation COMPLETE
- **Phase 2**: ☐ 5% (1/20 tasks) - Basic Interpreter
- **Phase 3**: ☐ 0% (0/20 tasks) - Advanced Features
- **Phase 4**: ☐ 0% (0/16 tasks) - Concurrency
- **Phase 5**: ☐ 0% (0/12 tasks) - Polish & Macros

### Recently Completed
- ✅ T1.1.1: Initialize Rust project structure
- ✅ T1.1.2: Set up basic error handling infrastructure
- ✅ T1.1.3: Set up local dependency source management
- ✅ T1.1.4: Create basic test framework structure
- ✅ T1.2.1: Implement basic `Value` enum
- ✅ T1.2.2: Implement immutable number type
- ✅ T1.2.3: Implement immutable string and symbol types
- ✅ T1.2.4: Implement immutable list type
- ✅ T1.2.5: Add comprehensive value system tests
- ✅ T1.3.1: Implement `Token` enum
- ✅ T1.3.2: Implement `Lexer` struct
- ✅ T1.3.3: Implement token recognition
- ✅ T1.3.4: Add lexer error handling
- ✅ T1.3.5: Create comprehensive lexer tests
- ✅ T2.1.1: Implement `Expression` enum (FR-2 compliance)

### Immediate Next Steps
1. **T2.1.2**: Implement `Parser` struct (🔥 Priority - Continue Phase 2)
2. **T2.1.3**: Implement expression parsing
3. **T2.1.4**: Add parser error handling

### Blocked Tasks
None currently - clear path forward through Phase 1.

---

This task plan provides a structured, educational approach to building the Twine Scheme interpreter while maximizing learning value through AI-assisted development. Each task builds understanding progressively while maintaining rigorous testing and quality standards.
