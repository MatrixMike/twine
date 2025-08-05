//! Fiber scheduler infrastructure for lightweight concurrency
//!
//! This module provides the core building blocks for fiber-based concurrency:
//! - Individual fibers with state tracking
//! - Suspension reasons for different blocking conditions
//! - Foundation for the complete scheduler system

use crate::Result;
use crate::types::Value;
use core::fmt;
use smol::future::poll_once;
use smol::{Executor, Timer, block_on};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle, available_parallelism, sleep};
use std::time::Duration;

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
    pub continuation: Pin<Box<dyn Future<Output = Result<Value>> + Send>>,
    /// Parent fiber that spawned this one (if any)
    pub parent: Option<FiberId>,

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

/// Fiber scheduler managing lightweight execution units
pub struct FiberScheduler {
    /// Queue of fibers ready to execute
    ready_queue: VecDeque<FiberId>,
    /// Map of all fibers by their ID
    fibers: HashMap<FiberId, Fiber>,
    /// Smol executor for async task execution
    runtime: Arc<Executor<'static>>,
    /// Thread pool for parallel fiber execution
    thread_pool: Vec<JoinHandle<()>>,
    /// Currently executing fiber (if any)
    current_fiber: Option<FiberId>,
    /// Next fiber ID to assign
    next_fiber_id: FiberId,
    /// Shutdown signal for the scheduler
    shutdown: Arc<AtomicBool>,
    /// Number of threads in the pool
    thread_count: usize,
}

