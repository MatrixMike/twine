# AI Coding Agent Instructions for Twine Scheme Interpreter

## Quick Reference

### Before Every Development Session
1. ✅ Read `requirements.md`, `design.md`, and `tasks.md`
2. ✅ Verify `deps/` structure is up-to-date
3. ✅ Check current phase and task dependencies

### After ANY Dependency Changes
1. ✅ Run `./scripts/vendor-deps.sh` immediately
2. ✅ Verify vendored sources updated in `deps/vendor/`
3. ✅ Confirm documentation updated in `deps/doc/`
4. ✅ Run tests to ensure compatibility

### When Using Third-Party APIs
1. ✅ Check source code in `deps/vendor/[crate-name]/`
2. ✅ Reference docs in `deps/docs/[crate-name]/`
3. ✅ Never guess - always verify locally

---

## Project Overview

**Twine** is an educational Scheme interpreter project designed for learning AI-assisted development, interpreter implementation, async I/O, parallelism, and advanced Rust concepts.

### Core Principles
1. **Fiber-based Concurrency**: Lightweight fiber scheduler with multi-threaded execution and no GIL
2. **Asynchronous I/O**: All I/O operations are async with fiber yielding
3. **Strict Immutability**: All data structures are immutable after creation
4. **Macro System**: R7RS-small macro support with `define-syntax` and `syntax-rules`
5. **Minimalism**: Essential language features only for maintainability and simplicity

### Educational Objectives
- **AI Agent Development**: Learn effective collaboration with AI coding agents
- **Interpreter Implementation**: Understand complete source-to-execution pipeline
- **Scheme Language**: Explore functional programming and Lisp-family syntax
- **Rust Async Ecosystem**: Hands-on experience with async programming and `smol`
- **Concurrency Models**: Implement fiber-based scheduling and parallelism
- **Software Architecture**: Balance technical requirements with learning value

---

## Core Documentation (MANDATORY READING)

These documents are the single source of truth. ALL implementation decisions must align with their specifications:

| Document | Purpose | When to Read |
|----------|---------|--------------|
| `requirements.md` | Functional/non-functional requirements, user stories, acceptance criteria | Before any development work |
| `design.md` | Technical architecture, component specs, concurrency model | Before implementation decisions |
| `tasks.md` | Structured implementation plan, phases, dependencies | Before starting new tasks |
| `README.md` | Project status and capabilities | Keep updated with new features |

### CRITICAL COMPLIANCE RULES
- ⚠️ **ZERO DEVIATIONS** permitted from documented specifications
- ✅ **ALWAYS** reference specific requirement numbers (FR-X, NFR-X, AC-X)
- ✅ **VERIFY** every implementation against design.md architecture
- ✅ **CHECK** acceptance criteria before marking any task complete
- 📋 **CITE** relevant sections when making implementation decisions
- 🚫 **NO ASSUMPTIONS** - if specification is unclear, ask for clarification

### Before Every Implementation Task
1. **Read requirements.md** - understand functional requirements and constraints
2. **Read design.md** - follow technical architecture and component specifications
3. **Check tasks.md** - verify dependencies and acceptance criteria
4. **Reference throughout** - cite specific sections during implementation

---

## Implementation Philosophy

### Educational First Approach
- **Simplicity over cleverness**: Prefer straightforward, readable code
- **Learning value prioritized**: Choose approaches that maximize educational benefit
- **Beginner-friendly**: Code should be understandable to Rust beginners
- **Progressive complexity**: Build understanding incrementally through phases

### Code Organization Principles
- **Logical module structure**: Break implementation into educational progression
- **Single responsibility**: Each module maps to specific learning concepts
- **Minimal dependencies**: Add only when absolutely necessary for current task
- **Clean interfaces**: Simple APIs that focus on essential concepts

### Quality Standards
- **Comprehensive testing**: Test each implemented feature thoroughly
- **Extensive documentation**: Explain design decisions and implementation choices
- **Rust best practices**: Follow idiomatic patterns while prioritizing simplicity
- **Descriptive naming**: Use clear, verbose names for functions and variables

---

## Dependency Management (CRITICAL)

### Mandatory Workflow
```
1. Modify Cargo.toml
2. IMMEDIATELY run ./scripts/vendor-deps.sh
3. Verify vendored sources in deps/vendor/
4. Check documentation in deps/doc/
5. Run tests to ensure compatibility
```

