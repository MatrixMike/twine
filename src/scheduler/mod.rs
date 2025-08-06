//! Fiber scheduler infrastructure for lightweight concurrency
//!
//! This module provides the core building blocks for fiber-based concurrency:
//! - Individual fibers with state tracking and lifecycle management
//! - Suspension reasons for different blocking conditions
//! - Main scheduler for fiber execution and coordination
//! - Async operations and coordination primitives
//! - Foundation for the complete concurrent execution system

// Re-export core types and structures
pub use async_ops::{AsyncFiberOps, FiberWait, TaskHandle};
pub use scheduler::FiberScheduler;
pub use types::{Fiber, FiberId, FiberState, SuspendReason};

// Module declarations
mod async_ops;
mod scheduler;
mod types;
