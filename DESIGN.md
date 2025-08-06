# Twine Scheme Interpreter - Technical Design Document

## Quick Reference

### System Architecture Overview
```
User Interface (REPL/File) → Parser → Evaluator → Fiber Executor → Thread Pool
                          ↓
              Immutable Data Types ← Environment Manager ← Built-ins
```

### Core Components
| Component | Purpose | Key Features |
|-----------|---------|--------------|
| **Lexer** | Tokenization | Comments, numbers, strings, symbols |
| **Parser** | AST Construction | S-expressions, quote syntax, error reporting |
| **Evaluator** | Expression Evaluation | Special forms, procedure application, tail-call optimization |
| **Environment** | Identifier Binding | Lexical scoping, closures, immutable chains |
| **Fiber Executor** | Concurrency Management | Automatic I/O yielding, thread pool execution, fiber completion |
| **Value System** | Data Types | Complete immutability, reference counting |
| **Macro System** | Code Transformation | R7RS-small patterns, hygienic expansion |

### Concurrency Model
- Twine uses a fiber-based concurrency model.
- All async execution and task management is coordinated by the fiber executor.
- **All code runs in fibers** managed by a central executor and scheduler.
- **Automatic I/O yielding**: I/O operations suspend and resume fibers transparently.
- **Thread pool execution**: Fibers run in parallel across CPU cores, with no global interpreter lock.
- **Immutable data sharing**: All data structures are immutable and thread-safe.

---

## System Overview

**Twine** is an educational Scheme interpreter designed for learning AI-assisted development, interpreter implementation, async I/O, parallelism, and advanced Rust concepts.

### Educational Focus Areas
| Area | Learning Goals |
|------|----------------|
| **AI Collaboration** | Iterative development with AI coding agents |
| **Interpreter Design** | Complete language implementation pipeline |
| **Scheme Language** | Functional programming and Lisp concepts |
| **Rust Async** | Async programming patterns with `smol` ecosystem |
| **Concurrency** | Fiber-based scheduling and parallelism |
| **Architecture** | Educational-first design decisions |

### Core Technical Principles
1. **Fiber-based Concurrency**: Lightweight scheduling with thread pool execution
2. **Asynchronous I/O**: Non-blocking I/O that appears synchronous to Scheme
3. **Strict Immutability**: All data structures immutable after creation
4. **Educational Simplicity**: Readable implementations over optimizations
5. **Minimal Feature Set**: Essential R7RS-small subset only

---

## Architecture

### High-Level System Design

```
┌─────────────────────────────────────────────────────────────────┐
│                     User Interface Layer                        │
│  ┌─────────────────┐     ┌─────────────────────────────────────┐ │
│  │  REPL Interface │     │        File Execution              │ │
│  └─────────────────┘     └─────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                      Interpreter Core                           │
│  ┌───────────┐  ┌──────────────┐  ┌─────────────────────────────┐ │
│  │  Lexer    │  │   Parser     │  │     Evaluator               │ │
│  │  Tokens   │→ │  AST Builder │→ │  Expression Evaluation      │ │
│  └───────────┘  └──────────────┘  └─────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Execution Engine                             │
│  ┌─────────────────┐                     ┌─────────────────────┐  │
│  │ Fiber Scheduler │                     │ Environment Manager │  │
│  │ Concurrency Mgmt│                     │ Identifier Binding  │  │
│  └─────────────────┘                     └─────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                    Runtime Foundation                           │
│  ┌──────────────────┐  ┌─────────────────────────────────────────┐ │
│  │ Smol Async       │  │         Thread Pool                     │ │
│  │ Runtime          │  │    Parallel Fiber Execution            │ │
│  └──────────────────┘  └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                      Data Layer                                 │
│  ┌──────────────────┐  ┌─────────────────────────────────────────┐ │
│  │ Immutable Values │  │        Built-in Procedures              │ │
│  │ Reference Counted│  │     I/O, Math, List Operations          │ │
│  └──────────────────┘  └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Module Organization

```
twine/src/
├── main.rs                  # CLI entry point
├── lib.rs                   # Library root and public API
│
├── lexer/                   # Tokenization
│   └── ...                  # Token types and stream generation
│
├── parser/                  # Syntax Analysis
│   └── ...                  # S-expression parsing, AST types, macros
│
├── runtime/                 # Core Evaluation
│   └── ...                  # Evaluation engine, environments, builtins
│
├── types/                   # Data Types
│   └── ...                  # Scheme value types and procedures
│
├── scheduler/               # Concurrency and Execution
│   └── ...                  # Fiber scheduler, async I/O
│
├── repl/                    # Interactive Interface
│   └── ...                  # REPL implementation and line editing
│
└── error/                   # Error Handling
    └── ...                  # Error types and reporting