### Critical Rules
- ⚠️ **NEVER guess about third-party APIs** - always check local sources
- ⚠️ **ALL async dependencies MUST be from smol ecosystem**
- ⚠️ **RUN vendor-deps script after EVERY Cargo.toml change**
- ⚠️ **USE local vendored sources for all dependency analysis**

### Local Dependency Resources
| Location | Purpose | Usage |
|----------|---------|--------|
| `deps/vendor/[crate]/` | Exact source code | Understanding implementation details |
| `deps/doc/[crate]/` | Complete API docs | API reference and examples |
| `Cargo.lock` | Version constraints | Troubleshooting conflicts |

### Dependency Analysis Process
1. **Source Analysis**: Read actual code in `deps/vendor/[crate-name]/`
2. **API Documentation**: Reference `deps/docs/[crate-name]/index.html`
3. **Version Verification**: Ensure compatibility with our requirements
4. **Integration Planning**: Design how dependency fits our architecture

### Troubleshooting Dependencies
| Issue | Solution |
|-------|----------|
| Compilation errors | Check source in `deps/vendor/[crate]/src/` |
| Version conflicts | Review `Cargo.lock` and vendored sources |
| Missing documentation | Re-run `./scripts/update-deps.sh` |
| Outdated sources | Always run update script after Cargo.toml changes |

---

## Technical Constraints

### Immutability Requirements
- ALL data structures must be immutable after creation
- No mutable operations permitted on data structures
- Side effects (I/O, display) allowed but must not mutate existing data
- Use Rust's ownership system and reference counting for shared data

### Concurrency Model
- Use fiber-based parallelism with `smol` runtime
- Support multi-threaded execution without GIL
- All I/O must be asynchronous with fiber yielding
- Maintain hierarchical task relationships with parent-child cleanup

### Language Feature Constraints
- Implement only R7RS-small subset specified in requirements
- Focus on essentials: arithmetic, lists, functions, conditionals, macros
- Support lexical scoping with closures

### Symbol Type Usage
- **ALWAYS use `Symbol` or `&Symbol`** when working with Scheme identifiers
- **NEVER use `String` or `&str`** for identifier parameters in internal functions
- **Exception**: Error functions and string literals use `&str` for general-purpose APIs
- **Performance**: Avoids unnecessary `Symbol → &str → Symbol` conversions
- **Examples**:
  - Function parameters: `fn lookup(identifier: &Symbol)` not `fn lookup(identifier: &str)`
  - Collections: `HashMap<Symbol, Value>` not `HashMap<String, Value>`
  - Comparisons: `if symbol1 == symbol2` not `if symbol1.as_str() == symbol2.as_str()`
- **Rationale**: `Symbol` is optimized for identifier use with efficient storage and comparison
- Provide both REPL and file execution modes

---

## Development Workflow

### Starting New Work
1. **Read Documentation**: Review relevant sections of core docs
   - **MANDATORY**: Read `requirements.md` for functional requirements (FR-X, NFR-X)
   - **MANDATORY**: Read `design.md` for technical architecture and component specs
   - **MANDATORY**: Read `tasks.md` for current phase and task dependencies
2. **Identify Current Task**: Check `tasks.md` for current phase and requirements
3. **Verify Dependencies**: Ensure task dependencies are satisfied
4. **Understand Acceptance Criteria**: Know what success looks like (AC-X criteria)
5. **Check Local Dependencies**: Ensure `deps/` structure is current

### Implementation Process
1. **Start Simple**: Begin with minimal working implementation
2. **Test-Driven Development**: Write tests first when possible
3. **Check Dependencies Locally**: Use `deps/vendor/` and `deps/docs/` before coding
4. **Update Dependencies**: Run `./scripts/update-deps.sh` after any Cargo.toml changes
5. **Maintain Tests**: Ensure all existing tests continue passing
6. **Document Decisions**: Explain educational value and design choices

### Completing Tasks
1. **Verify Acceptance Criteria**: Ensure all requirements met (cite specific AC-X numbers)
2. **Run Full Test Suite**: Confirm no regressions
3. **Update Documentation**: Reflect new capabilities in README.md
4. **Mark Progress**: Update `tasks.md` with completion status
5. **Identify Next Task**: Determine next priority in sequence

