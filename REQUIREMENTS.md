# Requirements for Twine Scheme Interpreter

## Quick Reference

### Core Principles (All HIGH Priority)
1. **Fiber-based Concurrency**: Lightweight scheduler with multi-threaded execution, no GIL
2. **Asynchronous I/O**: All I/O async with fiber yielding, appears synchronous to Scheme
3. **Strict Immutability**: All data structures immutable after creation
4. **Simplicity & Minimalism**: Essential R7RS-small subset only
5. **Educational Value**: Learning-first design and implementation

### Key Constraints
- ❌ **NO mutable operations** - Complete immutability enforced
- ✅ **smol ecosystem only** - All async dependencies from smol
- ✅ **R7RS-small subset** - Essential features only
- ✅ **Educational focus** - Simple, readable implementations

### Success Criteria
- All 17 functional requirements implemented
- All 12 acceptance criteria validated
- >90% test coverage
- Clear learning progression documented

---

## Project Overview

**Twine** is an educational Scheme interpreter designed for learning AI-assisted development, interpreter implementation, async I/O, parallelism, and advanced Rust concepts.

### Educational Objectives
| Learning Area | Goal |
|---------------|------|
| **AI Agent Development** | Practical collaboration with AI coding agents |
| **Interpreter Implementation** | Complete language pipeline understanding |
| **Scheme Language** | Functional programming and Lisp concepts |
| **Rust Async Ecosystem** | Hands-on async programming with `smol` |
| **Concurrency Models** | Fiber-based scheduling implementation |
| **Software Architecture** | Balance technical requirements with learning value |

### Technical Foundation
Implements functional subset of R7RS-small with:
- Immutable data structures only (core constraint)
- Fiber scheduler for concurrent execution
- Asynchronous I/O with automatic yielding
- Unified fiber system with completion values

---

## Core Design Principles

### 1. Fiber Scheduler and Async Task System
- **Execution Model**: All code runs within fibers managed by central scheduler
- **I/O Integration**: Automatic fiber yielding during I/O operations
- **Unified System**: `spawn-fiber` and `async` both create fibers that yield values
- **Parallelism**: Multi-threaded execution across CPU cores without GIL
- **Synchronization**: `fiber-wait` for completion and value retrieval

### 2. Asynchronous I/O
- **Transparent to Scheme**: I/O appears synchronous, no async/await syntax
- **Fiber Yielding**: Automatic suspension during I/O operations
- **Non-blocking**: Other fibers continue executing during I/O
- **Integration**: REPL and file execution work seamlessly with fibers

### 3. Strict Immutability
- **Core Constraint**: All data structures immutable after creation
- **No Mutation**: Zero mutable operations supported
- **Side Effects**: I/O operations allowed but don't mutate data
- **Enforcement**: System rejects any mutation attempts

### 4. Simplicity and Minimalism
- **Essential Features**: Only core R7RS-small constructs
- **Readable Code**: Straightforward implementations over optimizations
- **Learning Focus**: Code serves as educational resource
- **Minimal Dependencies**: Limited external dependencies, smol ecosystem only

### 5. Educational Value
- **Learning-First**: All decisions prioritize educational insight
- **Progressive Complexity**: Clear advancement through concepts
- **Documentation**: Extensive explanation of design decisions
- **Experimentation**: Architecture supports exploring alternatives

---

## User Stories

### Educational Stories
| Story | User | Goal | Benefit |
|-------|------|------|---------|
| **E1** | Developer learning AI programming | Build complex project with AI agent | Understand collaboration patterns |
| **E2** | Computer science student | Implement complete interpreter | Understand language implementation |
| **E3** | Rust developer | Build async system with smol | Learn fiber-based concurrency |
| **E4** | Functional programming explorer | Use Scheme features | Understand immutability and closures |

### Functional Stories
| Story | User | Goal | Benefit |
|-------|------|------|---------|
| **F1** | Functional programming learner | Evaluate basic expressions | Perform arithmetic/logical operations |
| **F2** | Scheme programmer | Define and call functions | Create reusable code modules |
| **F3** | Developer | Use interactive REPL | Experiment and debug interactively |
| **F4** | Programmer | Execute files | Run complete programs and scripts |
| **F5** | Code debugger | Receive clear errors | Quickly identify and fix issues |

---

## Functional Requirements

### Core Language (FR-1 to FR-8)

