# Twine Scheme Interpreter - Implementation Tasks

## Quick Reference

### Current Status
- **Phase 1**: ✅ **COMPLETE** (14/14 tasks) - Core Language Foundation
- **Phase 2.1**: ✅ **COMPLETE** (5/5 tasks) - Syntactic Analysis
- **Phase 2.2**: ✅ **COMPLETE** (4/4 tasks) - Environment Management
- **Phase 2.3**: ✅ **COMPLETE** (5/5 tasks) - Basic Evaluation Engine
- **Phase 2.4**: 🚧 **IN PROGRESS** (1/3 tasks) - Identifier Binding and Definition
- **Overall Progress**: 33% (27/81 tasks completed)

### Next Priority
**→ T2.4.2**: Implement `let` binding forms

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
- **Update `./scripts/vendor-deps.sh`** after any Cargo.toml changes

#### Documentation Compliance (MANDATORY)
- ✅ **ALWAYS** read and follow `REQUIREMENTS.md` for functional requirements
- ✅ **ALWAYS** read and follow `DESIGN.md` for technical architecture
- ✅ **VERIFY** all implementations align with documented specifications
- ✅ **REFERENCE** specific requirement numbers (FR-X, NFR-X) in task completion
- ✅ **CHECK** acceptance criteria (AC-X) before marking tasks complete
- ⚠️ **NO DEVIATIONS** from documented specifications without explicit approval
- 📋 **CITE** relevant requirement/design sections when implementing features

```bash
# Before starting ANY task:
1. Read relevant sections in REQUIREMENTS.md
2. Read relevant sections in DESIGN.md
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

#### T2.1.2: Implement `Parser` struct ✅
**Prerequisites**: Complete lexer and `Expression` enum
**Deliverables**:
- Parser with tokens and current position ✅
- Recursive descent parsing infrastructure ✅
- Expression parsing methods ✅

**Compliance**: FR-2 (Syntactic Analysis)
**Tests**: 13 passing parser struct tests + existing expression tests
**Completed**: Parser with recursive descent parsing for atoms, lists, and quotes

#### T2.1.3: Implement expression parsing ✅
**Prerequisites**: Parser struct complete
**Deliverables**:
- Parse atoms (numbers, strings, symbols, booleans) ✅
- Parse lists and nested expressions ✅
- Handle quote expressions ✅

**Compliance**: FR-2 (Syntactic Analysis)
**Tests**: Covered by T2.1.2 parser tests
**Completed**: All expression parsing implemented in T2.1.2

#### T2.1.4: Add parser error handling ✅
**Prerequisites**: Expression parsing complete
**Deliverables**:
- Syntax error reporting with position ✅
- Unmatched parentheses handling ✅
- Error recovery for partial expressions ✅

**Compliance**: FR-2 (Syntactic Analysis)
**Tests**: Covered by T2.1.2 error handling tests
**Completed**: Comprehensive error handling with position tracking

#### T2.1.5: Create comprehensive parser tests ✅
**Prerequisites**: Expression parsing and error handling complete
**Acceptance**: 25+ tests covering all expression types and error conditions

**Compliance**: FR-2 (Syntactic Analysis)
**Tests**: 24+ comprehensive parser tests covering atoms, lists, quotes, and error conditions
**Completed**: Comprehensive test suite with high coverage of parser functionality

### 2.2 Environment Management

#### T2.2.1: Implement `Environment` struct ✅
**Prerequisites**: Parser complete
**Deliverables**:
- Create `src/runtime/environment.rs`
- Environment with bindings HashMap and optional parent
- Thread-safe sharing with appropriate synchronization

#### T2.2.2: Implement environment operations ✅
**Deliverables**:
- `new()`, `with_parent()`, `define()`, `lookup()` methods
- Identifier binding and lookup
- Environment extension for function calls

#### T2.2.3: Add environment error handling ✅
**Deliverables**:
- Unbound identifier errors
- Detailed error messages
- Enhanced error context and suggestions

#### T2.2.4: Create environment tests ✅
**Acceptance**: 15+ tests covering scoping, binding, and thread safety

### 2.3 Basic Evaluation Engine

#### T2.3.1: Implement basic `eval` function ✅
**Prerequisites**: Environment system complete ✅
**Deliverables**:
- Create `src/runtime/mod.rs` ✅
- Evaluation for atoms (self-evaluating values) ✅
- Symbol lookup in environment ✅
- Basic list evaluation framework ✅

**Tests Added**:
```rust
fn test_eval_self_evaluating_atoms
fn test_eval_symbol_lookup
fn test_eval_unbound_symbol
fn test_eval_empty_list
fn test_eval_non_empty_list_not_implemented
fn test_eval_quote_atom
fn test_eval_quote_list
fn test_expression_to_value_conversion
fn test_nested_environment_symbol_lookup
fn test_eval_list_values
fn test_eval_integration // in lib.rs
```

#### T2.3.2: Implement arithmetic operations ✅
**Deliverables**:
- Built-in procedures: `+`, `-`, `*`, `/`, `=`, `<`, `>`, `<=`, `>=` ✅
- Arity checking ✅
- Type checking for numeric operations ✅

**Implementation Details**:
- Created `src/runtime/builtin/arithmetic.rs` with all arithmetic operations
- Created `src/runtime/builtin/mod.rs` for builtin module organization
- Updated `eval_list` to handle procedure calls with builtin arithmetic operations
- Comprehensive error handling for division by zero, type mismatches, arity violations

**Tests Added**:
```rust
// In arithmetic.rs
fn test_add
fn test_subtract
fn test_multiply
fn test_divide
fn test_equal
fn test_less_than
fn test_greater_than
fn test_less_than_or_equal
fn test_greater_than_or_equal
fn test_type_checking
fn test_edge_cases

