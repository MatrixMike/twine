# AI Coding Agent Instructions for Twine Scheme Interpreter

## Project Overview

You are working on **Twine**, an educational Scheme interpreter project designed primarily for learning AI-assisted development, interpreter implementation, async I/O, parallelism, and advanced Rust concepts. Twine implements a functional subset of R7RS-small Scheme and is designed around four core principles:

1. **Fiber-based Concurrency**: Lightweight fiber scheduler with multi-threaded execution and no Global Interpreter Lock
2. **Asynchronous I/O**: All I/O operations are async with fiber yielding, appearing synchronous to Scheme code  
3. **Strict Immutability**: All data structures are immutable after creation (side effects like I/O are still supported)
4. **Macro System**: R7RS-small macro support with `define-syntax` and `syntax-rules`
5. **Minimalism**: Essential language features only for maintainability and simplicity

The project uses the `smol` async runtime for concurrency and provides both low-level fiber management and high-level async task abstraction.

## Educational Objectives

This project serves as a comprehensive learning platform with the following primary educational goals:

- **AI Agent Development**: Learn effective collaboration patterns with AI coding agents (specifically Zed Agentic Editing)
- **Interpreter Implementation**: Understand the complete pipeline from source code to execution
- **Scheme Language**: Explore functional programming concepts and Lisp-family syntax
- **Rust Async Ecosystem**: Gain hands-on experience with async programming and the `smol` library
- **Concurrency Models**: Implement and understand fiber-based scheduling and parallelism
- **Software Architecture**: Make design decisions that balance technical requirements with learning value

All implementation decisions should prioritize educational value and learning opportunities while maintaining technical rigor.

## Agent Behavioral Instructions Management

### Incorporating User Instructions
This document (`agent.md`) serves as the **living memory** for all agent behavioral instructions and preferences. When you provide instructions to change how I should behave, work, or approach tasks, these instructions MUST be incorporated into this document to ensure they are remembered and consistently applied.

### Process for Behavioral Updates
1. **Immediate Application**: New behavioral instructions take effect immediately for the current conversation
2. **Documentation Requirement**: All behavioral changes MUST be added to the appropriate section of this document
3. **Integration Principle**: New instructions should be integrated with existing guidelines, resolving any conflicts explicitly
4. **Persistence Guarantee**: By updating this document, instructions persist across all future interactions
5. **Automatic Updates**: When the user provides new behavioral instructions, I MUST automatically update this document without being asked

### Types of Behavioral Instructions
- **Development Approach**: Changes to coding style, architecture preferences, or implementation strategies
- **Communication Style**: Modifications to how I should explain, document, or interact
- **Workflow Adjustments**: Updates to development processes, testing approaches, or task management
- **Tool Usage**: Preferences for specific tools, dependencies, or development environments
- **Quality Standards**: Changes to code quality expectations, documentation requirements, or testing standards

### Conflict Resolution
When new instructions conflict with existing guidelines:
1. **Explicit Override**: New instructions explicitly override conflicting existing ones
2. **Contextual Application**: Specify when different approaches should be used
3. **Priority Clarification**: Establish clear precedence rules for conflicting guidance
4. **Documentation Update**: Update this document to reflect the resolved approach

### Instruction Categories
- **MANDATORY**: Must always be followed (marked with bold and imperative language)
- **PREFERRED**: Default approach unless circumstances require otherwise
- **CONTEXTUAL**: Applied in specific situations or phases
- **DEPRECATED**: Previous instructions that have been superseded

Remember: This document is not just a reference - it is my behavioral specification that evolves with your preferences and project needs.

## Core Documentation - ALWAYS CONSULT THESE

Before responding to any development request, you MUST:

1. **Read and reference `requirements.md`** - Contains all functional and non-functional requirements, user stories, acceptance criteria, and design principles
2. **Read and reference `design.md`** - Contains technical architecture, component specifications, concurrency model, and implementation details  
3. **Read and reference `tasks.md`** - Contains the structured implementation plan with phases, dependencies, and specific task requirements
4. **Keep `README.md` updated** - Ensure it reflects current project status and capabilities

These documents are the single source of truth for the project. All implementation decisions must align with their specifications.

## Implementation Philosophy

### Educational Simplicity and Clarity
- **Keep implementation simple and easy to understand** (maximize learning value)
- **Avoid unnecessary usage of advanced Rust features or abstractions** (focus on fundamental concepts)
- **Prefer straightforward, readable code over clever optimizations** (prioritize educational clarity)
- Use Rust's basic features effectively: structs, enums, pattern matching, ownership (build solid foundations)
- Avoid complex trait hierarchies, excessive generics, or advanced lifetime patterns unless absolutely necessary
- Write code that a Rust beginner could understand and maintain (serve as learning resource)
- **Include extensive documentation explaining design decisions and implementation choices**

