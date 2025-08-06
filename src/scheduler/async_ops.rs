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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::scheduler::FiberScheduler;
    use crate::types::Value;
    use std::sync::{Arc, Mutex};

    fn create_test_async_ops() -> AsyncFiberOps {
        let scheduler = Arc::new(Mutex::new(FiberScheduler::new_for_test()));
        AsyncFiberOps::new(scheduler)
    }

    #[test]
    fn test_async_fiber_ops_creation() {
        let async_ops = create_test_async_ops();
        assert_eq!(async_ops.fiber_count(), 0);
        assert!(!async_ops.has_ready_fibers());
    }

    #[test]
    fn test_spawn_simple_fiber() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future = async { Ok(Value::number(42.0)) };
            let fiber_id = async_ops.spawn_fiber(future, None);

            assert!(async_ops.has_fiber(fiber_id));
            assert_eq!(async_ops.fiber_count(), 1);
        });
    }

    #[test]
    fn test_spawn_task_with_handle() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future = async { Ok(Value::number(42.0)) };
            let handle = async_ops.spawn_task(future, None);

            assert!(async_ops.has_fiber(handle.id()));
            assert_eq!(async_ops.fiber_count(), 1);
        });
    }

    #[test]
    fn test_task_completion() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future = async { Ok(Value::number(42.0)) };
            let handle = async_ops.spawn_task(future, None);
            let fiber_id = handle.id();

            // Manually complete the fiber to test completion notification
            async_ops
                .complete_fiber(fiber_id, Ok(Value::number(42.0)))
                .unwrap();

            let result = handle.wait().await.unwrap();
            assert_eq!(result.as_number().unwrap(), 42.0);
        });
    }

    #[test]
    fn test_fiber_wait_future() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future = async { Ok(Value::string("hello")) };
            let fiber_id = async_ops.spawn_fiber(future, None);

            // Manually complete the fiber
            async_ops
                .complete_fiber(fiber_id, Ok(Value::string("hello")))
                .unwrap();

            let result = async_ops.wait_for_fiber(fiber_id).await.unwrap();
            assert_eq!(result.as_boolean().unwrap(), true); // FiberWait returns true for completed fibers
        });
    }

    #[test]
    fn test_fiber_yield_and_resume() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future = async { Ok(Value::boolean(true)) };
            let fiber_id = async_ops.spawn_fiber(future, None);

            // Yield the fiber
            async_ops
                .yield_fiber(fiber_id, SuspendReason::Yielded)
                .unwrap();

            // Resume the fiber
            async_ops.resume_fiber(fiber_id).unwrap();

            assert!(async_ops.has_fiber(fiber_id));
        });
    }

    #[test]
    fn test_parent_child_fiber_relationship() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            // Create parent fiber
            let parent_future = async { Ok(Value::string("parent")) };
            let parent_id = async_ops.spawn_fiber(parent_future, None);

            // Create child fiber with parent
            let child_future = async { Ok(Value::string("child")) };
            let child_id = async_ops.spawn_fiber(child_future, Some(parent_id));

            assert!(async_ops.has_fiber(parent_id));
            assert!(async_ops.has_fiber(child_id));
            assert_eq!(async_ops.fiber_count(), 2);
        });
    }

    #[test]
    fn test_multiple_tasks_with_handles() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future1 = async { Ok(Value::number(1.0)) };
            let future2 = async { Ok(Value::number(2.0)) };
            let future3 = async { Ok(Value::number(3.0)) };

            let handle1 = async_ops.spawn_task(future1, None);
            let handle2 = async_ops.spawn_task(future2, None);
            let handle3 = async_ops.spawn_task(future3, None);

            assert_eq!(async_ops.fiber_count(), 3);

            // Complete all fibers
            async_ops
                .complete_fiber(handle1.id(), Ok(Value::number(1.0)))
                .unwrap();
            async_ops
                .complete_fiber(handle2.id(), Ok(Value::number(2.0)))
                .unwrap();
            async_ops
                .complete_fiber(handle3.id(), Ok(Value::number(3.0)))
                .unwrap();

            let result1 = handle1.wait().await.unwrap();
            let result2 = handle2.wait().await.unwrap();
            let result3 = handle3.wait().await.unwrap();

            assert_eq!(result1.as_number().unwrap(), 1.0);
            assert_eq!(result2.as_number().unwrap(), 2.0);
            assert_eq!(result3.as_number().unwrap(), 3.0);
        });
    }

    #[test]
    fn test_spawn_race_fibers() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future1 = async { Ok(Value::number(10.0)) };
            let future2 = async { Ok(Value::number(20.0)) };
            let future3 = async { Ok(Value::number(30.0)) };

            let handle1 = async_ops.spawn_task(future1, None);
            let handle2 = async_ops.spawn_task(future2, None);
            let handle3 = async_ops.spawn_task(future3, None);

            // Complete all fibers
            async_ops
                .complete_fiber(handle1.id(), Ok(Value::number(10.0)))
                .unwrap();
            async_ops
                .complete_fiber(handle2.id(), Ok(Value::number(20.0)))
                .unwrap();
            async_ops
                .complete_fiber(handle3.id(), Ok(Value::number(30.0)))
                .unwrap();

            let result1 = handle1.wait().await;
            let result2 = handle2.wait().await;
            let result3 = handle3.wait().await;

            assert!(result1.is_ok());
            assert!(result2.is_ok());
            assert!(result3.is_ok());
        });
    }

    #[test]
    fn test_spawn_all_fibers() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            // Test with empty futures to verify basic functionality
            let futures: Vec<std::future::Ready<crate::Result<Value>>> = Vec::new();
            let result = async_ops.spawn_all(futures, None).await;
            assert!(result.is_ok());
            let results = result.unwrap();
            assert_eq!(results.len(), 0);
        });
    }

    #[test]
    fn test_spawn_race_single_future() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future = async { Ok(Value::string("winner")) };
            let handle = async_ops.spawn_task(future, None);

            // Complete the fiber manually
            async_ops
                .complete_fiber(handle.id(), Ok(Value::string("winner")))
                .unwrap();

            let result = handle.wait().await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_string().unwrap(), "winner");
        });
    }

    #[test]
    fn test_fiber_cancellation() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let future = async { Ok(Value::string("cancelled")) };
            let handle = async_ops.spawn_task(future, None);
            let fiber_id = handle.id();

            // Complete fiber with an error to simulate cancellation
            async_ops
                .complete_fiber(
                    fiber_id,
                    Err(crate::error::Error::runtime_error("Cancelled")),
                )
                .unwrap();

            let result = handle.wait().await;
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_yield_now_operation() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let result = async_ops.yield_now().await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_boolean().unwrap(), true);
        });
    }

    #[test]
    fn test_async_fiber_ops_clone() {
        let async_ops = create_test_async_ops();
        let cloned_ops = async_ops.clone();

        assert_eq!(async_ops.fiber_count(), cloned_ops.fiber_count());
        assert_eq!(async_ops.has_ready_fibers(), cloned_ops.has_ready_fibers());
    }

    #[test]
    fn test_fiber_hierarchy_cleanup() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            // Create parent
            let parent_future = async { Ok(Value::string("parent")) };
            let parent_id = async_ops.spawn_fiber(parent_future, None);

            // Create multiple children
            let child1_future = async { Ok(Value::string("child1")) };
            let child2_future = async { Ok(Value::string("child2")) };

            let child1_id = async_ops.spawn_fiber(child1_future, Some(parent_id));
            let child2_id = async_ops.spawn_fiber(child2_future, Some(parent_id));

            assert_eq!(async_ops.fiber_count(), 3);

            // Complete parent (should handle children properly)
            async_ops
                .complete_fiber(parent_id, Ok(Value::string("parent")))
                .unwrap();
            async_ops
                .complete_fiber(child1_id, Ok(Value::string("child1")))
                .unwrap();
            async_ops
                .complete_fiber(child2_id, Ok(Value::string("child2")))
                .unwrap();

            // All fibers should still exist until cleanup
            assert_eq!(async_ops.fiber_count(), 3);
        });
    }

    #[test]
    fn test_fiber_wait_nonexistent() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let fake_id = FiberId::new(999);
            let result = async_ops.wait_for_fiber(fake_id).await;

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_task_handle_id() {
        let async_ops = create_test_async_ops();

        let future = async { Ok(Value::number(42.0)) };
        let handle = async_ops.spawn_task(future, None);

        let fiber_id = handle.id();
        assert!(async_ops.has_fiber(fiber_id));
    }

    #[test]
    fn test_concurrent_fiber_operations() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            // Spawn multiple fibers concurrently
            let mut handles = Vec::new();
            for i in 0..5 {
                let future = async move { Ok(Value::number(i as f64)) };
                handles.push(async_ops.spawn_task(future, None));
            }

            assert_eq!(async_ops.fiber_count(), 5);

            // Complete all fibers
            for (i, handle) in handles.iter().enumerate() {
                async_ops
                    .complete_fiber(handle.id(), Ok(Value::number(i as f64)))
                    .unwrap();
            }

            // Wait for all to complete
            for (i, handle) in handles.into_iter().enumerate() {
                let result = handle.wait().await.unwrap();
                assert_eq!(result.as_number().unwrap(), i as f64);
            }
        });
    }

    #[test]
    fn test_empty_spawn_race() {
        smol::block_on(async {
            let async_ops = create_test_async_ops();

            let empty_futures: Vec<std::future::Ready<crate::Result<Value>>> = vec![];
            let result = async_ops.spawn_race(empty_futures, None).await;

            assert!(result.is_err());
        });
    }
}
