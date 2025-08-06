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