```

**Module Design Principles**:
- **Single Responsibility**: Each top-level module has one clear purpose
- **Minimal Dependencies**: Clean interfaces between modules
- **Educational Clarity**: Structure reflects domain concepts
- **Progressive Complexity**: Learning-friendly organization

**File Organization Guidelines**:
- **Small Files**: Each file should be relatively small and focused
- **One Type Per File**: Define at most one major type per file (e.g., one Scheme type)
- **Logical Grouping**: Related functionality should be grouped in the same module
- **Clear Naming**: File names should clearly indicate their contents
- **Implementation-Driven**: Let actual implementation needs determine specific file structure

---

## Core Components

### Lexer (`lexer/`)

**Purpose**: Transform source code into token stream

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Delimiters
    LeftParen,
    RightParen,
    Quote,

    // Literals
    Number(f64),
    String(String),
    Symbol(String),
    Boolean(bool),

    // Control
    EOF,
}

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}
```

**Key Features**:
- **Error Location**: Line/column tracking for precise error reporting
- **Comment Handling**: Semicolon to end-of-line comments
- **Number Parsing**: Both integers and floating-point literals
- **String Literals**: Escape sequence support ("hello\nworld")

### Parser (`parser/`)

**Purpose**: Build Abstract Syntax Tree from tokens

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    Atom(Value),              // Single values (numbers, symbols, etc.)
    List(Vec<Expr>),          // S-expressions (function calls, special forms)
    Quote(Box<Expr>),         // Quoted expressions ('expr)
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}
```

**Key Features**:
- **S-expression Parsing**: Proper nested list handling
- **Quote Syntax**: Support for 'expr shorthand
- **Error Recovery**: Detailed syntax error messages
- **AST Generation**: Efficient tree structure for evaluation

### Value System (`types/`)

**Purpose**: Modular immutable data type representation with performance optimizations

**Module Organization**:
- `mod.rs` - Main module with re-exports and overview
- `value.rs` - Core Value enum and methods
- `number.rs` - Number wrapper type
- `string.rs` - String wrapper with Arc sharing
- `symbol.rs` - Symbol wrapper with SmolStr optimization
- `list.rs` - List wrapper type
- `procedure.rs` - Procedure types (future implementation)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // Primitive types using wrapper types
    Number(Number),           // Wraps f64 with Copy semantics
    Boolean(bool),            // Direct boolean values
    String(ArcString),        // Wraps Arc<String> for efficient sharing
    Symbol(Symbol),           // Wraps SmolStr for efficiency

    // Compound types
    List(List),               // Wraps Vec<Value> with sharing
    Procedure(Procedure),     // Functions (builtin/lambda)

    // Concurrency types
    FiberHandle(FiberId),     // Fiber reference for completion waiting

    // Special
    Nil,                      // Empty list/null value
}

#[derive(Debug)]
pub enum Procedure {
    Builtin {
        name: String,
        func: BuiltinFn,
    },
    Lambda {
        params: Vec<String>,
        body: Expr,
        closure: Environment,
    },
}

// Wrapper types for abstraction and optimization
pub struct Number(f64);      // Copy semantics, stack allocation
pub struct ArcString(Arc<String>);  // Reference counting for thread-safe sharing
pub struct Symbol(SmolStr);  // Stack allocation ≤23 bytes, heap otherwise
pub struct List(Vec<Value>); // Owned vector (sharing planned)
```

**Key Features**:
- **Modular Design**: Each type in separate module for clarity
- **Performance Optimized**: SmolStr for symbols, Arc for strings
- **Complete Immutability**: No mutation operations supported
- **Thread Safety**: All types implement Send + Sync
- **Memory Efficiency**: SmolStr stack allocation, Arc sharing