| ID | Requirement | Key Features |
|----|-------------|--------------|
| **FR-1** | **Lexical Analysis** | Tokenize source into atoms, numbers, strings, delimiters; handle comments |
| **FR-2** | **Syntactic Analysis** | Build AST from S-expressions; validate parentheses; report syntax errors |
| **FR-3** | **Immutable Data Types** | Numbers, booleans, strings, symbols, lists, procedures - NO mutation |
| **FR-4** | **Arithmetic Operations** | +, -, *, /, =, <, >, <=, >= with variadic support |
| **FR-5** | **List Operations** | car, cdr, cons, list, null?, pair? |
| **FR-6** | **Conditional Expressions** | `if` with condition, then-clause, optional else-clause |
| **FR-7** | **Identifier Binding** | `define` for globals, `lambda` for procedures - NO reassignment |
| **FR-8** | **Procedure Application** | Left-to-right evaluation, proper binding, tail-call optimization |

### Interactive Features (FR-9 to FR-12)

| ID | Requirement | Key Features |
|----|-------------|--------------|
| **FR-9** | **REPL Functionality** | Interactive prompt, read S-expressions, evaluate and print |
| **FR-10** | **File Execution** | Sequential evaluation of file expressions |
| **FR-11** | **Built-in Procedures** | Type predicates, I/O (fiber-yielding), list ops, higher-order functions |
| **FR-12** | **Error Handling** | Descriptive messages, location info, error type distinction |

### Advanced Features (FR-13 to FR-17)

| ID | Requirement | Key Features |
|----|-------------|--------------|
| **FR-13** | **Lexical Scoping** | Environment chains, closures, strict immutability enforcement |
| **FR-14** | **Fiber Scheduler Integration** | All code in fibers, automatic I/O yielding, transparent to Scheme |
| **FR-15** | **Fiber Completion System** | Fibers complete with values, `async` special form for convenient syntax |
| **FR-16** | **Macro System** | R7RS-small `define-syntax` and `syntax-rules`, hygienic expansion |
| **FR-17** | **Minimal Language Subset** | Essential features only, simple implementation priority |

---

## Non-Functional Requirements

### Performance (NFR-1)
- Simple arithmetic: <1ms completion
- Recursive functions: >1000 call depth support
- Responsive I/O: Non-blocking through fiber yielding
- Multi-core utilization: Parallel fiber execution
- Hardware scaling: Performance scales with available threads

### System Quality (NFR-2 to NFR-4)
| Area | Requirements |
|------|-------------|
| **Memory Management** | Automatic garbage collection via Rust ownership |
| **Usability** | Clear REPL prompts, readable output, multi-line input support |
| **Portability** | Cross-platform (Windows/macOS/Linux), minimal dependencies |

### Code Quality (NFR-5 to NFR-6)
| Principle | Requirements |
|-----------|-------------|
| **Maintainability** | Rust best practices, modular architecture, simplicity over features |
| **Implementation Standards** | Descriptive names, small focused functions, explicit over clever code |
| **Standards Compliance** | R7RS-small subset with documented deviations |

---

## Acceptance Criteria

### Basic Language Features

#### AC-1: Basic Arithmetic
```scheme
> (+ 1 2 3)
6
> (* 4 5)
20
> (/ 10 2)
5
```

#### AC-2: Function Definition and Application
```scheme
> (define square (lambda (x) (* x x)))
square
> (square 4)
16
```

#### AC-3: List Operations
```scheme
> (cons 1 (cons 2 '()))
(1 2)
> (car '(a b c))
a
> (cdr '(a b c))
(b c)
```

#### AC-4: Conditional Logic
```scheme
> (if (> 5 3) 'yes 'no)
yes
> (if #f 'true 'false)
false
```

#### AC-5: Function Parameters and Local Binding
```scheme
> ((lambda (x y) (+ x y)) 10 20)
30
> (define add-ten (lambda (x) (+ x 10)))
add-ten
> (add-ten 5)
15
```

### Error Handling and I/O

#### AC-6: Error Handling
```scheme
> (+ 1 'symbol)
Error: Type error - expected number, got symbol
> (car 42)
Error: Type error - expected pair, got number
```

#### AC-7: Fiber-Yielding I/O
```scheme
> (display "Hello, World!")
Hello, World!
> (newline)

```
*Note: I/O operations yield fiber execution but appear synchronous*

### Concurrency Features

#### AC-8: Fiber Completion System
```scheme
> (define fib (lambda (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2))))))
fib
> (define fiber1 (async (fib 35)))
fiber1
> (define fiber2 (async (fib 36)))
fiber2
> (fiber-wait fiber1)
9227465
> (fiber-wait fiber2)
14930352
```
*Note: Fibers execute in parallel and complete with values*

#### AC-9: Minimal Syntax Example
```scheme
> (define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))
factorial
> (factorial 5)
120
```

#### AC-10: Fiber Coordination
```scheme
> (define slow-work (lambda ()
    (display "Starting slow work\n")
    (display "Slow work complete\n")
    42))
slow-work
> (define fiber1 (async (slow-work)))
Starting slow work
fiber1
> (define fiber2 (async (+ 10 20)))
fiber2
> (fiber-wait fiber2)
30
> (fiber-wait fiber1)
Slow work complete
42
```

