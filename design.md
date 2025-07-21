# Twine Scheme Interpreter - Technical Design Document

## Quick Reference

### System Architecture Overview
```
User Interface (REPL/File) → Parser → Evaluator → Fiber Scheduler → Thread Pool
                          ↓
              Immutable Data Types ← Environment Manager ← Built-ins
```

### Core Components
| Component | Purpose | Key Features |
|-----------|---------|--------------|
| **Lexer** | Tokenization | Comments, numbers, strings, symbols |
| **Parser** | AST Construction | S-expressions, quote syntax, error reporting |
| **Evaluator** | Expression Evaluation | Special forms, function application, tail-call optimization |
| **Environment** | Variable Binding | Lexical scoping, closures, immutable chains |
| **Fiber Scheduler** | Concurrency Management | Automatic I/O yielding, thread pool execution |
| **Task System** | High-level Async | Hierarchical parent-child relationships |
| **Value System** | Data Types | Complete immutability, reference counting |
| **Macro System** | Code Transformation | R7RS-small patterns, hygienic expansion |

### Concurrency Model
- **All code runs in fibers** managed by central scheduler
- **Two-layer system**: Low-level fibers + High-level tasks
- **Automatic I/O yielding** - appears synchronous to Scheme
- **Thread pool execution** - true parallelism without GIL
- **Immutable data sharing** - thread-safe by design

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
│  ┌─────────────────┐  ┌──────────────┐  ┌─────────────────────┐  │
│  │ Fiber Scheduler │  │ Task System  │  │ Environment Manager │  │
│  │ Low-level Mgmt  │  │ High-level   │  │ Variable Binding    │  │
│  └─────────────────┘  └──────────────┘  └─────────────────────┘  │
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
│   ├── mod.rs               # Token stream generation
│   └── token.rs             # Token type definitions
│
├── parser/                  # Syntax Analysis
│   ├── mod.rs               # S-expression parsing
│   └── ast.rs               # Abstract syntax tree types
│
├── interpreter/             # Core Evaluation
│   ├── mod.rs               # Main evaluation engine
│   ├── environment.rs       # Variable scoping and binding
│   ├── builtins.rs          # Built-in procedure implementations
│   └── macros.rs            # Macro system (define-syntax, syntax-rules)
│
├── types/                   # Data Types
│   ├── mod.rs               # Core value types and conversions
│   ├── value.rs             # Scheme value enumeration
│   └── procedures.rs        # Function and procedure types
│
├── runtime/                 # Concurrency and Execution
│   ├── mod.rs               # Runtime coordination
│   ├── scheduler.rs         # Fiber scheduler implementation
│   ├── task.rs              # High-level task abstraction
│   └── io.rs                # Async I/O operations
│
├── repl/                    # Interactive Interface
│   ├── mod.rs               # REPL implementation
│   └── readline.rs          # Line editing and history
│
└── error/                   # Error Handling
    ├── mod.rs               # Error type hierarchy
    └── reporting.rs         # Error formatting and display
```

**Design Principles**:
- **Single Responsibility**: Each module has one clear purpose
- **Minimal Dependencies**: Clean interfaces between modules
- **Educational Clarity**: Structure reflects domain concepts
- **Progressive Complexity**: Learning-friendly organization

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

**Purpose**: Immutable data type representation

```rust
#[derive(Debug, Clone)]
pub enum Value {
    // Primitive types
    Number(f64),
    Boolean(bool),
    String(Arc<str>),         // Immutable shared strings
    Symbol(Arc<str>),         // Interned symbols
    
    // Compound types
    List(Arc<[Value]>),       // Immutable arrays
    Procedure(Arc<Procedure>), // Functions (builtin/lambda)
    