**Performance Optimizations**:
- **Symbols**: Use `SmolStr` - stack allocation for identifiers ≤23 bytes
- **Zero-copy Symbol creation**: Direct SmolStr → Symbol conversion via `from_smol_str()`
- **Strings**: Use `Arc<String>` for efficient sharing across threads
- **Numbers**: Use primitive `f64` with `Copy` semantics
- **Lists**: Use `Vec<Value>` with planned structural sharing

### Environment Management (`runtime/environment.rs`)

**Purpose**: Identifier binding and lexical scoping

```rust
#[derive(Debug, Clone)]
pub struct Environment {
    bindings: Arc<HashMap<String, Value>>,
    parent: Option<Arc<Environment>>,
}

impl Environment {
    pub fn new() -> Self
    pub fn with_parent(parent: Arc<Environment>) -> Self
    pub fn define(&self, name: String, value: Value) -> Self
    pub fn lookup(&self, name: &str) -> Option<Value>
    pub fn extend(&self, params: &[String], args: &[Value]) -> Self
}
```

**Key Features**:
- **Lexical Scoping**: Proper scope chain traversal
- **Closure Support**: Environment capture for lambdas
- **Immutable Chains**: New environments don't modify parents
- **Thread-Safe Sharing**: `Arc` enables concurrent access

---

## Concurrency Model

### Fiber-Based Concurrency System

```
┌─────────────────────────────────────────────────────────────────┐
│                   Fiber Scheduler System                        │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ spawn-fiber → creates independent fibers                   │ │
│  │ async → convenient syntax for fiber spawning               │ │
│  │ fiber-wait → synchronizes on fiber completion              │ │
│  │ I/O Yielding → automatic suspension/resumption             │ │
│  │ Ready Queue: [F1, F3] | Suspended: [F2→IO] | Running: F4   │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     Thread Pool Execution                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │  Thread 1   │ │  Thread 2   │ │  Thread 3   │ │  Thread 4   │ │
│  │  Running    │ │  Running    │ │  Running    │ │   (idle)    │ │
│  │  Fiber A    │ │  Fiber C    │ │  Fiber E    │ │             │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Fiber Execution Architecture

```rust
pub struct Fiber {
    id: FiberId,
    state: FiberState,
    continuation: Pin<Box<dyn Future<Output = Result<Value, Error>> + Send>>,
    parent: Option<FiberId>,
    children: HashSet<FiberId>,
}

pub enum FiberState {
    Ready,                          // In ready queue
    Running,                        // Executing on thread
    Suspended(SuspendReason),       // Waiting for something
    Completed(Result<Value, Error>), // Finished execution
}

pub enum SuspendReason {
    IoOperation(IoFuture),          // Waiting for I/O
    WaitingForFiber(FiberId),       // Waiting for fiber completion
    Yielded,                        // Explicit yield
}

pub struct FiberScheduler {
    ready_queue: VecDeque<FiberId>,
    fibers: HashMap<FiberId, Fiber>,
    runtime: smol::Executor<'static>,
    thread_pool: Vec<std::thread::JoinHandle<()>>,
    current_fiber: Option<FiberId>,
}
```



### Execution Flow Examples

#### Basic Fiber Spawning
```scheme
> (define worker (spawn-fiber (lambda () (+ 100 200))))
worker
> (fiber-wait worker)
300
```

#### Fiber Synchronization, Async Special Form, and I/O Yielding

```scheme
;; Using the async special form to spawn fibers
> (define f1 (async (+ 1 2)))
f1
> (fiber-wait f1)
3

> (define f2 (async (display "Starting...\n") (display "Finished!\n") 42))
Starting...
Finished!
f2
> (fiber-wait f2)
42

;; Direct fiber spawning also supported
> (define slow-fiber (spawn-fiber (lambda ()
    (display "Starting...\n")
    (display "Finished!\n")
    42)))
