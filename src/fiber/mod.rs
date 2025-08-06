//! Fiber-based concurrency infrastructure for lightweight parallelism
//!
//! This module provides the core building blocks for fiber-based concurrency:
//! - Individual fibers with state tracking and lifecycle management
//! - Suspension reasons for different blocking conditions
//! - Fiber scheduler for execution coordination
//! - Fiber executor for async operations and task management
//! - Foundation for the complete concurrent execution system

// Re-export core types and structures
pub use executor::{FiberExecutor, FiberTask, FiberWait};
pub use scheduler::FiberScheduler;
pub use types::{Fiber, FiberId, FiberState, SuspendReason};

// Module declarations
mod executor;
mod scheduler;
mod types;
