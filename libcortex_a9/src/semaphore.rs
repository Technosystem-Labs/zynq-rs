use super::{spin_lock_yield, notify_spin_lock};
use core::{
    task::{Context, Poll},
    pin::Pin,
    future::Future,
    sync::atomic::{AtomicI32, Ordering}
};

pub struct Semaphore {
    value: AtomicI32,
    max: i32
}

impl Semaphore {
    pub const fn new(value: i32, max: i32) -> Self {
        Semaphore { value: AtomicI32::new(value), max}
    }

    pub fn try_wait(&self) -> Option<()> {
        loop {
            let value = self.value.load(Ordering::Relaxed);
            if value > 0 {
                if self.value.compare_exchange_weak(value, value - 1, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                    return Some(());
                }
            } else {
                return None;
            }
        }
    }

    pub fn wait(&self) {
        while self.try_wait().is_none() {
            spin_lock_yield();
        }
    }

    pub async fn async_wait(&self) {
        struct Fut<'a>(&'a Semaphore);

        impl Future for Fut<'_> {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.0.try_wait() {
                    Some(_) => Poll::Ready(()),
                    None => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
        }

        Fut(&self).await
    }

    pub fn signal(&self) {
        loop {
            let value = self.value.load(Ordering::Relaxed);
            if value < self.max {
                if self.value.compare_exchange_weak(value, value + 1, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                    notify_spin_lock();
                    return;
                }
            } else {
                return;
            }
        }
    }
}