Starting...
slow-fiber
> (fiber-wait slow-fiber)
Finished!
42
```

**Key Properties**:
- **Transparent I/O**: All I/O operations automatically yield without syntax
- **True Parallelism**: Multiple fibers execute simultaneously across CPU cores
- **No GIL**: Immutable data enables lock-free parallel execution
- **Parent/Child Fibers**: Fibers can spawn child fibers for structured concurrency
- **Async Special Form**: `(async <expr>...)` spawns a new fiber for expressions, returning a fiber handle immediately
- **Unified Model**: Single fiber abstraction handles all concurrency needs

---

## Data Types and Memory Management

### Immutable Value Design

| Type | Implementation | Memory Strategy | Optimization |
|------|----------------|-----------------|--------------|
| **Numbers** | `Number(f64)` | Stack allocation | Copy semantics |
| **Booleans** | `bool` | Stack allocation | Copy semantics |
| **Strings** | `ArcString(Arc<String>)` | Heap + reference counting | Thread-safe sharing |
| **Symbols** | `Symbol(SmolStr)` | Stack ≤23 bytes, heap otherwise | Automatic optimization |
| **Lists** | `List(Vec<Value>)` | Heap allocation | Planned structural sharing |
| **Procedures** | `Procedure` | Future implementation | Planned Arc sharing |

### Memory Management Strategy

```rust
// Wrapper types with specific optimizations
pub struct Number(f64);                    // Copy, stack allocation
pub struct ArcString(Arc<String>);         // Reference counting for thread-safe sharing
pub struct Symbol(SmolStr);                // Smart string optimization
pub struct List(Vec<Value>);               // Owned vector

// SmolStr optimization details:
// - Symbols ≤23 bytes: stack allocated, O(1) clone
// - Symbols >23 bytes: heap allocated, reference counted
// - Automatic selection based on content length
// - Perfect for Scheme identifiers (most are short)

// ArcString sharing with Arc:
// - Thread-safe reference counting for std::string::String
// - Efficient sharing across Value instances
// - Automatic memory cleanup when no longer referenced
// - Clean naming avoiding std::string::String conflicts
```

**SmolStr Performance Characteristics**:
- **Short symbols** (≤23 bytes): Stack allocated, zero-cost cloning
- **Long symbols** (>23 bytes): Heap allocated, reference counted
- **Common Scheme identifiers**: `+`, `-`, `car`, `cdr`, `list`, `define` - all stack allocated
- **Thread safety**: Both stack and heap variants are Send + Sync

### Symbol Optimization Example

```rust
// Short symbols - stack allocated (≤23 bytes)
let short_sym = Symbol::new("+");           // Stack: 1 byte + metadata
let medium_sym = Symbol::new("define");     // Stack: 6 bytes + metadata
let car = Symbol::new("car");               // Stack: 3 bytes + metadata

// Cloning is zero-cost for stack-allocated symbols
let cloned = short_sym.clone();             // O(1) operation

// Long symbols - heap allocated (>23 bytes)
let long_sym = Symbol::new("very-long-identifier-name"); // Heap + refcount

// All symbols are thread-safe and immutable

// Zero-copy conversion for maximum efficiency
let existing_smol = SmolStr::new("existing-identifier");
let efficient_symbol = Symbol::from_smol_str(existing_smol); // O(1) operation
```

**Symbol Storage Benefits**:
- **Memory Efficiency**: Common identifiers use no heap allocation
- **Performance**: Zero-cost cloning for typical Scheme symbols
- **Zero-copy Construction**: `Symbol::from_smol_str()` for direct SmolStr conversion
- **Thread Safety**: Both stack and heap variants work across threads
- **Automatic Selection**: SmolStr picks optimal storage automatically
- **Scheme Optimized**: Perfect for typical identifier patterns

**Current ArcString Sharing**:
```rust
let original = ArcString::new("shared content");
let cloned = original.clone();
// Both reference the same Arc<String> - efficient sharing
```

**Planned List Structural Sharing** (Future):
```
Original List:    [1, 2, 3, 4]
                  └─── shared ───┘

Cons Operation:   [0, 1, 2, 3, 4]
                  │   └─ shared ──┘
                  └─ new ─┘
