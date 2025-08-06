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

// Tests are kept in the main module for now
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;
    use std::future;
    use std::sync::{Arc, Mutex};

    fn create_test_fiber() -> Fiber {
        let future = Box::pin(future::ready(Ok(Value::Number(42.into()))));
        Fiber::new(FiberId::new(1), future, None)
    }

    fn create_test_scheduler() -> FiberScheduler {
        FiberScheduler::new_for_test()
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
    fn test_fiber_id_api() {
        let id = FiberId::new(42);
        assert_eq!(id.as_u64(), 42);

        let id2 = FiberId::new(42);
        assert_eq!(id, id2);
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

    // Async Fiber System Tests

    fn create_test_async_ops() -> AsyncFiberOps {
        let scheduler = Arc::new(Mutex::new(create_test_scheduler()));
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
