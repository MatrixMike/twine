//! Core types for the fiber scheduler system
//!
//! This module defines the fundamental data structures used throughout
//! the fiber-based concurrency system:
//! - Fiber identifiers and states
//! - Suspension reasons for blocked fibers
//! - Common type aliases and utilities

use crate::Result;
use crate::types::Value;
use std::collections::HashSet;
use std::fmt::{self, Debug, Formatter};

/// Unique identifier for a fiber
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FiberId(u64);

impl FiberId {
    /// Create a new fiber ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the inner ID value
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// State of a fiber in the scheduler
#[derive(Debug, Clone)]
pub enum FiberState {
    /// Fiber is ready to run and waiting in the ready queue
    Ready,
    /// Fiber is currently executing on a thread
    Running,
    /// Fiber is suspended and waiting for something
    Suspended(SuspendReason),
    /// Fiber has completed execution with a result
    Completed(Result<Value>),
}

/// Reason why a fiber is suspended
#[derive(Debug, Clone)]
pub enum SuspendReason {
    /// Waiting for an I/O operation to complete
    IoOperation(String), // Simplified for now - will hold actual I/O future later
    /// Waiting for another fiber to complete
    WaitingForFiber(FiberId),
    /// Explicitly yielded by the fiber
    Yielded,
}

/// A lightweight execution unit managed by the fiber scheduler
pub struct Fiber {
    /// Unique identifier for this fiber
    pub id: FiberId,
    /// Current state of the fiber
    pub state: FiberState,
    /// The continuation representing the fiber's execution
    pub continuation: std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>>,
    /// Parent fiber that spawned this one (if any)
    pub parent: Option<FiberId>,
    /// Child fibers spawned by this fiber
    pub children: HashSet<FiberId>,
}

impl Fiber {
    /// Create a new fiber with the given id and continuation
    pub fn new(
        id: FiberId,
        continuation: std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>>,
        parent: Option<FiberId>,
    ) -> Self {
        Self {
            id,
            state: FiberState::Ready,
            continuation,
            parent,
            children: HashSet::new(),
        }
    }

    /// Check if the fiber is in a ready state
    pub fn is_ready(&self) -> bool {
        matches!(self.state, FiberState::Ready)
    }

    /// Check if the fiber is running
    pub fn is_running(&self) -> bool {
        matches!(self.state, FiberState::Running)
    }

    /// Check if the fiber is suspended
    pub fn is_suspended(&self) -> bool {
        matches!(self.state, FiberState::Suspended(_))
    }

    /// Check if the fiber has completed execution
    pub fn is_completed(&self) -> bool {
        matches!(self.state, FiberState::Completed(_))
    }

    /// Set the fiber to running state
    pub fn set_running(&mut self) {
        self.state = FiberState::Running;
    }

    /// Set the fiber to ready state
    pub fn set_ready(&mut self) {
        self.state = FiberState::Ready;
    }

    /// Suspend the fiber with the given reason
    pub fn suspend(&mut self, reason: SuspendReason) {
        self.state = FiberState::Suspended(reason);
    }

    /// Complete the fiber with the given result
    pub fn complete(&mut self, result: Result<Value>) {
        self.state = FiberState::Completed(result);
    }

    /// Add a child fiber
    pub fn add_child(&mut self, child_id: FiberId) {
        self.children.insert(child_id);
    }

    /// Remove a child fiber
    pub fn remove_child(&mut self, child_id: FiberId) {
        self.children.remove(&child_id);
    }
}

impl Debug for Fiber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fiber")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("parent", &self.parent)
            .field("children", &self.children)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;
    use std::future;

    fn create_test_fiber() -> Fiber {
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        Fiber::new(FiberId::new(1), future, None)
    }

    #[test]
    fn test_fiber_creation() {
        let fiber = create_test_fiber();
        assert_eq!(fiber.id.as_u64(), 1);
        assert!(fiber.is_ready());
        assert_eq!(fiber.parent, None);
        assert!(fiber.children.is_empty());
    }

    #[test]
    fn test_fiber_state_transitions() {
        let mut fiber = create_test_fiber();

        // Test ready -> running
        fiber.set_running();
        assert!(fiber.is_running());

        // Test running -> suspended
        fiber.suspend(SuspendReason::Yielded);
        assert!(fiber.is_suspended());

        // Test suspended -> ready
        fiber.set_ready();
        assert!(fiber.is_ready());

        // Test ready -> completed
        fiber.complete(Ok(Value::Number(42.into())));
        assert!(fiber.is_completed());
    }

    #[test]
    fn test_fiber_children_management() {
        let mut parent = create_test_fiber();
        let child1_id = FiberId::new(2);
        let child2_id = FiberId::new(3);

        // Add children
        parent.add_child(child1_id);
        parent.add_child(child2_id);
        assert_eq!(parent.children.len(), 2);
        assert!(parent.children.contains(&child1_id));
        assert!(parent.children.contains(&child2_id));

        // Remove a child
        parent.remove_child(child1_id);
        assert_eq!(parent.children.len(), 1);
        assert!(!parent.children.contains(&child1_id));
        assert!(parent.children.contains(&child2_id));
    }

    #[test]
    fn test_suspend_reason_creation() {
        let io_reason = SuspendReason::IoOperation("reading file".to_string());
        let _fiber_wait_reason = SuspendReason::WaitingForFiber(FiberId::new(5));
        let _yielded_reason = SuspendReason::Yielded;

        // Test that reasons can be created and used
        let mut fiber = create_test_fiber();
        fiber.suspend(io_reason);
        assert!(fiber.is_suspended());
    }

    #[test]
    fn test_fiber_with_parent() {
        let parent_id = FiberId::new(1);
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber = Fiber::new(FiberId::new(2), future, Some(parent_id));

        assert_eq!(fiber.parent, Some(parent_id));
    }

    #[test]
    fn test_fiber_id_api() {
        let id = FiberId::new(42);
        assert_eq!(id.as_u64(), 42);

        let id2 = FiberId::new(42);
        assert_eq!(id, id2);
    }
}
