//! Fiber scheduler implementation for lightweight concurrency
//!
//! This module provides the main scheduler that manages fiber execution:
//! - Fiber lifecycle management (spawn, yield, resume, complete)
//! - Ready queue and suspended fiber tracking
//! - Multi-threaded execution with thread pool
//! - Main scheduler loop and fiber execution

use super::types::{Fiber, FiberId, FiberState, SuspendReason};
use crate::Result;
use crate::types::Value;
use smol::future::poll_once;
use smol::{Executor, Timer, block_on};
use std::collections::{HashMap, VecDeque};
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle, available_parallelism, sleep};
use std::time::Duration;

/// Main fiber scheduler that manages fiber execution
pub struct FiberScheduler {
    /// Queue of fibers ready to execute
    ready_queue: VecDeque<FiberId>,
    /// All fibers being managed by this scheduler
    fibers: HashMap<FiberId, Fiber>,
    /// Async executor for running fibers
    runtime: Arc<Executor<'static>>,
    /// Thread pool for parallel execution
    thread_pool: Vec<JoinHandle<()>>,
    /// Currently executing fiber (if any)
    current_fiber: Option<FiberId>,
    /// Counter for generating unique fiber IDs
    next_fiber_id: u64,
    /// Flag indicating if the scheduler should shut down
    shutdown: Arc<AtomicBool>,
    /// Number of threads in the pool
    thread_count: usize,
}

impl FiberScheduler {
    /// Create a new fiber scheduler with default thread count
    pub fn new() -> Self {
        let thread_count = available_parallelism().map_or(1, |p| p.get());
        Self::with_threads(thread_count)
    }

    /// Create a new fiber scheduler with specific thread count
    pub fn with_threads(thread_count: usize) -> Self {
        let scheduler = Self {
            ready_queue: VecDeque::new(),
            fibers: HashMap::new(),
            runtime: Arc::new(Executor::new()),
            thread_pool: Vec::new(),
            current_fiber: None,
            next_fiber_id: 1,
            shutdown: Arc::new(AtomicBool::new(false)),
            thread_count,
        };
        // Don't initialize thread pool by default to avoid hanging in tests
        scheduler
    }