---

## Agent Behavioral Instructions Management

### Living Memory Principle
This document (`agent.md`) serves as **living memory** for all agent behavioral instructions. When you provide new instructions, they MUST be incorporated here to ensure persistence across conversations.

### Update Process
1. **Immediate Application**: New instructions take effect immediately
2. **Automatic Documentation**: Update this document without being asked
3. **Conflict Resolution**: Integrate with existing guidelines, resolve conflicts explicitly
4. **Persistence Guarantee**: Ensure instructions survive conversation boundaries

### Instruction Types
- **Development Approach**: Coding style, architecture preferences, implementation strategies
- **Communication Style**: How to explain, document, or interact
- **Workflow Adjustments**: Process changes, testing approaches, task management
- **Tool Usage**: Preferences for tools, dependencies, development environments
- **Quality Standards**: Code quality expectations, documentation requirements

### Conflict Resolution Rules
1. **Explicit Override**: New instructions override conflicting existing ones
2. **Contextual Application**: Specify when different approaches apply
3. **Priority Clarification**: Establish precedence rules for conflicts
4. **Documentation Update**: Reflect resolved approach in this document

---

## Code Style Guidelines

### Import Style
- **Place imports at module top**: All `use` statements must be at the top of the file, never within functions
- **Exception for test modules**: `use` statements within test functions are acceptable for test-specific imports
- **Organize imports logically**: Group standard library, external crates, and local crate imports separately

### Commenting Style
- **Focus on Functionality**: Comments should describe what code does and why technical decisions were made
- **Include Important "Why"**: Explain technical rationale, performance considerations, safety concerns, and non-obvious design choices
- **Avoid Requirements Fluff**: Remove references to requirements compliance (FR-X, NFR-X), design principles, or educational goals from code comments
- **Keep Practical**: Include usage examples, doctests, and functional descriptions
- **Module Documentation**: Keep module docs concise - describe what the module provides, not architectural rationale

#### Examples of Good Comments
```rust
/// Lexical analyzer for Scheme source code.
/// Maintains line and column tracking for precise error reporting.
pub struct Lexer { ... }

/// Uses Box because recursive enum variants would have infinite size.
/// Box provides heap allocation to break the recursion.
Quote(Box<Expression>),

/// Wraps Arc<String> to enable efficient sharing across multiple threads
/// while maintaining immutability guarantees required by Scheme semantics.
pub struct ArcString(Arc<String>);
```

#### Examples to Avoid
```rust
/// Lexical analyzer following FR-2 requirements and educational principles...
/// This design prioritizes learning value by implementing...
/// Requirements Compliance: ✅ FR-2 (Syntactic Analysis)...
```

---

## Communication Guidelines

### Reference Standards (MANDATORY)
- **Requirements**: ALWAYS cite specific requirement numbers (FR-X, NFR-X)
- **Design**: ALWAYS reference relevant sections from design.md
- **Acceptance Criteria**: ALWAYS quote specific criteria (AC-X) when validating
- **Core Principles**: ALWAYS explain alignment with four core principles
- **Documentation Compliance**: NEVER implement without citing source specifications

### Dependency Communication
- **Local Sources**: Reference specific files in `deps/vendor/` and `deps/doc/`
- **Update Reminders**: Mention when `./scripts/vendor-deps.sh` needs running
- **Verification**: Confirm dependency compatibility with project constraints

### Educational Focus
- **Learning Value**: Explain educational benefits of implementation choices
- **Simplicity Rationale**: Describe why simple solutions were chosen
- **Module Organization**: Explain how code organization supports learning
- **Concept Reinforcement**: Show how implementations build on previous concepts

### Scheme Terminology (CRITICAL)
**NEVER use "variable" - always use proper Scheme terminology:**
- **Identifier** = the symbolic name (e.g., `x`, `my-function`, `calculate`)
- **Binding** = the association between an identifier and a value in an environment
- **"Variable"** is an imperative programming concept - avoid this term entirely
- **Examples**: "unbound identifier", "identifier binding", "binding lookup"
- **Error messages**: Use "Unbound identifier" not "Undefined variable"
- **Comments**: "Define an identifier binding" not "Define a variable"
- **Shadowing**: Shadowing is normal behavior in Scheme - no warnings needed

