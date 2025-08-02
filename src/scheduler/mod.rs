//! Fiber scheduler infrastructure for lightweight concurrency
//!
//! This module provides the core building blocks for fiber-based concurrency:
//! - Individual fibers with state tracking
//! - Suspension reasons for different blocking conditions
//! - Foundation for the complete scheduler system

use smol::Executor;

use crate::Result;
use crate::types::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::thread::JoinHandle;

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

/// Unique identifier for a task (higher-level abstraction over fibers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    /// Create a new task ID
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

/// Fiber scheduler managing lightweight execution units
pub struct FiberScheduler {
    /// Queue of fibers ready to execute
    ready_queue: VecDeque<FiberId>,
    /// Map of all fibers by their ID
    fibers: HashMap<FiberId, Fiber>,
    /// Smol executor for async task execution
    runtime: Executor<'static>,
    /// Thread pool for parallel fiber execution
    thread_pool: Vec<JoinHandle<()>>,
    /// Currently executing fiber (if any)
    current_fiber: Option<FiberId>,
    /// Next fiber ID to assign
    next_fiber_id: FiberId,
}

impl FiberScheduler {
    /// Create a new fiber scheduler
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            fibers: HashMap::new(),
            runtime: Executor::new(),
            thread_pool: Vec::new(),
            current_fiber: None,
            next_fiber_id: FiberId::new(1),
        }
    }

    /// Create a new fiber scheduler with specified thread pool size
    pub fn with_threads(thread_count: usize) -> Self {
        let mut scheduler = Self::new();
        scheduler.init_thread_pool(thread_count);
        scheduler
    }

    /// Initialize the thread pool with the specified number of threads
    fn init_thread_pool(&mut self, thread_count: usize) {
        // For now, just reserve capacity - actual thread spawning will be in T4.1.4
        self.thread_pool.reserve(thread_count);
    }

    /// Get the next available fiber ID
    fn next_id(&mut self) -> FiberId {
        let id = self.next_fiber_id;
        self.next_fiber_id = FiberId::new(self.next_fiber_id.as_u64() + 1);
        id
    }

    /// Get the number of fibers in the ready queue
    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }

    /// Get the total number of managed fibers
    pub fn fiber_count(&self) -> usize {
        self.fibers.len()
    }

    /// Get the current executing fiber ID
    pub fn current_fiber(&self) -> Option<FiberId> {
        self.current_fiber
    }

    /// Check if the scheduler has any ready fibers
    pub fn has_ready_fibers(&self) -> bool {
        !self.ready_queue.is_empty()
    }

    /// Check if a specific fiber exists
    pub fn has_fiber(&self, id: FiberId) -> bool {
        self.fibers.contains_key(&id)
    }

    /// Get a reference to a fiber by ID
    pub fn get_fiber(&self, id: FiberId) -> Option<&Fiber> {
        self.fibers.get(&id)
    }

    /// Get a mutable reference to a fiber by ID
    pub fn get_fiber_mut(&mut self, id: FiberId) -> Option<&mut Fiber> {
        self.fibers.get_mut(&id)
    }
}

impl Default for FiberScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for FiberScheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FiberScheduler")
            .field("ready_count", &self.ready_queue.len())
            .field("fiber_count", &self.fibers.len())
            .field("current_fiber", &self.current_fiber)
            .field("thread_pool_size", &self.thread_pool.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_fiber() -> Fiber {
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        Fiber::new(FiberId::new(1), future, None)
    }

    #[test]
    fn test_fiber_creation() {
        let fiber = create_test_fiber();
        assert_eq!(fiber.id, FiberId::new(1));
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
        let child1 = FiberId::new(2);
        let child2 = FiberId::new(3);

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
        let task_reason = SuspendReason::WaitingForTask(TaskId::new(42));
        let fiber_reason = SuspendReason::WaitingForFiber(FiberId::new(10));
        let yield_reason = SuspendReason::Yielded;

        // Just test that they can be created and debug printed
        println!("{:?}", io_reason);
        println!("{:?}", task_reason);
        println!("{:?}", fiber_reason);
        println!("{:?}", yield_reason);
    }

    #[test]
    fn test_fiber_with_parent() {
        let parent_id = FiberId::new(100);
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        let fiber = Fiber::new(FiberId::new(101), future, Some(parent_id));

        assert_eq!(fiber.parent, Some(parent_id));
        assert_eq!(fiber.id, FiberId::new(101));
    }

    #[test]
    fn test_fiber_scheduler_creation() {
        let scheduler = FiberScheduler::new();
        assert_eq!(scheduler.ready_count(), 0);
        assert_eq!(scheduler.fiber_count(), 0);
        assert_eq!(scheduler.current_fiber(), None);
        assert!(!scheduler.has_ready_fibers());
    }

    #[test]
    fn test_fiber_scheduler_with_threads() {
        let scheduler = FiberScheduler::with_threads(4);
        assert_eq!(scheduler.ready_count(), 0);
        assert_eq!(scheduler.fiber_count(), 0);
        assert_eq!(scheduler.thread_pool.capacity(), 4);
    }

    #[test]
    fn test_fiber_scheduler_default() {
        let scheduler = FiberScheduler::default();
        assert_eq!(scheduler.ready_count(), 0);
        assert_eq!(scheduler.fiber_count(), 0);
    }

    #[test]
    fn test_next_fiber_id_generation() {
        let mut scheduler = FiberScheduler::new();
        let id1 = scheduler.next_id();
        let id2 = scheduler.next_id();
        let id3 = scheduler.next_id();

        assert_eq!(id1, FiberId::new(1));
        assert_eq!(id2, FiberId::new(2));
        assert_eq!(id3, FiberId::new(3));
    }

    #[test]
    fn test_fiber_existence_checking() {
        let scheduler = FiberScheduler::new();
        let non_existent_id = FiberId::new(999);

        assert!(!scheduler.has_fiber(non_existent_id));
        assert!(scheduler.get_fiber(non_existent_id).is_none());
    }

    #[test]
    fn test_fiber_scheduler_debug_formatting() {
        let scheduler = FiberScheduler::with_threads(2);
        let debug_str = format!("{:?}", scheduler);

        assert!(debug_str.contains("FiberScheduler"));
        assert!(debug_str.contains("ready_count: 0"));
        assert!(debug_str.contains("fiber_count: 0"));
        assert!(debug_str.contains("current_fiber: None"));
        assert!(debug_str.contains("thread_pool_size: 0"));
    }

    #[test]
    fn test_fiber_id_api() {
        let id = FiberId::new(42);
        assert_eq!(id.as_u64(), 42);

        let id2 = FiberId::new(100);
        assert_eq!(id2.as_u64(), 100);
        assert_ne!(id, id2);
    }

    #[test]
    fn test_task_id_api() {
        let id = TaskId::new(99);
        assert_eq!(id.as_u64(), 99);

        let id2 = TaskId::new(200);
        assert_eq!(id2.as_u64(), 200);
        assert_ne!(id, id2);
    }
}