```

---

## Execution Engine

### Evaluation Model

```rust
pub fn eval(
    expr: Expr,
    env: Environment,
    scheduler: &mut FiberScheduler,
    fiber_id: FiberId
) -> Result<Value, Error> {
    match expr {
        Expr::Atom(value) => eval_atom(value, env),
        Expr::List(exprs) => eval_list(exprs, env, scheduler, fiber_id),
        Expr::Quote(expr) => Ok(expr_to_value(*expr)),
    }
}

fn eval_list(
    exprs: Vec<Expr>,
    env: Environment,
    scheduler: &mut FiberScheduler,
    fiber_id: FiberId
) -> Result<Value, Error> {
    if exprs.is_empty() {
        return Ok(Value::Nil);
    }

    // Check for special forms first
    if let Expr::Atom(Value::Symbol(sym)) = &exprs[0] {
        match sym.as_ref() {
            "if" => eval_if(&exprs[1..], env, scheduler, fiber_id),
            "define" => eval_define(&exprs[1..], env, scheduler, fiber_id),
            "lambda" => eval_lambda(&exprs[1..], env),
            "quote" => eval_quote(&exprs[1..]),

            _ => eval_application(exprs, env, scheduler, fiber_id),
        }
    } else {
        eval_application(exprs, env, scheduler, fiber_id)
    }
}
```

### Special Forms Implementation

| Special Form | Purpose | Example |
|--------------|---------|---------|
| **if** | Conditional execution | `(if (> x 0) 'positive 'non-positive)` |
| **define** | Identifier binding | `(define pi 3.14159)` |
| **lambda** | Procedure creation | `(lambda (x) (* x x))` |
| **quote** | Literal data | `(quote (a b c))` or `'(a b c)` |

### Tail Call Optimization

```rust
fn eval_application(
    exprs: Vec<Expr>,
    env: Environment,
    scheduler: &mut FiberScheduler,
    fiber_id: FiberId
) -> Result<Value, Error> {
    let func = eval(exprs[0].clone(), env.clone(), scheduler, fiber_id)?;
    let args = eval_args(&exprs[1..], env.clone(), scheduler, fiber_id)?;

    match func {
        Value::Procedure(proc) => match proc.as_ref() {
            Procedure::Lambda { params, body, closure } => {
                let new_env = closure.extend(params, &args);
                // Tail call optimization - direct recursion
                eval(body.clone(), new_env, scheduler, fiber_id)
            }
            Procedure::Builtin { func, .. } => {
                func(&args, scheduler, fiber_id)
            }
        },
        _ => Err(Error::TypeError("Not a procedure".to_string())),
    }
}
```

---

## Asynchronous I/O Integration

### I/O Architecture Principles

- **Transparent Yielding**: I/O operations automatically yield current fiber
- **Synchronous Appearance**: No async/await syntax in Scheme code
- **Non-blocking Runtime**: Other fibers continue during I/O
- **Automatic Resumption**: Fibers resume when I/O completes

### I/O Implementation Pattern

```rust
// Internal async implementation
async fn display_async(value: &Value) -> Result<(), Error> {
    let output = format_scheme_value(value);
    let mut stdout = smol::io::stdout();
    stdout.write_all(output.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}

// Fiber-yielding wrapper (appears synchronous to Scheme)
pub fn display(
    args: &[Value],
    scheduler: &mut FiberScheduler,
    fiber_id: FiberId
) -> Result<Value, Error> {
    if args.len() != 1 {
        return Err(Error::ArityError("display expects 1 argument".into()));
    }

    let value = args[0].clone();
    let io_future = async move {
        display_async(&value).await.unwrap();
    };

    // Yield fiber for I/O operation
    scheduler.yield_for_io(fiber_id, Box::pin(io_future));

    // Execution resumes here after I/O completes
    Ok(Value::Nil)
}
```

### Built-in I/O Procedures

| Procedure | Purpose | Behavior |
|-----------|---------|----------|
| **display** | Output value | `(display "Hello")` - yields fiber during output |
| **newline** | Output newline | `(newline)` - yields fiber during output |
| **read-line** | Input line | `(read-line)` - yields fiber during input |

---

## Macro System

### R7RS-small Macro Support