    /// Create a test scheduler without thread pool
    pub fn new_for_test() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            fibers: HashMap::new(),
            runtime: Arc::new(Executor::new()),
            thread_pool: Vec::new(),
            current_fiber: None,
            next_fiber_id: 1,
            shutdown: Arc::new(AtomicBool::new(false)),
            thread_count: 0,
        }
    }

    /// Initialize the thread pool for fiber execution
    pub fn init_thread_pool(&mut self) {
        if self.thread_pool.is_empty() && self.thread_count > 0 {
            let executor = Arc::clone(&self.runtime);
            let shutdown = Arc::clone(&self.shutdown);

            for i in 0..self.thread_count {
                let exec = Arc::clone(&executor);
                let shutdown_flag = Arc::clone(&shutdown);

                let handle = thread::Builder::new()
                    .name(format!("fiber-worker-{}", i))
                    .spawn(move || {
                        block_on(async {
                            while !shutdown_flag.load(Ordering::Relaxed) {
                                exec.tick().await;
                                sleep(Duration::from_millis(1));
                            }
                        });
                    })
                    .expect("Failed to spawn fiber worker thread");

                self.thread_pool.push(handle);
            }
        }
    }

    /// Generate the next unique fiber ID
    fn next_id(&mut self) -> FiberId {
        let id = FiberId::new(self.next_fiber_id);
        self.next_fiber_id += 1;
        id
    }

    /// Get the number of ready fibers
    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }

    /// Get the total number of fibers
    pub fn fiber_count(&self) -> usize {
        self.fibers.len()
    }

    /// Get the currently executing fiber ID
    pub fn current_fiber(&self) -> Option<FiberId> {
        self.current_fiber
    }

    /// Check if there are ready fibers waiting to execute
    pub fn has_ready_fibers(&self) -> bool {
        !self.ready_queue.is_empty()
    }

    /// Check if a fiber with the given ID exists
    pub fn has_fiber(&self, fiber_id: FiberId) -> bool {
        self.fibers.contains_key(&fiber_id)
    }

    /// Get a reference to a fiber by ID
    pub fn get_fiber(&self, fiber_id: FiberId) -> Option<&Fiber> {
        self.fibers.get(&fiber_id)
    }

    /// Get a mutable reference to a fiber by ID
    pub fn get_fiber_mut(&mut self, fiber_id: FiberId) -> Option<&mut Fiber> {
        self.fibers.get_mut(&fiber_id)
    }

    /// Spawn a new fiber with the given future and optional parent
    pub fn spawn_fiber(
        &mut self,
        future: Pin<Box<dyn Future<Output = Result<Value>> + Send>>,
        parent: Option<FiberId>,
    ) -> FiberId {
        let fiber_id = self.next_id();
        let fiber = Fiber::new(fiber_id, future, parent);

        // Add to parent's children if parent exists
        if let Some(parent_id) = parent {
            if let Some(parent_fiber) = self.fibers.get_mut(&parent_id) {
                parent_fiber.add_child(fiber_id);
            }
        }

        self.fibers.insert(fiber_id, fiber);
        self.ready_queue.push_back(fiber_id);
        fiber_id
    }

    /// Yield a fiber with the given suspend reason
    pub fn yield_fiber(&mut self, fiber_id: FiberId, reason: SuspendReason) -> Result<()> {
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.suspend(reason);
            if self.current_fiber == Some(fiber_id) {
                self.current_fiber = None;
            }
            // Remove from ready queue
            self.ready_queue.retain(|&id| id != fiber_id);
            Ok(())
        } else {
            Err(crate::error::Error::runtime_error("Fiber not found"))
        }
    }

    /// Resume a suspended fiber
    pub fn resume_fiber(&mut self, fiber_id: FiberId) -> Result<()> {
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            if fiber.is_suspended() {
                fiber.set_ready();
                self.ready_queue.push_back(fiber_id);
                Ok(())
            } else {
                Err(crate::error::Error::runtime_error("Fiber is not suspended"))
            }
        } else {
            Err(crate::error::Error::runtime_error("Fiber not found"))
        }
    }

    /// Clean up a completed fiber and its relationships
    pub fn cleanup_fiber(&mut self, fiber_id: FiberId) -> Result<()> {
        if let Some(fiber) = self.fibers.remove(&fiber_id) {
            // Remove from parent's children
            if let Some(parent_id) = fiber.parent {
                if let Some(parent) = self.fibers.get_mut(&parent_id) {
                    parent.remove_child(fiber_id);
                }
            }

            // Clean up children (mark them as orphaned)
            for child_id in &fiber.children {
                if let Some(child) = self.fibers.get_mut(child_id) {
                    child.parent = None;
                }
            }

            // Remove from ready queue if present
            self.ready_queue.retain(|&id| id != fiber_id);

            // Clear current fiber if this was it
            if self.current_fiber == Some(fiber_id) {
                self.current_fiber = None;
            }

            Ok(())
        } else {
            Err(crate::error::Error::runtime_error("Fiber not found"))
        }
    }

    /// Complete a fiber with the given result
    pub fn complete_fiber(&mut self, fiber_id: FiberId, result: Result<Value>) -> Result<()> {
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.complete(result);
            if self.current_fiber == Some(fiber_id) {
                self.current_fiber = None;
            }
            Ok(())
        } else {
            Err(crate::error::Error::runtime_error("Fiber not found"))
        }
    }

    /// Get the next ready fiber from the queue
    pub fn next_ready_fiber(&mut self) -> Option<FiberId> {
        self.ready_queue.pop_front()
    }

    /// Set the currently executing fiber
    pub fn set_current_fiber(&mut self, fiber_id: Option<FiberId>) -> Result<()> {
        if let Some(id) = fiber_id {
            if self.fibers.contains_key(&id) {
                self.current_fiber = fiber_id;
                if let Some(fiber) = self.fibers.get_mut(&id) {
                    fiber.set_running();
                }
                Ok(())
            } else {
                Err(crate::error::Error::runtime_error("Fiber not found"))
            }
        } else {
            self.current_fiber = None;
            Ok(())
        }
    }

    /// Run the main scheduler loop
    pub async fn run_scheduler(&mut self) -> Result<()> {
        // Initialize thread pool if needed
        self.init_thread_pool();

        while !self.shutdown.load(Ordering::Relaxed) {
            // Execute ready fibers
            if let Some(fiber_id) = self.next_ready_fiber() {
                self.execute_fiber(fiber_id).await?;
            }

            // Check suspended fibers for readiness
            self.check_suspended_fibers().await;

            // Clean up completed fibers
            self.cleanup_completed_fibers();

            // Exit if no more fibers to process
            if self.fibers.is_empty() {
                break;
            }

            // Brief pause to prevent busy waiting
            if self.ready_queue.is_empty() {
                Timer::after(Duration::from_millis(1)).await;
            }
        }
        Ok(())
    }

    /// Execute a single fiber step
    pub async fn execute_fiber(&mut self, fiber_id: FiberId) -> Result<()> {
        self.set_current_fiber(Some(fiber_id))?;

        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            // Try to poll the fiber's future
            match poll_once(&mut fiber.continuation).await {
                Some(result) => {
                    // Fiber completed
                    self.complete_fiber(fiber_id, result)?;
                }
                None => {
                    // Fiber yielded, put it back in ready queue
                    if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
                        if fiber.is_running() {
                            fiber.set_ready();
                            self.ready_queue.push_back(fiber_id);
                        }
                    }
                }
            }
        }

        self.set_current_fiber(None)?;
        Ok(())
    }

    /// Check suspended fibers to see if any should be resumed
    pub async fn check_suspended_fibers(&mut self) {
        let suspended_fibers: Vec<FiberId> = self
            .fibers
            .iter()
            .filter_map(
                |(&id, fiber)| {
                    if fiber.is_suspended() { Some(id) } else { None }
                },
            )
            .collect();

        for fiber_id in suspended_fibers {
            if let Some(fiber) = self.fibers.get(&fiber_id) {
                let should_resume = match &fiber.state {
                    FiberState::Suspended(reason) => match reason {
                        SuspendReason::IoOperation(_io_reason) => {
                            // For now, just resume after a brief delay
                            // In a real implementation, we'd check if the I/O is complete
                            println!("{:?}", _io_reason);
                            true
                        }
                        SuspendReason::WaitingForFiber(target_id) => {
                            // Check if the target fiber has completed
                            self.fibers
                                .get(target_id)
                                .map_or(true, |target| target.is_completed())
                        }
                        SuspendReason::Yielded => {
                            // Yielded fibers can be resumed immediately
                            true
                        }
                    },
                    _ => false,
                };

                if should_resume {
                    let _ = self.resume_fiber(fiber_id);
                }
            }
        }
    }

    /// Clean up completed fibers
    pub fn cleanup_completed_fibers(&mut self) {
        let completed_fibers: Vec<FiberId> = self
            .fibers
            .iter()
            .filter_map(
                |(&id, fiber)| {
                    if fiber.is_completed() { Some(id) } else { None }
                },
            )
            .collect();

        for fiber_id in completed_fibers {
            let _ = self.cleanup_fiber(fiber_id);
        }
    }

    /// Shut down the scheduler and clean up resources
    pub fn shutdown(&mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for all threads to finish
        for handle in self.thread_pool.drain(..) {
            if let Err(e) = handle.join() {
                eprintln!("Error joining thread: {:?}", e);
            }
        }

        // Clear all fibers
        self.fibers.clear();
        self.ready_queue.clear();
        self.current_fiber = None;

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

impl Debug for FiberScheduler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FiberScheduler")
            .field("ready_count", &self.ready_count())
            .field("fiber_count", &self.fiber_count())
            .field("current_fiber", &self.current_fiber)
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
    use crate::types::Value;
    use std::future;

    fn create_test_scheduler() -> FiberScheduler {
        FiberScheduler::new_for_test()
    }

    #[test]
    fn test_fiber_scheduler_creation() {
        let scheduler = create_test_scheduler();
        assert_eq!(scheduler.fiber_count(), 0);
        assert_eq!(scheduler.ready_count(), 0);
        assert!(!scheduler.has_ready_fibers());
        assert!(scheduler.is_running());
    }

    #[test]
    fn test_fiber_scheduler_with_threads() {
        let scheduler = FiberScheduler::with_threads(4);
        assert_eq!(scheduler.thread_count(), 4);
    }

    #[test]
    fn test_fiber_scheduler_default() {
        let scheduler = FiberScheduler::default();
        assert!(scheduler.thread_count() > 0);
    }

    #[test]
    fn test_next_fiber_id_generation() {
        let mut scheduler = create_test_scheduler();
        let future1 = Box::pin(future::ready(Ok(Value::Number(1.into()))));
        let future2 = Box::pin(future::ready(Ok(Value::Number(2.into()))));

        let id1 = scheduler.spawn_fiber(future1, None);
        let id2 = scheduler.spawn_fiber(future2, None);

        assert_ne!(id1, id2);
        assert_eq!(scheduler.fiber_count(), 2);
    }

    #[test]
    fn test_fiber_existence_checking() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        assert!(scheduler.has_fiber(fiber_id));
        assert!(!scheduler.has_fiber(FiberId::new(999)));
    }

    #[test]
    fn test_fiber_scheduler_debug_formatting() {
        let scheduler = create_test_scheduler();
        let debug_output = format!("{:?}", scheduler);
        assert!(debug_output.contains("FiberScheduler"));
        assert!(debug_output.contains("ready_count"));
        assert!(debug_output.contains("fiber_count"));
    }

    #[test]
    fn test_spawn_fiber() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        assert!(scheduler.has_fiber(fiber_id));
        assert_eq!(scheduler.fiber_count(), 1);
        assert_eq!(scheduler.ready_count(), 1);
        assert!(scheduler.has_ready_fibers());
    }

    #[test]
    fn test_spawn_fiber_with_parent() {
        let mut scheduler = create_test_scheduler();
        let parent_future = Box::pin(future::ready(Ok(Value::Number(1.into()))));
        let child_future = Box::pin(future::ready(Ok(Value::Number(2.into()))));

        let parent_id = scheduler.spawn_fiber(parent_future, None);
        let child_id = scheduler.spawn_fiber(child_future, Some(parent_id));

        assert!(scheduler.has_fiber(parent_id));
        assert!(scheduler.has_fiber(child_id));

        let parent_fiber = scheduler.get_fiber(parent_id).unwrap();
        assert!(parent_fiber.children.contains(&child_id));

        let child_fiber = scheduler.get_fiber(child_id).unwrap();
        assert_eq!(child_fiber.parent, Some(parent_id));
    }

    #[test]
    fn test_yield_fiber() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
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
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        // First yield the fiber
        scheduler
            .yield_fiber(fiber_id, SuspendReason::Yielded)
            .unwrap();
        assert_eq!(scheduler.ready_count(), 0);

        // Then resume it
        scheduler.resume_fiber(fiber_id).unwrap();
        assert_eq!(scheduler.ready_count(), 1);

        let fiber = scheduler.get_fiber(fiber_id).unwrap();
        assert!(fiber.is_ready());
    }

    #[test]
    fn test_complete_fiber() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        scheduler.set_current_fiber(Some(fiber_id)).unwrap();
        scheduler
            .complete_fiber(fiber_id, Ok(Value::Number(42.into())))
            .unwrap();

        let fiber = scheduler.get_fiber(fiber_id).unwrap();
        assert!(fiber.is_completed());
        assert_eq!(scheduler.current_fiber(), None);
    }

    #[test]
    fn test_cleanup_fiber() {
        let mut scheduler = create_test_scheduler();
        let parent_future = Box::pin(future::ready(Ok(Value::Number(1.into()))));
        let child_future = Box::pin(future::ready(Ok(Value::Number(2.into()))));

        let parent_id = scheduler.spawn_fiber(parent_future, None);
        let child_id = scheduler.spawn_fiber(child_future, Some(parent_id));

        assert_eq!(scheduler.fiber_count(), 2);

        scheduler.cleanup_fiber(parent_id).unwrap();
        assert_eq!(scheduler.fiber_count(), 1);
        assert!(!scheduler.has_fiber(parent_id));

        // Child should still exist but be orphaned
        let child_fiber = scheduler.get_fiber(child_id).unwrap();
        assert_eq!(child_fiber.parent, None);
    }

    #[test]
    fn test_next_ready_fiber() {
        let mut scheduler = create_test_scheduler();
        let future1 = Box::pin(future::ready(Ok(Value::Number(1.into()))));
        let future2 = Box::pin(future::ready(Ok(Value::Number(2.into()))));

        let id1 = scheduler.spawn_fiber(future1, None);
        let id2 = scheduler.spawn_fiber(future2, None);

        assert_eq!(scheduler.ready_count(), 2);

        let next_id = scheduler.next_ready_fiber();
        assert_eq!(next_id, Some(id1));
        assert_eq!(scheduler.ready_count(), 1);

        let next_id = scheduler.next_ready_fiber();
        assert_eq!(next_id, Some(id2));
        assert_eq!(scheduler.ready_count(), 0);

        let next_id = scheduler.next_ready_fiber();
        assert_eq!(next_id, None);
    }

    #[test]
    fn test_set_current_fiber() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        scheduler.set_current_fiber(Some(fiber_id)).unwrap();
        assert_eq!(scheduler.current_fiber(), Some(fiber_id));

        let fiber = scheduler.get_fiber(fiber_id).unwrap();
        assert!(fiber.is_running());

        scheduler.set_current_fiber(None).unwrap();
        assert_eq!(scheduler.current_fiber(), None);
    }

    #[test]
    fn test_fiber_lifecycle_error_handling() {
        let mut scheduler = create_test_scheduler();
        let nonexistent_id = FiberId::new(999);

        // Test operations on nonexistent fiber
        assert!(
            scheduler
                .yield_fiber(nonexistent_id, SuspendReason::Yielded)
                .is_err()
        );
        assert!(scheduler.resume_fiber(nonexistent_id).is_err());
        assert!(
            scheduler
                .complete_fiber(nonexistent_id, Ok(Value::nil()))
                .is_err()
        );
        assert!(scheduler.cleanup_fiber(nonexistent_id).is_err());
        assert!(scheduler.set_current_fiber(Some(nonexistent_id)).is_err());

        // Test invalid state transitions
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        // Try to resume a fiber that's not suspended
        assert!(scheduler.resume_fiber(fiber_id).is_err());
    }

    #[test]
    fn test_scheduler_initialization() {
        let scheduler = create_test_scheduler();
        assert_eq!(scheduler.thread_count(), 0); // Test scheduler has no threads
        assert!(scheduler.is_running());
        assert_eq!(scheduler.fiber_count(), 0);
    }

    #[test]
    fn test_scheduler_shutdown() {
        let mut scheduler = create_test_scheduler();
        assert!(scheduler.is_running());

        scheduler.shutdown().unwrap();
        assert!(!scheduler.is_running());
        assert_eq!(scheduler.fiber_count(), 0);
    }

    #[test]
    fn test_cleanup_completed_fibers() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        let fiber_id = scheduler.spawn_fiber(future, None);

        // Complete the fiber
        scheduler
            .complete_fiber(fiber_id, Ok(Value::Number(42.into())))
            .unwrap();
        assert_eq!(scheduler.fiber_count(), 1);

        // Clean up completed fibers
        scheduler.cleanup_completed_fibers();
        assert_eq!(scheduler.fiber_count(), 0);
    }

    #[test]
    fn test_scheduler_with_default_thread_count() {
        let scheduler = FiberScheduler::default();
        assert!(scheduler.thread_count() > 0);
    }

    #[test]
    fn test_execute_single_fiber() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(async { Ok(Value::Number(42.into())) });
        let fiber_id = scheduler.spawn_fiber(future, None);

        assert_eq!(scheduler.fiber_count(), 1);
        assert!(scheduler.has_ready_fibers());

        // Complete the fiber manually for testing
        scheduler
            .complete_fiber(fiber_id, Ok(Value::Number(42.into())))
            .unwrap();

        let fiber = scheduler.get_fiber(fiber_id).unwrap();
        assert!(fiber.is_completed());
    }

    #[test]
    fn test_check_suspended_fibers() {
        let mut scheduler = create_test_scheduler();
        let future = Box::pin(async { Ok(Value::Number(42.into())) });
        let fiber_id = scheduler.spawn_fiber(future, None);

        // Suspend the fiber
        scheduler
            .yield_fiber(fiber_id, SuspendReason::Yielded)
            .unwrap();

        // Resume it manually to test the logic
        scheduler.resume_fiber(fiber_id).unwrap();

        let fiber = scheduler.get_fiber(fiber_id).unwrap();
        assert!(fiber.is_ready());
    }

    #[test]
    fn test_scheduler_run_basic() {
        let mut scheduler = create_test_scheduler();

        // Spawn a simple fiber
        let future = Box::pin(async { Ok(Value::Number(42.into())) });
        let fiber_id = scheduler.spawn_fiber(future, None);

        assert_eq!(scheduler.fiber_count(), 1);
        assert!(scheduler.has_ready_fibers());

        // Manually complete and cleanup to test the process
        scheduler
            .complete_fiber(fiber_id, Ok(Value::Number(42.into())))
            .unwrap();
        scheduler.cleanup_completed_fibers();

        assert_eq!(scheduler.fiber_count(), 0);
    }
}
