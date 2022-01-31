pub mod simple_executor;
pub mod keyboard;
pub mod executor;

use core::{pin::Pin, future::Future, task::{Context, Poll}, sync::atomic::AtomicU64};
use alloc::boxed::Box;

pub struct Task {
    task_id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Task {
            task_id: TaskId::new(),
            future: Box::pin(future)
        }
    }

    fn poll(&mut self, cx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(cx)
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }
}