```rust
#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Value),              // Exact match: 42, #t, "hello"
    Identifier(String),          // Bind to identifier: x, condition
    List(Vec<Pattern>),          // Match list structure: (if ...)
    Ellipsis(Box<Pattern>),      // Variable length: body ...
}

#[derive(Debug, Clone)]
pub enum Template {
    Literal(Value),              // Insert literal value
    Identifier(String),          // Substitute identifier
    List(Vec<Template>),         // Generate list structure
    Substitution(String),        // Pattern substitution
}

pub struct MacroRule {
    pattern: Pattern,
    template: Template,
}

pub struct Macro {
    name: String,
    rules: Vec<MacroRule>,
}
```

### Standard Macro Examples

#### when Macro
```scheme
(define-syntax when
  (syntax-rules ()
    ((when condition body ...)
     (if condition (begin body ...)))))

;; Usage
(when (> x 0)
  (display "Positive")
  (newline))
```

#### async Special Form
**Note**: `async` is implemented as a special form that takes a sequence of expressions and spawns them in a new fiber.

```scheme
;; async is a special form that takes a sequence of expressions (like begin)
;; Usage examples:
(define fiber1 (async (+ 1 2 3)))                      ; Single expression
(define fiber2 (async 
  (display "Working...")
  (* 6 7)))                                            ; Multiple expressions

;; Identifier capture example
(let ((x 10))
  (async (+ x 1)))                                     ; Captures x from environment
```

**Evaluation Semantics**: The `async` special form:
1. **Takes zero or more expressions** - expressions are not pre-evaluated
2. **Creates implicit thunk** - wraps expressions in a closure automatically
3. **Captures environment** - lexical environment is captured at async call site
4. **Returns immediately** - returns a `FiberHandle` without blocking
5. **Spawns fiber** - expressions will be evaluated in a separate fiber

**Special Form Signature**: `(async <expr>...)`
- **Zero expressions**: `(async)` - returns completed fiber with nil value
- **Single expression**: `(async (+ 1 2))` - evaluates expression in fiber
- **Multiple expressions**: `(async (display "hi") (* 3 4))` - sequential evaluation like begin
- **No restrictions**: Any valid Scheme expressions allowed

**Design Rationale**: `async` is implemented as a special form rather than a built-in procedure:
- **Convenient Syntax**: No need to wrap expressions in lambda manually
- **Natural Feel**: Similar to `begin` and `let` - takes expression sequences
- **Flexible**: Supports zero, one, or many expressions naturally
- **Consistent**: Aligns with other special forms that control evaluation

**Key Features**:
- **Pattern Matching**: R7RS-small syntax-rules patterns
- **Ellipsis Support**: Variable-length pattern matching (`...`)
- **Hygienic Expansion**: Prevents identifier capture
- **Compile-time**: Macros expanded before evaluation

---

## Error Handling

### Error Type Hierarchy

```rust
#[derive(Debug, Clone)]
pub enum Error {
    // Parse-time errors
    SyntaxError { message: String, line: usize, column: usize },
    ParseError(String),

    // Runtime errors
    TypeError(String),
    ArityError(String),
    UnboundIdentifier(String),
    DivisionByZero,

    // System errors
    IoError(String),
    FiberError(String),
    MacroError(String),
    SystemError(String),
}
```

### Error Propagation in Async Context

```rust
pub type Result<T> = std::result::Result<T, Error>;

// Async-safe error handling
pub async fn execute_with_error_handling(
    expr: Expr,
    env: Environment,
    scheduler: &mut FiberScheduler,
    fiber_id: FiberId,
) -> Result<Value> {
    match eval(expr, env, scheduler, fiber_id) {
        Ok(value) => Ok(value),
        Err(error) => {
            // Log error and propagate
            eprintln!("Error in fiber {}: {:?}", fiber_id, error);
            Err(error)
        }
    }
}
```

### Error Display Examples

```
Syntax error at line 5, column 12: Unexpected token ')'
Type error: Expected number, got string in arithmetic operation
Arity error: + expects at least 1 argument, got 0
Unbound identifier: undefined-procedure
I/O error: Failed to write to stdout
Fiber error: Fiber deadlock detected
Task error: Parent task cancelled, terminating children
```

---

## Implementation Considerations

### Core Dependencies