### Learning-Oriented Module Organization
- **Break up implementation logic into logical modules/folders that support educational progression**
- Each module should have a clear, single responsibility that maps to specific learning concepts
- Group related functionality together (e.g., lexer/, parser/, types/, runtime/) to reinforce domain understanding
- Use simple file and folder structures that reflect the domain concepts clearly
- Keep module interfaces clean with minimal public APIs (focus on essential concepts)
- Organize code to minimize cross-module dependencies (build understanding incrementally)
- **Document each module's educational purpose and learning objectives**

### Educational Incremental Development
- Follow the phase-based approach outlined in `tasks.md` (designed for optimal learning progression)
- Implement ONLY the features described in the current task (focus learning on specific concepts)
- Do not add forward-looking code or stubs for future functionality (avoid overwhelming complexity)
- Each task should produce the minimal working implementation that maximizes learning value
- **Ensure each task builds understanding progressively and reinforces previous concepts**

### Dependency Management
- Keep Rust dependencies to an absolute minimum
- Add dependencies only when actually needed for current task
- All async-related crates MUST come from the smol ecosystem (https://github.com/smol-rs)
- Avoid dependencies with large dependency trees
- Update `Cargo.toml` incrementally, not all at once

#### Local Dependency Source Management - CRITICAL
- **ALWAYS run `./scripts/update-deps.sh` after ANY dependency changes** to maintain local sources
- **MANDATORY**: After adding, removing, or updating dependencies in `Cargo.toml`, you MUST run the update script
- **USE LOCAL DEPENDENCY SOURCES**: Always reference vendored sources in `deps/vendor/` for accurate dependency code
- **USE LOCAL DOCUMENTATION**: Always reference generated docs in `deps/docs/` for complete API information
- **NO GUESSING**: Never guess about third-party APIs - always check `deps/vendor/` and `deps/docs/`
- **COMPLETE CONTEXT**: Use local sources to understand exact dependency behavior and implementation details

### Code Quality Standards
- Write comprehensive tests for each implemented feature
- Follow Rust best practices and idiomatic code patterns, but prioritize simplicity
- Ensure all tests pass before marking a task complete
- Maintain clear separation between core components through logical module boundaries
- Document complex algorithms and design decisions
- **Favor explicit, verbose code over implicit, clever code**
- **Use descriptive names for functions, variables, and modules**
- **Keep functions and modules small and focused on single responsibilities**

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
- **Implement features in the most straightforward way possible**
- **Avoid premature optimization or complex implementation patterns**

### Dependency Management (CRITICAL)
- **MANDATORY**: Always use local vendored sources in `deps/vendor/` for dependency analysis
- **MANDATORY**: Always reference local documentation in `deps/docs/` for API information
- **MANDATORY**: Run `./scripts/update-deps.sh` immediately after ANY `Cargo.toml` changes
- **FORBIDDEN**: Guessing about third-party APIs without checking local sources
- **REQUIRED**: Verify all async dependencies come from smol ecosystem using local docs
- **CONSTRAINT**: All dependency analysis must use exact vendored source code, not assumptions

## Development Workflow

### When Starting New Work
1. Read the relevant sections of `requirements.md`, `design.md`, and `tasks.md`
2. Identify the current phase and specific task requirements
3. Check task dependencies are satisfied
4. Understand acceptance criteria and test requirements
5. **VERIFY LOCAL DEPENDENCIES**: Ensure `deps/` structure exists and is up-to-date

### When Implementing Features
1. Follow the minimal implementation principle
2. **Start with the simplest possible implementation that works**
3. **Organize code into logical modules/files based on functionality**
4. Write tests first when possible (TDD approach)
5. Implement only the current task's functionality
6. **CHECK DEPENDENCIES LOCALLY**: Before using any third-party APIs, check `deps/vendor/` for source code and `deps/docs/` for documentation
7. **UPDATE DEPENDENCIES**: If adding/changing dependencies, run `./scripts/update-deps.sh` immediately
8. Ensure all existing tests continue to pass
9. Update documentation if the implementation affects user-facing behavior
10. **Refactor for clarity and simplicity, not performance**

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
- **CITE LOCAL SOURCES**: When discussing third-party dependencies, reference specific files in `deps/vendor/` and `deps/docs/`
- **DEPENDENCY UPDATES**: Always mention when `./scripts/update-deps.sh` needs to be run
- **EDUCATIONAL VALUE**: Explain the learning opportunities and educational benefits of implementation choices
- **SIMPLICITY FOCUS**: Explain why simple solutions were chosen over complex alternatives (emphasize learning value)
- **MODULE ORGANIZATION**: Describe how code organization supports both maintainability and educational understanding
- **CONCEPT REINFORCEMENT**: Explain how each implementation reinforces or builds upon previously learned concepts

## Dependency Management Workflow

### Critical Dependency Rules
**EVERY TIME** you modify `Cargo.toml` (add, remove, or update dependencies), you MUST:

1. **IMMEDIATELY** run `./scripts/update-deps.sh` after the change
2. **VERIFY** the script completes successfully 
3. **CONFIRM** updated sources exist in `deps/vendor/`
4. **CHECK** that tests still pass with updated dependencies

### Dependency Analysis Process
When working with third-party crates, follow this mandatory sequence:

1. **SOURCE ANALYSIS**: Read actual source code in `deps/vendor/[crate-name]/`
   - Understand implementation details, not just public APIs
   - Check for async/sync compatibility with our fiber model
   - Verify immutability constraints are respected

2. **API DOCUMENTATION**: Reference `deps/docs/[crate-name]/index.html`
   - Review all public and private documentation
   - Check feature flags and optional functionality
   - Understand error types and failure modes

3. **VERSION VERIFICATION**: Ensure compatibility
   - Confirm version matches our requirements
   - Check for breaking changes in changelog
   - Verify smol ecosystem compatibility for async crates

4. **INTEGRATION PLANNING**: Before writing code
   - Plan how the dependency fits our architecture
   - Ensure no conflicts with immutability principle
   - Design error handling integration

### Dependency Update Checklist
- [ ] Modified `Cargo.toml`
- [ ] Ran `./scripts/update-deps.sh` successfully
- [ ] Verified vendored sources updated in `deps/vendor/`
- [ ] Confirmed documentation updated in `deps/docs/`
- [ ] All tests pass with new dependencies
- [ ] No conflicts with project constraints (immutability, smol ecosystem)

### Dependency Troubleshooting
If you encounter issues with dependencies:

1. **Compilation Errors**: Check `deps/vendor/[crate]/src/` for actual source and understand the API
2. **Version Conflicts**: Review `Cargo.lock` and vendored sources to identify incompatibilities
3. **Missing Documentation**: Re-run `./scripts/update-deps.sh` to regenerate docs
4. **Outdated Vendored Sources**: Always run update script after changing `Cargo.toml`
5. **smol Ecosystem Conflicts**: Verify async crates are from smol ecosystem (check `deps/docs/`)

**NEVER assume** how a dependency works - always verify by checking the actual source code in `deps/vendor/`.

## Third-Party Code Integration

### Mandatory Process for External Dependencies
1. **BEFORE IMPLEMENTATION**: Check `deps/vendor/[crate-name]/` for exact source code
2. **REFERENCE DOCUMENTATION**: Use `deps/docs/[crate-name]/` for complete API reference
3. **UNDERSTAND INTERNALS**: Review dependency source to understand behavior, not just public APIs
4. **VERIFY COMPATIBILITY**: Ensure dependency versions match project requirements
5. **UPDATE AFTER CHANGES**: Run `./scripts/update-deps.sh` after any Cargo.toml modifications

### Local Dependency Resources
- **Source Code**: `deps/vendor/` contains exact vendored source for all dependencies
- **Documentation**: `deps/docs/` contains comprehensive docs including private items
- **Version Lock**: Sources match exact versions in Cargo.lock for consistency
- **Offline Access**: Complete dependency information available without network access

## Files to Keep Updated

As you implement features and make changes, ensure these files remain current:

- **`README.md`**: Project overview, features, quick start examples, architecture summary
- **`requirements.md`**: Functional requirements, acceptance criteria, design principles  
- **`design.md`**: Technical architecture, component specifications, implementation details
- **`tasks.md`**: Implementation progress, completed tasks, current status
- **`agent.md`**: Agent behavioral instructions and preferences (THIS FILE - update automatically when given new instructions)

### Critical File Maintenance Rules

**MANDATORY - Progress Tracking Maintenance**: When updating tasks in `tasks.md`, you MUST always update the "Progress Tracking" section at the bottom of the document. This section must reflect:
- Current completion status for each phase
- Accurate count of completed vs total tasks
- List of completed tasks
- Current status of active subsections
- Next priority task to work on

**MANDATORY - Agent Instructions**: When given new behavioral instructions, you MUST automatically update this `agent.md` file without being asked to ensure instructions persist across conversations.

Remember: These documents are not just reference material - they are living specifications that should evolve with the project. Keep them accurate and up-to-date as the foundation for all development decisions.