// In eval.rs
fn test_eval_arithmetic_operations
fn test_eval_comparison_operations
fn test_eval_arithmetic_with_identifiers
fn test_eval_unknown_procedure
fn test_eval_non_symbol_procedure
```

#### T2.3.3: Implement conditional expressions ✅
**Prerequisites**: Arithmetic operations complete ✅
**Deliverables**:
- `if` special form evaluation ✅
- Boolean evaluation logic ✅
- Conditional flow control ✅

**Implementation Details**:
- Modified `eval_list` to handle special forms before builtin procedures
- Implemented `eval_if` function with proper arity checking (3 arguments required)
- Implemented `is_truthy` helper function following Scheme semantics (only #f is false)
- Added comprehensive error handling for invalid argument counts

**Tests Added**:
```rust
// In eval.rs
fn test_eval_if_true_condition
fn test_eval_if_false_condition
fn test_eval_if_truthiness
fn test_eval_if_with_expressions
fn test_eval_if_nested
fn test_eval_if_arity_errors
fn test_eval_if_evaluation_order
fn test_is_truthy_function

// In lib.rs
fn test_eval_integration // end-to-end if expression testing
```

#### T2.3.4: Implement basic list operations ✅
**Deliverables**:
- Built-in procedures: `car`, `cdr`, `cons`, `list`, `null?` ✅
- List type checking ✅
- List construction and deconstruction ✅

**File Structure Note**: This task should add list operations to the builtins directory:
```
runtime/
├── special_forms.rs    # if
└── builtins/
    ├── mod.rs          # dispatch (update to include list operations) ✅
    ├── arithmetic.rs   # +, -, *, /, =, <, >, etc.
    └── list.rs         # car, cdr, cons, list, null? ✅