#### AC-11: Direct Fiber Management
```scheme
> (define worker-fiber (spawn-fiber (lambda ()
    (display "Independent worker running\n")
    (+ 100 200))))
Independent worker running
worker-fiber
> (define result (fiber-wait worker-fiber))
result
> result
300
```

#### AC-12: Basic Macro Usage
```scheme
> (define-syntax when
    (syntax-rules ()
      ((when condition body ...)
       (if condition (begin body ...)))))
when
> (when #t (display "Hello") (display " World"))
Hello World
```

---

## Out of Scope

### Excluded for Simplicity
- **Complex Language Features**
  - Continuations and call/cc
  - Module system (define-library, import, export)
  - Exception handling (guard, raise)
  - Dynamic binding, quasiquote, multiple return values
  - Eval procedure

- **Advanced Data Types**
  - Full numeric tower (complex, rationals, exact/inexact)
  - Vector and bytevector operations
  - Record types, parameter objects
  - Advanced character/string manipulation

### Excluded for Minimalism
- **I/O and System Features**
  - Synchronous file I/O (async alternatives considered)
  - Port operations beyond basic display
  - Environment variable access

- **Concurrency and Debugging**
  - Manual threading (handled by async runtime)
  - Debugging facilities, performance profiling

### Excluded by Core Constraints
- **ALL mutable operations** (violates immutability principle)
- Local binding forms (let, let*, letrec) for simplicity

---

## Educational Validation

### Learning Objective Validation

| Objective | Criteria | Evidence | Outcome |
|-----------|----------|----------|---------|
| **AI Collaboration** | Effective AI-assisted patterns | Task progression docs, commit history | Understanding of AI tool capabilities |
| **Interpreter Implementation** | Complete pipeline comprehension | Module separation, documentation | Deep language implementation knowledge |
| **Async Programming** | Practical async/concurrency mastery | Fiber scheduler with `smol` | Confidence in async Rust systems |
| **Functional Programming** | Immutability and closure understanding | Complete immutable implementation | Solid functional programming foundation |
| **Software Architecture** | Balance requirements with maintainability | Simple, readable code organization | System design for functionality and education |

### Success Metrics

#### Technical Completeness
- [ ] All 17 functional requirements implemented and tested
- [ ] All 12 acceptance criteria validated
- [ ] Comprehensive test suite with >90% coverage
- [ ] Complete and current documentation

#### Educational Achievement
- [ ] Clear learning progression through phases
- [ ] Extensive design decision documentation
- [ ] Code examples for key concepts
- [ ] AI collaboration lessons documented

#### Knowledge Transfer
- [ ] Project serves as learning resource
- [ ] Implementation rationale clearly documented
- [ ] Alternative approaches considered and documented
- [ ] Learning objectives measurably achieved

---

## Glossary

| Term | Definition |
|------|------------|
| **S-expression** | Symbolic expression, fundamental Scheme syntax |
| **REPL** | Read-Eval-Print Loop, interactive interpreter interface |
| **AST** | Abstract Syntax Tree, internal representation of parsed code |
| **Tail-call optimization** | Recursive calls in tail position reuse stack frames |
| **Lexical scoping** | Identifiers refer to bindings in enclosing lexical scope |
| **Closure** | Procedure capturing identifiers from defining environment |
| **Fiber** | Lightweight computation unit that completes with a value |
| **Fiber Scheduler** | Central component managing fiber execution and I/O yielding |
| **Fiber Handle** | Reference for fiber synchronization with fiber-wait |
| **Fiber Yielding** | Automatic suspension for I/O allowing other fibers to run |
| **Thread Pool** | Worker threads executing fibers without creation overhead |
| **GIL-free** | Architecture allowing true parallel execution |
| **Essential Subset** | Minimal language features for functional programming |
| **Implementation Simplicity** | Straightforward code over complex optimizations |
| **Binding** | Immutable association between identifier and value in environment |
| **Environment** | Scope managing identifier bindings with lexical scoping chains |
| **Identifier** | Symbol or name bound to value (immutable, not mutable) |
| **Symbol** | Immutable interned string for identifiers using SmolStr optimization |
| **Closure Environment** | Specialized environment capturing only required bindings for closures |
| **Shadowing** | Inner scope identifier hiding outer scope binding of same name |
| **Value** | Immutable data types: numbers, booleans, strings, symbols, lists, procedures |

---

This document serves as the definitive specification for Twine's requirements. All implementation decisions must align with these specifications, prioritizing educational value while maintaining technical rigor.