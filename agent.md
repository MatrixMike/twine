# AI Coding Agent Instructions for Twine Scheme Interpreter

## Project Overview

You are working on **Twine**, a minimalist Scheme interpreter written in Rust that implements a functional subset of R7RS-small Scheme. Twine is designed around four core principles:

1. **Fiber-based Concurrency**: Lightweight fiber scheduler with multi-threaded execution and no Global Interpreter Lock
2. **Asynchronous I/O**: All I/O operations are async with fiber yielding, appearing synchronous to Scheme code  
3. **Strict Immutability**: All data structures are immutable after creation (side effects like I/O are still supported)
4. **Macro System**: R7RS-small macro support with `define-syntax` and `syntax-rules`
5. **Minimalism**: Essential language features only for maintainability and simplicity

The project uses the `smol` async runtime for concurrency and provides both low-level fiber management and high-level async task abstraction.

## Core Documentation - ALWAYS CONSULT THESE

Before responding to any development request, you MUST:

1. **Read and reference `requirements.md`** - Contains all functional and non-functional requirements, user stories, acceptance criteria, and design principles
2. **Read and reference `design.md`** - Contains technical architecture, component specifications, concurrency model, and implementation details  
3. **Read and reference `tasks.md`** - Contains the structured implementation plan with phases, dependencies, and specific task requirements
4. **Keep `README.md` updated** - Ensure it reflects current project status and capabilities

These documents are the single source of truth for the project. All implementation decisions must align with their specifications.

## Implementation Philosophy

### Incremental Development
- Follow the phase-based approach outlined in `tasks.md`
- Implement ONLY the features described in the current task
- Do not add forward-looking code or stubs for future functionality
- Each task should produce the minimal working implementation

### Dependency Management
- Keep Rust dependencies to an absolute minimum
- Add dependencies only when actually needed for current task
- All async-related crates MUST come from the smol ecosystem (https://github.com/smol-rs)
- Avoid dependencies with large dependency trees
- Update `Cargo.toml` incrementally, not all at once

### Code Quality Standards
- Write comprehensive tests for each implemented feature
- Follow Rust best practices and idiomatic code patterns
- Ensure all tests pass before marking a task complete
- Maintain clear separation between core components
- Document complex algorithms and design decisions

## Key Technical Constraints

### Immutability
- ALL data structures must be immutable after creation
- No mutable operations are permitted on data structures
- Side effects (I/O, display) are allowed but must not mutate existing data
- Use Rust's ownership system and reference counting for shared data

### Concurrency Model
- Use fiber-based parallelism with `smol` runtime
- Support multi-threaded execution without GIL
- All I/O must be asynchronous with fiber yielding
- Maintain hierarchical task relationships with parent-child cleanup

### Language Features
- Implement only the R7RS-small subset specified in requirements
- Focus on essential features: arithmetic, lists, functions, conditionals, macros
- Support lexical scoping with closures
- Provide both REPL and file execution modes

## Development Workflow

### When Starting New Work
1. Read the relevant sections of `requirements.md`, `design.md`, and `tasks.md`
2. Identify the current phase and specific task requirements
3. Check task dependencies are satisfied
4. Understand acceptance criteria and test requirements

### When Implementing Features
1. Follow the minimal implementation principle
2. Write tests first when possible (TDD approach)
3. Implement only the current task's functionality
4. Ensure all existing tests continue to pass
5. Update documentation if the implementation affects user-facing behavior

### When Completing Tasks
1. Verify all acceptance criteria are met
2. Run comprehensive test suite
3. Update `README.md` if new capabilities are added
4. Mark task as complete in `tasks.md` if appropriate
5. Identify next task in the sequence

## Error Handling and Debugging

- Use Rust's Result type for error propagation
- Provide clear, informative error messages for Scheme syntax and runtime errors
- Implement proper error handling for async operations
- Maintain error context through the fiber scheduler
- Add debugging output when helpful for development

## Communication Guidelines

- Reference specific requirement numbers (FR-X, NFR-X) when explaining design decisions
- Cite relevant sections from design.md when discussing architecture
- Quote acceptance criteria (AC-X) when validating implementations
- Always explain how your solution aligns with the four core principles
- Update the core documents when requirements or design change

## Files to Keep Updated

As you implement features and make changes, ensure these files remain current:

- **`README.md`**: Project overview, features, quick start examples, architecture summary
- **`requirements.md`**: Functional requirements, acceptance criteria, design principles  
- **`design.md`**: Technical architecture, component specifications, implementation details
- **`tasks.md`**: Implementation progress, completed tasks, current status

Remember: These documents are not just reference material - they are living specifications that should evolve with the project. Keep them accurate and up-to-date as the foundation for all development decisions.