```

**Implementation Notes**:
- Added `list.rs` with all 5 required functions with proper error handling
- Updated dispatch in `mod.rs` to include list operations
- Added comprehensive unit tests (12 test functions, 50+ assertions)
- Added integration tests in eval module
- All functions include arity checking and type validation
- Error messages are descriptive and follow Scheme conventions

**Tests Added**:
```rust
fn test_car                               // Normal car operation
fn test_car_errors                        // Error conditions for car
fn test_cdr                               // Normal cdr operation
fn test_cdr_errors                        // Error conditions for cdr
fn test_cons                              // Normal cons operation
fn test_cons_errors                       // Error conditions for cons
fn test_list                              // Normal list creation
fn test_null_p                            // null? predicate testing
fn test_null_p_errors                     // Error conditions for null?
fn test_list_operations_integration       // Integration between operations
fn test_edge_cases                        // Edge cases and boundaries
fn test_type_checking                     // Type error validation
fn test_eval_list_operations              // Integration with evaluator
fn test_eval_list_operations_with_variables // Variable usage
fn test_eval_list_operations_errors       // Evaluator error handling
```

#### T2.3.5: Create basic evaluation tests ✅
**Acceptance**: 20+ tests covering arithmetic, conditionals, and list operations
**Completed**: 26 comprehensive integration tests added covering:
- Basic list construction and access operations (car, cdr, cons, list, null?)
- Arithmetic operations with lists and variables
- Conditional expressions with list predicates
- Complex nested operations mixing arithmetic, conditionals, and lists
- Error conditions and edge cases
- Quote handling with lists
- Variable binding with list operations
**Tests cleaned up**: Removed redundant tests from runtime/eval.rs to avoid duplication

### 2.4 Identifier Binding and Definition

#### T2.4.1: Implement `define` special form ✅
**Prerequisites**: Basic evaluation complete
**Deliverables**:
- ✅ Identifier definition in current environment
- ✅ Function definition syntax sugar (placeholder until lambda implementation)
- ✅ Proper scoping for definitions

**File Structure Note**: ✅ Transitioned `special_forms.rs` to directory structure:
```
runtime/
├── special_forms/
│   ├── mod.rs          # dispatch system
│   ├── control_flow.rs # if (and future: cond, case, when, unless, begin)
│   └── binding.rs      # define (and future: let, let*, le
└── builtins/
    ├── mod.rs          # dispatch
    └── arithmetic.rs   # +, -, *, /, =, <, >, etc.
```

**Implementation Notes**:
- Variable definition: `(define identifier expression)` - evaluates expression and binds result
- Function definition: `(define (name param...) body...)` - syntactic sugar (placeholder for lambda)
- Returns `Nil` after successful definition
- Supports variable shadowing in current environment
- Comprehensive test coverage including error cases

#### T2.4.2: Implement `let` binding forms ✅
**Deliverables**:
- `let` for local identifier binding ✅
- Lexical scoping implementation ✅
- Binding evaluation order ✅

**Implementation Notes**:
- Added `eval_let` function in `runtime/special_forms/binding.rs`
- Implemented proper lexical scoping with `Environment::new_scope`
- Ensures simultaneous binding evaluation (all expressions evaluated in current environment before any bindings)
- Added comprehensive unit tests (9 tests) and integration tests (9 tests)
- Proper error handling for malformed let expressions

#### T2.4.3: Create identifier binding tests ✅
**Acceptance**: 10+ tests covering define, let, and scoping behavior ✅

**Implementation Notes**:
- 29+ comprehensive tests implemented covering define, let, and scoping behavior
- Unit tests: 7 for define, 9 for let (runtime/special_forms/binding.rs)
- Integration tests: 11 end-to-end tests (tests/integration.rs)
- Special form dispatch tests: 2 tests (runtime/special_forms/mod.rs)
- Coverage includes: basic functionality, lexical scoping, simultaneous binding, error cases, nested scoping, integration with other features

---

## Phase 3: Advanced Language Features

### 3.1 Function Definition and Application

#### T3.1.1: Implement `Procedure` enum ✅
**Prerequisites**: Identifier binding complete
**Deliverables**:
- Create `Builtin` and `Lambda` variants ✅
- Parameter lists and body storage ✅
- Closure capture implementation ✅

#### T3.1.2: Implement `lambda` special form ✅
**Deliverables**:
- Lambda expression parsing and evaluation ✅
- Closure creation with environment capture ✅
- Parameter binding logic ✅

**Implementation Details**:
- Created `lambda.rs` module in `special_forms/`
- Added `Lambda` variant to `SpecialForm` enum
- Implemented `eval_lambda` function with comprehensive parameter validation
- Added environment flattening for closure capture (resolves lifetime constraints)
- Comprehensive test coverage including edge cases and error conditions
- Integration tests verify end-to-end lambda creation functionality

**File Structure Updated**:
```
runtime/
├── special_forms/
│   ├── mod.rs          # dispatch (updated to include lambda) ✅
│   ├── control_flow.rs # if
│   ├── binding.rs      # define, let
│   └── lambda.rs       # lambda ✅
└── builtins/
    ├── mod.rs          # dispatch
    ├── arithmetic.rs   # +, -, *, /, =, <, >, etc.
    ├── list.rs         # car, cdr, cons, list, null?
    └── predicates.rs   # number?, boolean?, string?, etc.