---

## Error Handling and Debugging

### Error Management
- Use Rust's `Result` type for error propagation
- Provide clear, informative error messages for Scheme syntax and runtime errors
- Implement proper error handling for async operations
- Maintain error context through the fiber scheduler

### Debugging Approach
- Add descriptive logging statements to track variable and code state
- Create test functions to isolate problems
- Address root causes instead of symptoms
- Use debugging output when helpful for development

### Rust Diagnostics Management
- **Fix All Diagnostics**: Always fix compiler errors, warnings, and clippy lints when encountered
- **Run Clippy**: Execute `cargo clippy` after making any Rust code changes
- **Zero Tolerance**: No warnings or lints should remain in committed code
- **Immediate Resolution**: Address diagnostics as soon as they appear
- **Use Diagnostics Tool**: Run diagnostics checks regularly during development
- **Auto-fix When Possible**: Use `cargo clippy --fix --allow-dirty` for automatic fixes
- **Common Issues**: Pay attention to unused imports, dead code, inefficient patterns, and style violations

### Code Formatting Requirements
- **Always Format**: Run `cargo fmt` after making any changes to Rust source code
- **Consistent Style**: Maintain consistent formatting across the entire codebase  
- **Pre-Commit**: Format code before considering any implementation complete
- **IDE Integration**: Configure development environment to format on save when possible

---

## Development Quality Checklist

### After Every Code Change
1. ✅ **Run Clippy**: Execute `cargo clippy` and fix all lints and warnings
2. ✅ **Fix Diagnostics**: Run diagnostics and resolve all errors, warnings, lints
3. ✅ **Format Code**: Run `cargo fmt` to maintain consistent style
4. ✅ **Run Tests**: Ensure all existing tests pass
5. ✅ **Update Documentation**: Reflect changes in relevant docs

### Before Marking Tasks Complete
1. ✅ **Clean Diagnostics**: Zero compiler warnings or clippy lints
2. ✅ **Formatted Code**: All Rust files properly formatted
3. ✅ **Acceptance Criteria**: All specified criteria met
4. ✅ **Test Coverage**: New functionality tested appropriately

---

## File Maintenance Requirements

### Always Keep Updated
| File | Update Trigger | Required Changes |
|------|----------------|------------------|
| `README.md` | New features added | Project overview, capabilities, examples |
| `requirements.md` | Requirements change | Functional requirements, acceptance criteria |
| `design.md` | Architecture changes | Technical specs, implementation details |
| `tasks.md` | Task completion | Progress tracking, completion status |
| `agent.md` | New behavioral instructions | Agent preferences and approaches |

### Critical Maintenance Rules

#### Progress Tracking in tasks.md
**MANDATORY**: When updating tasks, ALWAYS update the "Progress Tracking" section:
- Current completion status for each phase
- Accurate count of completed vs total tasks
- List of completed tasks
- Current status of active subsections
- Next priority task

#### Agent Instructions in agent.md
**MANDATORY**: When given new behavioral instructions, automatically update this file without being asked to ensure instructions persist across conversations.

---

## Examples and Common Scenarios

### Adding a New Dependency
```bash
# 1. Add to Cargo.toml
[dependencies]
new-crate = "1.0"

# 2. IMMEDIATELY run dependency management
./scripts/vendor-deps.sh

# 3. Verify sources updated
ls deps/vendor/new-crate/

# 4. Check documentation
open deps/doc/new-crate/index.html

# 5. Run tests
cargo test
```

### Using Third-Party APIs
```rust
// DON'T: Guess about API behavior
// some_crate::function(); // What does this return?

// DO: Check local documentation first
// 1. Read deps/vendor/some-crate/src/lib.rs
// 2. Reference deps/doc/some-crate/index.html
// 3. Understand exact behavior before using

use some_crate::Function; // Now I know exactly what this does
```

### Starting a New Implementation Phase
1. Read current task requirements in `tasks.md`
2. Check dependencies are satisfied
3. Review acceptance criteria
4. Plan minimal implementation approach
5. Write tests for expected behavior
6. Implement using local dependency docs
7. Update progress tracking

---

This document evolves with the project and your preferences. It serves as the foundation for all development decisions and agent behavior.