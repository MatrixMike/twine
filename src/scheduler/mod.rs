//! Fiber scheduler infrastructure for lightweight concurrency
//!
//! This module provides the core building blocks for fiber-based concurrency:
//! - Individual fibers with state tracking
//! - Suspension reasons for different blocking conditions
//! - Foundation for the complete scheduler system

use crate::Result;
use crate::types::Value;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

/// Unique identifier for a fiber
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FiberId(pub u64);

/// Unique identifier for a task (higher-level abstraction over fibers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

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
    /// Waiting for a task to complete
    WaitingForTask(TaskId),
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
    pub continuation: Pin<Box<dyn Future<Output = Result<Value>> + Send>>,
    /// Parent fiber that spawned this one (if any)
    pub parent: Option<FiberId>,
    /// Associated task if this fiber was created by a task
    pub associated_task: Option<TaskId>,
    /// Child fibers spawned by this fiber
    pub children: HashSet<FiberId>,
}

impl Fiber {
    /// Create a new fiber with the given id and continuation
    pub fn new(
        id: FiberId,
        continuation: Pin<Box<dyn Future<Output = Result<Value>> + Send>>,
        parent: Option<FiberId>,
    ) -> Self {
        Self {
            id,
            state: FiberState::Ready,
            continuation,
            parent,
            associated_task: None,
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

    /// Check if the fiber is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.state, FiberState::Completed(_))
    }

    /// Set the fiber state to running
    pub fn set_running(&mut self) {
        self.state = FiberState::Running;
    }

    /// Set the fiber state to ready
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

impl std::fmt::Debug for Fiber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fiber")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("parent", &self.parent)
            .field("associated_task", &self.associated_task)
            .field("children", &self.children)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_fiber() -> Fiber {
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        Fiber::new(FiberId(1), future, None)
    }

    #[test]
    fn test_fiber_creation() {
        let fiber = create_test_fiber();
        assert_eq!(fiber.id, FiberId(1));
        assert!(fiber.is_ready());
        assert_eq!(fiber.parent, None);
        assert_eq!(fiber.associated_task, None);
        assert!(fiber.children.is_empty());
    }

    #[test]
    fn test_fiber_state_transitions() {
        let mut fiber = create_test_fiber();

        // Initially ready
        assert!(fiber.is_ready());

        // Set to running
        fiber.set_running();
        assert!(fiber.is_running());

        // Suspend with reason
        fiber.suspend(SuspendReason::Yielded);
        assert!(fiber.is_suspended());

        // Set back to ready
        fiber.set_ready();
        assert!(fiber.is_ready());

        // Complete with result
        fiber.complete(Ok(Value::Number(100.into())));
        assert!(fiber.is_completed());
    }

    #[test]
    fn test_fiber_children_management() {
        let mut fiber = create_test_fiber();
        let child1 = FiberId(2);
        let child2 = FiberId(3);

        // Add children
        fiber.add_child(child1);
        fiber.add_child(child2);
        assert_eq!(fiber.children.len(), 2);
        assert!(fiber.children.contains(&child1));
        assert!(fiber.children.contains(&child2));

        // Remove a child
        fiber.remove_child(child1);
        assert_eq!(fiber.children.len(), 1);
        assert!(!fiber.children.contains(&child1));
        assert!(fiber.children.contains(&child2));
    }

    #[test]
    fn test_suspend_reason_creation() {
        let io_reason = SuspendReason::IoOperation("reading file".to_string());
        let task_reason = SuspendReason::WaitingForTask(TaskId(42));
        let fiber_reason = SuspendReason::WaitingForFiber(FiberId(10));
        let yield_reason = SuspendReason::Yielded;

        // Just test that they can be created and debug printed
        println!("{:?}", io_reason);
        println!("{:?}", task_reason);
        println!("{:?}", fiber_reason);
        println!("{:?}", yield_reason);
    }

    #[test]
    fn test_fiber_with_parent() {
        let parent_id = FiberId(100);
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        let fiber = Fiber::new(FiberId(101), future, Some(parent_id));

        assert_eq!(fiber.parent, Some(parent_id));
        assert_eq!(fiber.id, FiberId(101));
    }
}