```

**Tests Added**:
```
lambda::tests::test_lambda_no_parameters ✅
lambda::tests::test_lambda_single_parameter ✅ 
lambda::tests::test_lambda_multiple_parameters ✅
lambda::tests::test_lambda_environment_capture ✅
lambda::tests::test_lambda_arity_errors ✅
lambda::tests::test_lambda_parameter_validation_errors ✅
lambda::tests::test_lambda_duplicate_parameters ✅
lambda::tests::test_lambda_parameter_list_parsing ✅
lambda::tests::test_validate_parameters ✅
lambda::tests::test_lambda_edge_cases ✅
lambda::tests::test_lambda_procedure_display ✅
lambda::tests::test_lambda_with_keyword_parameter_names ✅
special_forms::tests::test_dispatch_lambda_special_form ✅
```

**Integration Tests**:
```
test_integration_lambda_creation_basic ✅
test_integration_lambda_creation_no_parameters ✅
test_integration_lambda_creation_multiple_parameters ✅
test_integration_lambda_environment_capture ✅
test_integration_lambda_error_cases ✅
test_integration_lambda_with_define ✅
test_integration_lambda_nested_environments ✅
test_integration_lambda_complex_body ✅
test_integration_lambda_with_special_form_names ✅
```

#### T3.1.3: Implement function application ✅
**Deliverables**:
- Procedure call evaluation ✅
- Argument evaluation and binding ✅
- Arity checking for all procedure types ✅
**Acceptance**: 12+ tests covering lambda calling, parameter binding, arity checking, closures
```
test_integration_lambda_application_basic ✅
test_integration_lambda_application_multiple_parameters ✅
test_integration_lambda_application_no_parameters ✅
test_integration_lambda_application_closure ✅
test_integration_lambda_application_nested_calls ✅
test_integration_lambda_application_with_lists ✅
test_integration_lambda_application_complex_expression ✅
test_integration_lambda_application_arity_errors ✅
test_integration_lambda_application_error_cases ✅
test_integration_lambda_application_parameter_shadowing ✅
test_integration_lambda_application_recursive_pattern ✅
test_integration_lambda_application_comprehensive ✅
```

#### T3.1.4: Implement tail call optimization ✅
**Deliverables**:
- Tail position detection ✅
- Tail call elimination ✅
- Recursive function optimization ✅

**Acceptance Criteria**: ✅ FR-8 (Procedure Application with tail-call optimization)
- ✅ **AC-8.1**: Lambda procedures detect tail position calls
- ✅ **AC-8.2**: Tail calls to lambda procedures use iterative evaluation  
- ✅ **AC-8.3**: Tail calls to builtin procedures are optimized
- ✅ **AC-8.4**: Non-tail calls continue to work correctly
- ✅ **AC-8.5**: Conditional expressions in tail position are handled properly

**Implementation Notes**:
- Enhanced `call_procedure` function with tail call optimization for lambda procedures
- Added `call_lambda_with_tco` function that uses iterative evaluation instead of recursion
- Added `detect_tail_call_in_expression` to identify procedure calls in tail position
- Comprehensive test coverage including unit tests and integration tests
- Prevents stack overflow for deeply recursive functions while maintaining correctness

#### T3.1.5: Create function system tests ✅
**Acceptance**: 25+ tests covering lambda creation, application, recursion, and tail calls
**Completed**: 45 comprehensive tests in `tests/integration_lambda.rs` covering:
- Lambda creation (29 existing tests)
- Lambda application (comprehensive coverage)
- Recursion (factorial, fibonacci, countdown, list operations, mutual recursion)
- Tail call optimization (basic, conditional, with let, recursive, procedure syntax)
- Higher-order functions (map, compose, curry)
- Complete function system integration tests

### 3.2 Advanced Built-in Procedures

#### T3.2.1: Implement type checking procedures
**Prerequisites**: Function system complete
**Deliverables**:
- `number?`, `boolean?`, `string?`, `symbol?`, `list?`, `procedure?`, `null?`
- Comprehensive type checking logic

**File Structure Note**: This task should add predicates to the builtins directory:
```
runtime/
├── special_forms/
│   ├── mod.rs          # dispatch
│   ├── control_flow.rs # if
│   ├── binding.rs      # define, let
│   └── function.rs     # lambda
└── builtins/
    ├── mod.rs          # dispatch (update to include predicates)
    ├── arithmetic.rs   # +, -, *, /, =, <, >, etc.
    ├── list.rs         # car, cdr, cons, list, null?, append, etc.
    └── predicates.rs   # number?, boolean?, string?, symbol?, list?, procedure?, null?