    // Concurrency types
    TaskHandle(TaskId),       // High-level task reference
    FiberHandle(FiberId),     // Low-level fiber reference
    
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
```

**Key Features**:
- **Complete Immutability**: No mutation operations supported
- **Reference Counting**: `Arc<T>` for efficient sharing
- **Thread Safety**: Immutable data shared safely across threads
- **Memory Efficiency**: Structural sharing for large values

### Environment Management (`interpreter/environment.rs`)

**Purpose**: Variable binding and lexical scoping

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

### Two-Layer Concurrency System

```
┌─────────────────────────────────────────────────────────────────┐
│                    High-Level: Task System                      │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ async macro → spawn-fiber → creates hierarchical tasks     │ │
│  │ task-wait → synchronization with parent-child cleanup      │ │
│  │ Task Tree: Main → [TaskA, TaskB → [TaskB1, TaskB2]]        │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                   Low-Level: Fiber Scheduler                    │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │ spawn-fiber → independent execution units                   │ │
│  │ fiber-wait → manual synchronization                        │ │
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

### Fiber Scheduler Architecture

```rust
pub struct Fiber {
    id: FiberId,
    state: FiberState,
    continuation: Pin<Box<dyn Future<Output = Result<Value, Error>> + Send>>,
    associated_task: Option<TaskId>, // Only for task-spawned fibers
}

pub enum FiberState {
    Ready,                          // In ready queue
    Running,                        // Executing on thread
    Suspended(SuspendReason),       // Waiting for something
    Completed(Result<Value, Error>), // Finished execution
}

pub enum SuspendReason {
    IoOperation(IoFuture),          // Waiting for I/O
    WaitingForTask(TaskId),         // Waiting for task completion
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

### Task System Architecture

```rust
pub struct Task {
    id: TaskId,
    fiber_id: FiberId,               // Associated fiber
    parent: Option<TaskId>,          // Hierarchical parent
    children: HashSet<TaskId>,       // Child tasks
    state: TaskState,
    result: Option<Result<Value, Error>>,
}

pub struct TaskHandle {
    id: TaskId,
    scheduler_ref: Weak<RefCell<TaskScheduler>>,
}

impl TaskHandle {
    pub fn wait(&self) -> Result<Value, Error>  // Suspend until completion
    pub fn is_finished(&self) -> bool           // Check completion status
    pub fn cancel(&self) -> Result<(), Error>   // Cancel with children
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

#### Hierarchical Task Execution
```scheme
> (define task1 (async (+ 10 20)))
task1
> (define task2 (async (lambda () 
    (let ((subtask (async (* 3 4))))
      (+ (task-wait subtask) 100)))))
task2
> (task-wait task1)
30
> (task-wait task2)
112
```

#### Automatic I/O Yielding
```scheme
> (define slow-task (async (lambda ()
    (display "Starting...\n")    ; Automatically yields fiber
    (display "Finished!\n")      ; Resumes after I/O
    42)))
Starting...
slow-task
> (define quick-task (async (+ 1 2 3)))
quick-task
> (task-wait quick-task)
6
> (task-wait slow-task)
Finished!
42
```

**Key Properties**:
- **Transparent I/O**: All I/O operations automatically yield without syntax
- **True Parallelism**: Multiple fibers execute simultaneously across CPU cores
- **No GIL**: Immutable data enables lock-free parallel execution
- **Hierarchical Tasks**: Parent-child relationships for resource management
- **Independent Fibers**: Low-level control for advanced use cases

---

## Data Types and Memory Management

### Immutable Value Design

| Type | Implementation | Memory Strategy |
|------|----------------|-----------------|
| **Numbers** | `f64` (Copy) | Stack allocation |
| **Booleans** | `bool` (Copy) | Stack allocation |
| **Strings** | `Arc<str>` | Heap + reference counting |
| **Symbols** | `Arc<str>` | Heap + interning |
| **Lists** | `Arc<[Value]>` | Heap + structural sharing |
| **Procedures** | `Arc<Procedure>` | Heap + sharing |

### Memory Management Strategy

```rust
// Efficient immutable string sharing
pub type SchemeString = Arc<str>;
pub type SchemeSymbol = Arc<str>;  // Could use string interning

// Immutable array for lists - enables structural sharing
pub type SchemeList = Arc<[Value]>;

// Reference-counted procedures
pub type SchemeProcedure = Arc<Procedure>;
```

### Structural Sharing Example

```
Original List:    [1, 2, 3, 4]
                  └─── Arc ────┘

Cons Operation:   [0, 1, 2, 3, 4]
                  │   └─ Arc (shared) ──┘
                  └─ New Arc ─┘

Result: Two lists sharing memory for [1, 2, 3, 4] portion
```

**Benefits**:
- **Memory Efficiency**: Shared data reduces allocation
- **Thread Safety**: Immutable data can be shared safely
- **Performance**: No copying required for most operations
- **Automatic Cleanup**: Rust's ownership handles deallocation

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
| **define** | Variable binding | `(define pi 3.14159)` |
| **lambda** | Function creation | `(lambda (x) (* x x))` |
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
    Variable(String),            // Bind to variable: x, condition
    List(Vec<Pattern>),          // Match list structure: (if ...)
    Ellipsis(Box<Pattern>),      // Variable length: body ...
}

#[derive(Debug, Clone)]
pub enum Template {
    Literal(Value),              // Insert literal value
    Variable(String),            // Substitute variable
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

#### async Macro
```scheme
(define-syntax async
  (syntax-rules (lambda)
    ((async (lambda () body ...))
     (spawn-fiber (lambda () body ...)))
    ((async expr)
     (spawn-fiber (lambda () expr)))))

;; Usage
(define task1 (async (+ 1 2 3)))           ; Simple expression
(define task2 (async (lambda () 
  (display "Working...")
  (* 6 7))))                               ; Explicit thunk
```

**Key Features**:
- **Pattern Matching**: R7RS-small syntax-rules patterns
- **Ellipsis Support**: Variable-length pattern matching (`...`)
- **Hygienic Expansion**: Prevents variable capture
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
    UnboundVariable(String),
    DivisionByZero,

    // System errors
    IoError(String),
    FiberError(String),
    TaskError(String),
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
Unbound variable: undefined-function
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

# Error handling
thiserror = "1.0"         # Error derive macros

# Optional development dependencies
[dev-dependencies]
criterion = "0.5"         # Benchmarking
tokio-test = "0.4"        # Async testing
```

### Local Dependency Management

```bash
# After any Cargo.toml changes, update local sources
./scripts/update-deps.sh

# Verify vendored sources
ls deps/vendor/smol/
ls deps/docs/smol/

# Check dependency compatibility
cargo tree | grep smol
```

### Performance Characteristics

| Operation | Expected Performance |
|-----------|---------------------|
| **Simple Arithmetic** | <1ms completion |
| **Function Calls** | <100μs overhead |
| **Fiber Spawning** | <10μs creation |
| **I/O Operations** | Non-blocking, <1ms yield |
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
- **Simple Evaluation**: Arithmetic and variable lookup
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