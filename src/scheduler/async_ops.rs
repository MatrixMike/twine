//! Async fiber operations and coordination primitives
//!
//! This module provides high-level async operations for fiber management:
//! - Spawning fibers and async tasks
//! - Waiting for fiber completion
//! - Coordination and synchronization primitives
//! - Task handles and completion tracking

use super::types::{FiberId, SuspendReason};
use crate::Result;
use crate::types::Value;
use smol::Timer;
use smol::channel::{Receiver, Sender, bounded};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

use super::scheduler::FiberScheduler;

/// Handle for an async task that can be awaited
#[derive(Debug, Clone)]
pub struct TaskHandle {
    fiber_id: FiberId,
    receiver: Receiver<Result<Value>>,
}

impl TaskHandle {
    /// Create a new task handle
    pub fn new(fiber_id: FiberId, receiver: Receiver<Result<Value>>) -> Self {
        Self { fiber_id, receiver }
    }

    /// Get the fiber ID for this task
    pub fn id(&self) -> FiberId {
        self.fiber_id
    }

    /// Wait for the task to complete and return its result
    pub async fn wait(self) -> Result<Value> {
        match self.receiver.recv().await {
            Ok(result) => result,
            Err(_) => Err(crate::error::Error::runtime_error(
                "Task was cancelled or failed to complete",
            )),
        }
    }
}

/// Future that waits for a specific fiber to complete
#[derive(Debug)]
pub struct FiberWait {
    fiber_id: FiberId,
    scheduler: Arc<Mutex<FiberScheduler>>,
}

impl FiberWait {
    /// Create a new fiber wait future
    pub fn new(fiber_id: FiberId, scheduler: Arc<Mutex<FiberScheduler>>) -> Self {
        Self {
            fiber_id,
            scheduler,
        }
    }
}

impl Future for FiberWait {
    type Output = Result<Value>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let scheduler = self.scheduler.lock().unwrap();

        if let Some(fiber) = scheduler.get_fiber(self.fiber_id) {
            if fiber.is_completed() {
                // For this implementation, we'll return a default value
                // In a real implementation, we'd store the completion value
                Poll::Ready(Ok(Value::boolean(true)))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(Err(crate::error::Error::runtime_error("Fiber not found")))
        }
    }
}

/// Async operations for fiber management
#[derive(Debug)]
pub struct AsyncFiberOps {
    scheduler: Arc<Mutex<FiberScheduler>>,
    completion_senders: Arc<Mutex<HashMap<FiberId, Sender<Result<Value>>>>>,
}

impl AsyncFiberOps {
    /// Create new async fiber operations with the given scheduler
    pub fn new(scheduler: Arc<Mutex<FiberScheduler>>) -> Self {
        Self {
            scheduler,
            completion_senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a new fiber with the given future
    pub fn spawn_fiber<F>(&self, future: F, parent: Option<FiberId>) -> FiberId
    where
        F: Future<Output = Result<Value>> + Send + 'static,
    {
        let future = Box::pin(future);
        let mut scheduler = self.scheduler.lock().unwrap();
        scheduler.spawn_fiber(future, parent)
    }

    /// Spawn a new async task (fiber) and return a handle for waiting
    pub fn spawn_task<F>(&self, future: F, parent: Option<FiberId>) -> TaskHandle
    where
        F: Future<Output = Result<Value>> + Send + 'static,
    {
        let (sender, receiver) = bounded(1);

        let fiber_id = {
            let mut scheduler = self.scheduler.lock().unwrap();
            scheduler.spawn_fiber(Box::pin(future), parent)
        };

        // Store the completion sender
        {
            let mut senders = self.completion_senders.lock().unwrap();
            senders.insert(fiber_id, sender);
        }

        TaskHandle::new(fiber_id, receiver)
    }

    /// Wait for a fiber to complete
    pub async fn wait_for_fiber(&self, fiber_id: FiberId) -> Result<Value> {
        FiberWait::new(fiber_id, Arc::clone(&self.scheduler)).await
    }

    /// Yield the current fiber
    pub fn yield_fiber(&self, fiber_id: FiberId, reason: SuspendReason) -> Result<()> {
        let mut scheduler = self.scheduler.lock().unwrap();
        scheduler.yield_fiber(fiber_id, reason)
    }

    /// Resume a suspended fiber
    pub fn resume_fiber(&self, fiber_id: FiberId) -> Result<()> {
        let mut scheduler = self.scheduler.lock().unwrap();
        scheduler.resume_fiber(fiber_id)
    }

    /// Complete a fiber with the given result
    pub fn complete_fiber(&self, fiber_id: FiberId, result: Result<Value>) -> Result<()> {
        // Send completion notification if there's a waiting task handle
        if let Ok(mut senders) = self.completion_senders.lock() {
            if let Some(sender) = senders.remove(&fiber_id) {
                let _ = sender.try_send(result.clone());
            }
        }

        let mut scheduler = self.scheduler.lock().unwrap();
        scheduler.complete_fiber(fiber_id, result)
    }

    /// Get the current fiber ID
    pub fn current_fiber(&self) -> Option<FiberId> {
        let scheduler = self.scheduler.lock().unwrap();
        scheduler.current_fiber()
    }

    /// Check if a fiber exists
    pub fn has_fiber(&self, fiber_id: FiberId) -> bool {
        let scheduler = self.scheduler.lock().unwrap();
        scheduler.has_fiber(fiber_id)
    }

    /// Get the total number of fibers
    pub fn fiber_count(&self) -> usize {
        let scheduler = self.scheduler.lock().unwrap();
        scheduler.fiber_count()
    }

    /// Check if there are ready fibers
    pub fn has_ready_fibers(&self) -> bool {
        let scheduler = self.scheduler.lock().unwrap();
        scheduler.has_ready_fibers()
    }
}

impl Clone for AsyncFiberOps {
    fn clone(&self) -> Self {
        Self {
            scheduler: Arc::clone(&self.scheduler),
            completion_senders: Arc::clone(&self.completion_senders),
        }
    }
}

impl AsyncFiberOps {
    /// Spawn multiple fibers and wait for all to complete
    pub async fn spawn_all<F, I>(&self, futures: I, parent: Option<FiberId>) -> Result<Vec<Value>>
    where
        F: Future<Output = Result<Value>> + Send + 'static,
        I: IntoIterator<Item = F>,
    {
        let handles: Vec<TaskHandle> = futures
            .into_iter()
            .map(|future| self.spawn_task(future, parent))
            .collect();

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            results.push(handle.wait().await?);
        }
        Ok(results)
    }

    /// Spawn multiple fibers and wait for the first to complete
    pub async fn spawn_race<F, I>(&self, futures: I, parent: Option<FiberId>) -> Result<Value>
    where
        F: Future<Output = Result<Value>> + Send + 'static,
        I: IntoIterator<Item = F>,
    {
        let handles: Vec<TaskHandle> = futures
            .into_iter()
            .map(|future| self.spawn_task(future, parent))
            .collect();

        if handles.is_empty() {
            return Err(crate::error::Error::runtime_error(
                "No futures provided to race",
            ));
        }

        // For simplicity, just wait for the first one
        // In a real implementation, we'd use select! or similar
        handles.into_iter().next().unwrap().wait().await
    }

    /// Create a fiber that yields immediately (for testing)
    pub async fn yield_now(&self) -> Result<Value> {
        if let Some(current) = self.current_fiber() {
            self.yield_fiber(current, SuspendReason::Yielded)?;
        }
        Timer::after(Duration::from_millis(1)).await;
        Ok(Value::boolean(true))
    }
}