```

#### T3.2.2: Implement advanced list operations
**Deliverables**:
- `length`, `append`, `reverse`, `member`, `assoc`
- List transformation procedures
- Error handling for list operations

#### T3.2.3: Implement I/O procedures (synchronous) ✅
**Deliverables**:
- ✅ `display`, `newline` (implemented in runtime/builtins/io.rs)
- ✅ String output formatting (display handles all value types)
- ☐ `read` (basic) - deferred to later phase

**Implementation Notes**:
- Added Display and Newline builtin variants
- Implemented synchronous I/O with proper flushing
- All value types display correctly (strings without quotes, proper formatting)
- Return unspecified value (Nil) as per Scheme convention
- Comprehensive test coverage (11 unit tests + 4 integration tests)
- Ready for Phase 4 async replacement

**File Structure Note**: This task should add I/O procedures to the builtins directory:
```
runtime/
├── special_forms/
│   ├── mod.rs          # dispatch
│   ├── control_flow.rs # if
│   ├── binding.rs      # define, let
│   └── function.rs     # lambda
└── builtins/
    ├── mod.rs          # dispatch (update to include I/O)
    ├── arithmetic.rs   # +, -, *, /, =, <, >, etc.
    ├── list.rs         # car, cdr, cons, list, null?, append, etc.
    ├── predicates.rs   # number?, boolean?, string?, symbol?, list?, procedure?, null?
    └── io.rs           # display, newline, read
```

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
- Create `src/scheduler/` module
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

**File Structure Note**: This task should expand the builtins directory to include fiber procedures:
```
runtime/
├── special_forms/
│   ├── mod.rs          # dispatch (update to include async)
│   ├── control_flow.rs # if
│   ├── binding.rs      # define, let
│   ├── function.rs     # lambda
│   └── concurrency.rs  # async
└── builtins/
    ├── mod.rs          # dispatch (update to include fibers)
    ├── arithmetic.rs   # +, -, *, /, =, <, >, etc.
    ├── list.rs         # car, cdr, cons, list, null?, append, etc.
    ├── predicates.rs   # number?, boolean?, string?, symbol?, list?, procedure?, null?
    ├── io.rs           # display, newline, read
    └── fibers.rs       # spawn-fiber, yield, current-fiber, fiber-status