impl FiberScheduler {
    /// Create a new fiber scheduler
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            fibers: HashMap::new(),
            runtime: Arc::new(Executor::new()),
            thread_pool: Vec::new(),
            current_fiber: None,
            next_fiber_id: FiberId::new(1),
            shutdown: Arc::new(AtomicBool::new(false)),
            thread_count: 0,
        }
    }

    /// Create a new fiber scheduler with specified thread pool size
    pub fn with_threads(thread_count: usize) -> Self {
        let mut scheduler = Self::new();
        scheduler.thread_count = thread_count;
        scheduler
    }

    /// Initialize the thread pool with the specified number of threads
    fn init_thread_pool(&mut self) {
        if self.thread_count == 0 {
            self.thread_count = available_parallelism().map(|n| n.get()).unwrap_or(4);
        }

        self.thread_pool.reserve(self.thread_count);

        for i in 0..self.thread_count {
            let executor = Arc::clone(&self.runtime);
            let shutdown = Arc::clone(&self.shutdown);

            let handle = thread::Builder::new()
                .name(format!("fiber-worker-{i}"))
                .spawn(move || {
                    // Run the executor until shutdown
                    while !shutdown.load(Ordering::Relaxed) {
                        block_on(executor.tick());
                        sleep(Duration::from_micros(1));
                    }
                })
                .expect("Failed to spawn worker thread");

            self.thread_pool.push(handle);
        }
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

    /// Spawn a new fiber with the given continuation
    pub fn spawn_fiber(
        &mut self,
        continuation: Pin<Box<dyn Future<Output = Result<Value>> + Send>>,
        parent: Option<FiberId>,
    ) -> FiberId {
        let id = self.next_id();
        let fiber = Fiber::new(id, continuation, parent);

        // Add to parent's children if there is a parent
        if let Some(parent_id) = parent {
            if let Some(parent_fiber) = self.get_fiber_mut(parent_id) {
                parent_fiber.add_child(id);
            }
        }

        // Add fiber to scheduler and ready queue
        self.fibers.insert(id, fiber);
        self.ready_queue.push_back(id);

        id
    }

    /// Yield the current fiber, suspending it with the given reason
    pub fn yield_fiber(&mut self, fiber_id: FiberId, reason: SuspendReason) -> Result<()> {
        if let Some(fiber) = self.get_fiber_mut(fiber_id) {
            fiber.suspend(reason);

            // Remove from current fiber if it was the current one
            if self.current_fiber == Some(fiber_id) {
                self.current_fiber = None;
            }

            // Remove from ready queue if present
            self.ready_queue.retain(|&id| id != fiber_id);

            Ok(())
        } else {
            Err(crate::Error::runtime_error(&format!(
                "Fiber {fiber_id:?} not found"
            )))
        }
    }

    /// Resume a suspended fiber by moving it to the ready queue
    pub fn resume_fiber(&mut self, fiber_id: FiberId) -> Result<()> {
        if let Some(fiber) = self.get_fiber_mut(fiber_id) {
            if fiber.is_suspended() {
                fiber.set_ready();
                self.ready_queue.push_back(fiber_id);
                Ok(())
            } else {
                Err(crate::Error::runtime_error(&format!(
                    "Fiber {:?} is not suspended (current state: {:?})",
                    fiber_id, fiber.state
                )))
            }
        } else {
            Err(crate::Error::runtime_error(&format!(
                "Fiber {fiber_id:?} not found"
            )))
        }
    }

    /// Remove a completed fiber and clean up its resources
    pub fn cleanup_fiber(&mut self, fiber_id: FiberId) -> Result<()> {
        if let Some(fiber) = self.fibers.remove(&fiber_id) {
            // Remove from parent's children
            if let Some(parent_id) = fiber.parent {
                if let Some(parent_fiber) = self.get_fiber_mut(parent_id) {
                    parent_fiber.remove_child(fiber_id);
                }
            }

            // Clean up all child fibers recursively
            let children: Vec<FiberId> = fiber.children.iter().copied().collect();
            for child_id in children {
                let _ = self.cleanup_fiber(child_id); // Ignore errors for cleanup
            }

            // Remove from current fiber if it was the current one
            if self.current_fiber == Some(fiber_id) {
                self.current_fiber = None;
            }

            // Remove from ready queue if present
            self.ready_queue.retain(|&id| id != fiber_id);

            Ok(())
        } else {
            Err(crate::Error::runtime_error(&format!(
                "Fiber {fiber_id:?} not found"
            )))
        }
    }

    /// Mark a fiber as completed and schedule it for cleanup
    pub fn complete_fiber(&mut self, fiber_id: FiberId, result: Result<Value>) -> Result<()> {
        if let Some(fiber) = self.get_fiber_mut(fiber_id) {
            fiber.complete(result);

            // Remove from current fiber if it was the current one
            if self.current_fiber == Some(fiber_id) {
                self.current_fiber = None;
            }

            Ok(())
        } else {
            Err(crate::Error::runtime_error(&format!(
                "Fiber {fiber_id:?} not found"
            )))
        }
    }

    /// Get the next ready fiber from the queue
    pub fn next_ready_fiber(&mut self) -> Option<FiberId> {
        self.ready_queue.pop_front()
    }

    /// Set the current executing fiber
    pub fn set_current_fiber(&mut self, fiber_id: Option<FiberId>) -> Result<()> {
        if let Some(id) = fiber_id {
            if let Some(fiber) = self.get_fiber_mut(id) {
                fiber.set_running();
                self.current_fiber = Some(id);
                Ok(())
            } else {
                Err(crate::Error::runtime_error(&format!(
                    "Fiber {id:?} not found"
                )))
            }
        } else {
            self.current_fiber = None;
            Ok(())
        }
    }

    /// Run the main scheduler loop
    pub async fn run_scheduler(&mut self) -> Result<()> {
        // Initialize thread pool if not already done
        if self.thread_pool.is_empty() && self.thread_count > 0 {
            self.init_thread_pool();
        }

        // Main scheduler event loop
        while !self.shutdown.load(Ordering::Relaxed) {
            // Check if there are any ready fibers
            if let Some(fiber_id) = self.next_ready_fiber() {
                // Execute the fiber
                self.execute_fiber(fiber_id).await?;
            } else {
                // No ready fibers, check for suspended fibers that might be ready
                self.check_suspended_fibers().await;

                // If still no work, sleep briefly to avoid busy waiting
                if self.ready_queue.is_empty() {
                    Timer::after(Duration::from_micros(1)).await;
                }
            }

            // Clean up completed fibers
            self.cleanup_completed_fibers();

            // Check if we should shutdown (no more fibers to process)
            if self.fibers.is_empty() {
                break;
            }
        }

        Ok(())
    }

    /// Execute a single fiber
    async fn execute_fiber(&mut self, fiber_id: FiberId) -> Result<()> {
        // Set fiber as current and running
        self.set_current_fiber(Some(fiber_id))?;

        // Get the fiber (we need to temporarily remove it to avoid borrow issues)
        if let Some(mut fiber) = self.fibers.remove(&fiber_id) {
            // Poll the fiber's continuation
            let result = poll_once(&mut fiber.continuation).await;

            match result {
                Some(Ok(value)) => {
                    // Fiber completed successfully
                    fiber.complete(Ok(value));
                    self.fibers.insert(fiber_id, fiber);
                    self.set_current_fiber(None)?;
                }
                Some(Err(error)) => {
                    // Fiber completed with error
                    fiber.complete(Err(error));
                    self.fibers.insert(fiber_id, fiber);
                    self.set_current_fiber(None)?;
                }
                None => {
                    // Fiber needs more time, suspend it
                    fiber.suspend(SuspendReason::Yielded);
                    self.fibers.insert(fiber_id, fiber);
                    self.set_current_fiber(None)?;
                }
            }
        }

        Ok(())
    }

    /// Check suspended fibers to see if any can be resumed
    async fn check_suspended_fibers(&mut self) {
        let suspended_ids: Vec<FiberId> = self
            .fibers
            .iter()
            .filter_map(|(id, fiber)| {
                if fiber.is_suspended() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        for fiber_id in suspended_ids {
            // For now, just resume yielded fibers immediately
            // In future tasks, this will check I/O completion, task completion, etc.
            if let Some(fiber) = self.get_fiber(fiber_id) {
                if let FiberState::Suspended(SuspendReason::Yielded) = &fiber.state {
                    let _ = self.resume_fiber(fiber_id);
                }
            }
        }
    }

    /// Clean up completed fibers
    fn cleanup_completed_fibers(&mut self) {
        let completed_ids: Vec<FiberId> = self
            .fibers
            .iter()
            .filter_map(|(id, fiber)| {
                if fiber.is_completed() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        for fiber_id in completed_ids {
            let _ = self.cleanup_fiber(fiber_id);
        }
    }

    /// Shutdown the scheduler and wait for all threads to finish
    pub fn shutdown(&mut self) -> Result<()> {
        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for all worker threads to finish
        while let Some(handle) = self.thread_pool.pop() {
            if let Err(e) = handle.join() {
                eprintln!("Error joining worker thread: {e:?}");
            }
        }

        Ok(())
    }

    /// Check if the scheduler is running
    pub fn is_running(&self) -> bool {
        !self.shutdown.load(Ordering::Relaxed)
    }

    /// Get the number of threads in the pool
    pub fn thread_count(&self) -> usize {
        self.thread_count
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
            .field("thread_count", &self.thread_count)
            .field("is_running", &self.is_running())
            .finish()
    }
}

impl Drop for FiberScheduler {
    fn drop(&mut self) {
        let _ = self.shutdown();
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
        let fiber_reason = SuspendReason::WaitingForFiber(FiberId::new(10));
        let yield_reason = SuspendReason::Yielded;

        // Just test that they can be created and debug printed
        println!("{:?}", io_reason);
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
        assert_eq!(scheduler.thread_count(), 4);
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
    fn test_spawn_fiber() {
        let mut scheduler = FiberScheduler::new();
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));

        let fiber_id = scheduler.spawn_fiber(future, None);

        assert_eq!(scheduler.fiber_count(), 1);
        assert_eq!(scheduler.ready_count(), 1);
        assert!(scheduler.has_fiber(fiber_id));
        assert!(scheduler.get_fiber(fiber_id).unwrap().is_ready());
    }

    #[test]
    fn test_spawn_fiber_with_parent() {
        let mut scheduler = FiberScheduler::new();
        let parent_future = Box::pin(std::future::ready(Ok(Value::Number(1.into()))));
        let child_future = Box::pin(std::future::ready(Ok(Value::Number(2.into()))));

        let parent_id = scheduler.spawn_fiber(parent_future, None);
        let child_id = scheduler.spawn_fiber(child_future, Some(parent_id));

        assert_eq!(scheduler.fiber_count(), 2);
        assert_eq!(scheduler.ready_count(), 2);

        let parent_fiber = scheduler.get_fiber(parent_id).unwrap();
        assert!(parent_fiber.children.contains(&child_id));

        let child_fiber = scheduler.get_fiber(child_id).unwrap();
        assert_eq!(child_fiber.parent, Some(parent_id));
    }

    #[test]
    fn test_yield_fiber() {
        let mut scheduler = FiberScheduler::new();
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        scheduler.set_current_fiber(Some(fiber_id)).unwrap();
        assert_eq!(scheduler.current_fiber(), Some(fiber_id));

        scheduler
            .yield_fiber(fiber_id, SuspendReason::Yielded)
            .unwrap();

        let fiber = scheduler.get_fiber(fiber_id).unwrap();
        assert!(fiber.is_suspended());
        assert_eq!(scheduler.current_fiber(), None);
    }

    #[test]
    fn test_resume_fiber() {
        let mut scheduler = FiberScheduler::new();
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        // Suspend the fiber first
        scheduler
            .yield_fiber(fiber_id, SuspendReason::Yielded)
            .unwrap();
        assert!(scheduler.get_fiber(fiber_id).unwrap().is_suspended());
        assert_eq!(scheduler.ready_count(), 0);

        // Resume the fiber
        scheduler.resume_fiber(fiber_id).unwrap();
        assert!(scheduler.get_fiber(fiber_id).unwrap().is_ready());
        assert_eq!(scheduler.ready_count(), 1);
    }

    #[test]
    fn test_complete_fiber() {
        let mut scheduler = FiberScheduler::new();
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        scheduler.set_current_fiber(Some(fiber_id)).unwrap();

        let result = Ok(Value::Number(100.into()));
        scheduler.complete_fiber(fiber_id, result).unwrap();

        let fiber = scheduler.get_fiber(fiber_id).unwrap();
        assert!(fiber.is_completed());
        assert_eq!(scheduler.current_fiber(), None);
    }

    #[test]
    fn test_cleanup_fiber() {
        let mut scheduler = FiberScheduler::new();
        let parent_future = Box::pin(std::future::ready(Ok(Value::Number(1.into()))));
        let child_future = Box::pin(std::future::ready(Ok(Value::Number(2.into()))));

        let parent_id = scheduler.spawn_fiber(parent_future, None);
        let child_id = scheduler.spawn_fiber(child_future, Some(parent_id));

        assert_eq!(scheduler.fiber_count(), 2);

        // Cleanup parent should also cleanup child
        scheduler.cleanup_fiber(parent_id).unwrap();

        assert_eq!(scheduler.fiber_count(), 0);
        assert!(!scheduler.has_fiber(parent_id));
        assert!(!scheduler.has_fiber(child_id));
    }

    #[test]
    fn test_next_ready_fiber() {
        let mut scheduler = FiberScheduler::new();
        let future1 = Box::pin(std::future::ready(Ok(Value::Number(1.into()))));
        let future2 = Box::pin(std::future::ready(Ok(Value::Number(2.into()))));

        let fiber_id1 = scheduler.spawn_fiber(future1, None);
        let fiber_id2 = scheduler.spawn_fiber(future2, None);

        assert_eq!(scheduler.ready_count(), 2);

        let next_fiber = scheduler.next_ready_fiber();
        assert_eq!(next_fiber, Some(fiber_id1));
        assert_eq!(scheduler.ready_count(), 1);

        let next_fiber = scheduler.next_ready_fiber();
        assert_eq!(next_fiber, Some(fiber_id2));
        assert_eq!(scheduler.ready_count(), 0);

        let next_fiber = scheduler.next_ready_fiber();
        assert_eq!(next_fiber, None);
    }

    #[test]
    fn test_set_current_fiber() {
        let mut scheduler = FiberScheduler::new();
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        assert_eq!(scheduler.current_fiber(), None);

        scheduler.set_current_fiber(Some(fiber_id)).unwrap();
        assert_eq!(scheduler.current_fiber(), Some(fiber_id));
        assert!(scheduler.get_fiber(fiber_id).unwrap().is_running());

        scheduler.set_current_fiber(None).unwrap();
        assert_eq!(scheduler.current_fiber(), None);
    }

    #[test]
    fn test_fiber_lifecycle_error_handling() {
        let mut scheduler = FiberScheduler::new();
        let non_existent_id = FiberId::new(999);

        // Test yield_fiber with non-existent fiber
        let result = scheduler.yield_fiber(non_existent_id, SuspendReason::Yielded);
        assert!(result.is_err());

        // Test resume_fiber with non-existent fiber
        let result = scheduler.resume_fiber(non_existent_id);
        assert!(result.is_err());

        // Test complete_fiber with non-existent fiber
        let result = scheduler.complete_fiber(non_existent_id, Ok(Value::Number(1.into())));
        assert!(result.is_err());

        // Test cleanup_fiber with non-existent fiber
        let result = scheduler.cleanup_fiber(non_existent_id);
        assert!(result.is_err());

        // Test set_current_fiber with non-existent fiber
        let result = scheduler.set_current_fiber(Some(non_existent_id));
        assert!(result.is_err());
    }

    #[test]
    fn test_scheduler_initialization() {
        let scheduler = FiberScheduler::with_threads(2);
        assert_eq!(scheduler.thread_count(), 2);
        assert!(scheduler.is_running());
        assert_eq!(scheduler.thread_pool.len(), 0); // Not initialized yet
    }

    #[test]
    fn test_scheduler_shutdown() {
        let mut scheduler = FiberScheduler::with_threads(2);
        assert!(scheduler.is_running());

        scheduler.shutdown().unwrap();
        assert!(!scheduler.is_running());
    }

    #[test]
    fn test_cleanup_completed_fibers() {
        let mut scheduler = FiberScheduler::new();
        let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        // Complete the fiber
        scheduler
            .complete_fiber(fiber_id, Ok(Value::Number(100.into())))
            .unwrap();
        assert_eq!(scheduler.fiber_count(), 1);

        // Cleanup should remove completed fibers
        scheduler.cleanup_completed_fibers();
        assert_eq!(scheduler.fiber_count(), 0);
    }

    #[test]
    fn test_scheduler_with_default_thread_count() {
        let scheduler = FiberScheduler::new();
        assert_eq!(scheduler.thread_count(), 0); // Default is 0 until init
    }

    #[test]
    fn test_execute_single_fiber() {
        smol::block_on(async {
            let mut scheduler = FiberScheduler::new();
            let future = Box::pin(async { Ok(Value::Number(42.into())) });
            let fiber_id = scheduler.spawn_fiber(future, None);

            assert_eq!(scheduler.ready_count(), 1);

            // Execute the fiber
            scheduler.execute_fiber(fiber_id).await.unwrap();

            // Fiber should be completed
            let fiber = scheduler.get_fiber(fiber_id).unwrap();
            assert!(fiber.is_completed());
        });
    }

    #[test]
    fn test_check_suspended_fibers() {
        smol::block_on(async {
            let mut scheduler = FiberScheduler::new();
            let future = Box::pin(std::future::ready(Ok(Value::Number(42.into()))));
            let fiber_id = scheduler.spawn_fiber(future, None);

            // Suspend the fiber
            scheduler
                .yield_fiber(fiber_id, SuspendReason::Yielded)
                .unwrap();
            assert_eq!(scheduler.ready_count(), 0);

            // Check suspended fibers should resume yielded ones
            scheduler.check_suspended_fibers().await;
            assert_eq!(scheduler.ready_count(), 1);
        });
    }

    #[test]
    fn test_scheduler_run_basic() {
        smol::block_on(async {
            let mut scheduler = FiberScheduler::new();

            // Add a simple fiber that completes immediately
            let future = Box::pin(async { Ok(Value::Number(42.into())) });
            scheduler.spawn_fiber(future, None);

            assert_eq!(scheduler.fiber_count(), 1);
            assert_eq!(scheduler.ready_count(), 1);

            // Run scheduler should complete when no more fibers
            scheduler.run_scheduler().await.unwrap();

            // All fibers should be cleaned up
            assert_eq!(scheduler.fiber_count(), 0);
            assert_eq!(scheduler.ready_count(), 0);
        });
    }
}
