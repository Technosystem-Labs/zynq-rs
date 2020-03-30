//! Asynchronous tasks

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use super::executor;

/// Drives the future `f` to completion
///
/// This also makes any previously `spawn`-ed future make progress
pub fn block_on<T>(f: impl Future<Output = T>) -> T {
    executor::current().block_on(f)
}

/// Spawns a task onto the executor
///
/// The spawned task will not make any progress until `block_on` is called.
pub fn spawn(f: impl Future + 'static) {
    executor::current().spawn(f)
}

/// Use `r#yield.await` to suspend the execution of a task
pub async fn r#yield() {
    struct Yield {
        yielded: bool,
    }

    impl Future for Yield {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                // wake ourselves
                cx.waker().wake_by_ref();
                //asm::sev();
                Poll::Pending
            }
        }
    }

    Yield { yielded: false }.await
}