```

#### T4.4.1a: Implement async special form
**Prerequisites**: Fiber scheduler complete
**Deliverables**:
- `async` special form for convenient fiber creation
- Support for expression sequences like `begin`
- Environment capture for lexical scoping

**Implementation Note**: `async` should be implemented as a special form (not a built-in procedure) to provide convenient syntax for expression sequences.

**Async Special Form Specification**:
- **Signature**: `(async <expr>...)` - takes zero or more expressions
- **Behavior**: Wraps expressions in implicit thunk and spawns in new fiber
- **Environment**: Captures lexical environment at call site
- **Return value**: Returns a `TaskHandle` immediately (non-blocking)
- **Examples**:
  - `(async (+ 1 2))` - single expression
  - `(async (display "hi") (* 3 4))` - multiple expressions like begin
  - `(async)` - empty body, returns task with nil value

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
- Create `src/parser/macros.rs`
- `Pattern` enum (Literal, Identifier, List, Ellipsis)
- Pattern matching algorithms
- Pattern identifier binding

#### T5.1.2: Implement template system
**Deliverables**:
- `Template` enum (Literal, Identifier, List, Substitution)
- Template expansion with substitution
- Identifier substitution logic

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

**Note**: The `async` special form is implemented in T4.4.1a, not as a macro or built-in procedure.

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
4. **Update local sources** with `./scripts/vendor-deps.sh`

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
**Current Phase**: Phase 3 (Advanced Language Features)
**Overall Progress**: 41% (33/81 tasks completed)  
**Estimated Completion**: 12-16 weeks

### Phase Progress
- **Phase 1**: ✅ 100% (14/14 tasks) - Foundation COMPLETE
- **Phase 2**: ✅ 100% (20/20 tasks) - Basic Interpreter COMPLETE
- **Phase 3**: ☐ 30% (6/20 tasks) - Advanced Features (Section 3.1 COMPLETE, Section 3.2 in progress)
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
- ✅ T2.1.1: Implement `Expr` enum
- ✅ T2.1.2: Implement `Parser` struct
- ✅ T2.1.3: Implement expression parsing
- ✅ T2.1.4: Add parser error handling
- ✅ T2.1.5: Create comprehensive parser tests
- ✅ T2.2.1: Implement `Environment` struct
- ✅ T2.2.2: Implement environment operations
- ✅ T2.2.3: Add environment error handling
- ✅ T2.2.4: Create environment tests
- ✅ T2.3.1: Implement basic `eval` function
- ✅ T2.3.2: Implement arithmetic operations
- ✅ T2.3.3: Implement conditional expressions
- ✅ T2.3.4: Implement basic list operations
- ✅ T2.3.5: Create basic evaluation tests
- ✅ T2.4.1: Implement `define` special form
- ✅ T2.4.2: Implement `let` binding forms
- ✅ T2.4.3: Create identifier binding tests
- ✅ T3.1.1: Implement `Procedure` enum
- ✅ T3.1.2: Implement `lambda` special form
- ✅ T3.1.3: Implement function application
- ✅ T3.1.4: Implement tail call optimization
- ✅ T3.1.5: Create function system tests (45 comprehensive tests)
- ✅ T3.2.3: Implement I/O procedures (synchronous)

### Immediate Next Steps  
1. **T3.2.1**: Implement type checking procedures (🔥 Priority - Continue Phase 3.2)
2. **T3.2.2**: Implement advanced list operations
3. **T3.2.4**: Create built-in procedure tests

### Blocked Tasks
None currently - clear path forward through Phase 1.

---

This task plan provides a structured, educational approach to building the Twine Scheme interpreter while maximizing learning value through AI-assisted development. Each task builds understanding progressively while maintaining rigorous testing and quality standards.