```toml
[dependencies]
# Async runtime ecosystem (all from smol)
smol = "2.0"              # Main async runtime
futures-lite = "2.0"      # Future utilities
async-task = "4.7"        # Task spawning
async-channel = "2.1"     # Message passing
polling = "3.3"           # I/O polling

# String optimization
smol_str = "0.3"          # Optimized string storage for symbols

# Error handling
thiserror = "2.0"         # Error derive macros

# Optional development dependencies
[dev-dependencies]
criterion = "0.5"         # Benchmarking
tokio-test = "0.4"        # Async testing
```

### Performance Characteristics

| Operation | Expected Performance |
|-----------|---------------------|
| **Simple Arithmetic** | <1ms completion |
| **Function Calls** | <100μs overhead |
| **Fiber Spawning** | <10μs creation |
| **I/O Operations** | Non-blocking, <1ms yield |
| **Symbol Creation** | O(1) for ≤23 bytes (SmolStr), O(1) clone, zero-copy from SmolStr |
| **String Sharing** | O(1) clone via Arc reference counting |
| **Memory Usage** | O(n) for data, minimal runtime overhead |
| **Concurrency** | Scales with CPU cores |

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let result = eval_string("(+ 1 2 3)");
        assert_eq!(result, Ok(Value::Number(6.0)));
    }

    #[smol_potat::test]
    async fn test_fiber_execution() {
        let mut scheduler = FiberScheduler::new(4);
        let fiber_id = scheduler.spawn_fiber(
            Value::Procedure(Arc::new(Procedure::Lambda {
                params: vec![],
                body: Expr::Atom(Value::Number(42.0)),
                closure: Environment::new(),
            })),
            None
        );

        let result = scheduler.run_until_complete(fiber_id).await;
        assert_eq!(result, Ok(Value::Number(42.0)));
    }
}
```

---

## Security and Resource Management

### Memory Safety
- **Rust Ownership**: Automatic memory safety guarantees
- **No Manual Memory Management**: Reference counting handles cleanup
- **Thread Safety**: Immutable data prevents data races
- **Bounds Checking**: Array access bounds checked automatically

### Resource Limits

```rust
pub struct ResourceLimits {
    max_stack_depth: usize,      // Prevent stack overflow
    max_fiber_count: usize,      // Limit concurrent fibers
    max_memory_usage: usize,     // Memory usage cap
    execution_timeout: Duration, // Prevent infinite loops
}

impl ResourceLimits {
    pub fn check_stack_depth(&self, current: usize) -> Result<(), Error> {
        if current > self.max_stack_depth {
            Err(Error::SystemError("Stack overflow".into()))
        } else {
            Ok(())
        }
    }
}
```

---

## Educational Architecture

### Learning Progression Design

#### Phase 1: Foundation (Weeks 1-2)
- **Core Data Types**: Numbers, booleans, strings, symbols
- **Basic Parsing**: Tokenization and S-expression parsing
- **Simple Evaluation**: Arithmetic and identifier lookup
- **Learning Focus**: Understanding interpreter basics

#### Phase 2: Language Features (Weeks 3-4)
- **Functions**: Lambda creation and application
- **Control Flow**: Conditional expressions (if)
- **Built-ins**: Core procedure library
- **Learning Focus**: Scheme language semantics

#### Phase 3: Concurrency (Weeks 5-6)
- **Fiber Scheduler**: Low-level fiber management
- **Task System**: High-level async abstraction
- **I/O Integration**: Asynchronous I/O with yielding
- **Learning Focus**: Concurrent programming patterns

#### Phase 4: Advanced Features (Weeks 7-8)
- **Macro System**: R7RS-small macro support
- **Optimization**: Tail-call optimization
- **Polish**: Error handling and performance
- **Learning Focus**: Advanced language features

### Knowledge Transfer Goals

- **Practical Skills**: Building interpreters from scratch
- **Theoretical Understanding**: Language design principles
- **Tool Mastery**: Effective AI collaboration patterns
- **System Design**: Balancing simplicity with functionality

---

This design document serves as the technical blueprint for Twine's implementation. All development decisions should align with these specifications while maintaining the educational focus and technical rigor outlined here.
