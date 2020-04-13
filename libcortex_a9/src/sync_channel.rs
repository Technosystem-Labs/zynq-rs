use core::{
    future::Future,
    pin::Pin,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
    task::{Context, Poll},
};
use alloc::{
    boxed::Box,
    sync::Arc,
    vec::Vec,
};
use super::asm::*;


type Channel<T> = Vec<AtomicPtr<T>>;

/// Create a bounded channel
///
/// Returns `(tx, rx)` where one should be used one the local core,
/// and the other is to be shared with another core.
pub fn sync_channel<T>(bound: usize) -> (Sender<T>, Receiver<T>) {
    // allow for bound=0
    let len = bound + 1;
    let mut channel = Vec::with_capacity(len);
    for _ in 0..len {
        channel.push(AtomicPtr::default());
    }

    let channel = Arc::new(channel);
    let sender = Sender {
        channel: channel.clone(),
        pos: 0,
    };
    let receiver = Receiver {
        channel: channel,
        pos: 0,
    };
    (sender, receiver)
}

/// Sending half of a channel
pub struct Sender<T> {
    channel: Arc<Channel<T>>,
    pos: usize,
}

impl<T> Sender<T> {
    /// Blocking send
    pub fn send<B: Into<Box<T>>>(&mut self, content: B) {
        let ptr = Box::into_raw(content.into());
        let entry = &self.channel[self.pos];
        // try to write the new pointer if the current pointer is
        // NULL, retrying while it is not NULL
        while entry.compare_and_swap(null_mut(), ptr, Ordering::Acquire) != null_mut() {
            // power-saving
            wfe();
        }
        dsb();
        // wake power-saving receivers
        sev();

        // advance
        self.pos += 1;
        // wrap
        if self.pos >= self.channel.len() {
            self.pos = 0;
        }
    }

    /// Non-blocking send, handing you back ownership of the content on **failure**
    pub fn try_send<B: Into<Box<T>>>(&mut self, content: B) -> Option<Box<T>> {
        let ptr = Box::into_raw(content.into());
        let entry = &self.channel[self.pos];
        // try to write the new pointer if the current pointer is
        // NULL
        if entry.compare_and_swap(null_mut(), ptr, Ordering::Acquire) == null_mut() {
            dsb();
            // wake power-saving receivers
            sev();

            // advance
            self.pos += 1;
            // wrap
            if self.pos >= self.channel.len() {
                self.pos = 0;
            }

            // success
            None
        } else {
            let content = unsafe { Box::from_raw(ptr) };
            // failure
            Some(content)
        }
    }

    pub async fn async_send<B: Into<Box<T>>>(&mut self, content: B) {
        struct Send<'a, T> {
            sender: &'a mut Sender<T>,
            content: Option<Box<T>>,
        }

        impl<T> Future for Send<'_, T> {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match self.content.take() {
                    Some(content) => {
                        if let Some(content) = self.sender.try_send(content) {
                            // failure
                            self.content = Some(content);
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        } else {
                            // success
                            Poll::Ready(())
                        }
                    }
                    None => panic!("Send future polled after success"),
                }
            }
        }

        Send {
            sender: self,
            content: Some(content.into()),
        }.await
    }
}




/// Receiving half of a channel
pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
    pos: usize,
}

impl<T> Receiver<T> {
    /// Blocking receive
    pub fn recv(&mut self) -> Box<T> {
        let entry = &self.channel[self.pos];

        loop {
            dmb();
            let ptr = entry.swap(null_mut(), Ordering::Release);
            if ptr != null_mut() {
                dsb();
                // wake power-saving senders
                sev();

                let content = unsafe { Box::from_raw(ptr) };

                // advance
                self.pos += 1;
                // wrap
                if self.pos >= self.channel.len() {
                    self.pos = 0;
                }

                return content;
            }

            // power-saving
            wfe();
        }
    }

    /// Non-blocking receive
    pub fn try_recv(&mut self) -> Option<Box<T>> {
        let entry = &self.channel[self.pos];

        dmb();
        let ptr = entry.swap(null_mut(), Ordering::Release);
        if ptr != null_mut() {
            dsb();
            // wake power-saving senders
            sev();

            let content = unsafe { Box::from_raw(ptr) };

            // advance
            self.pos += 1;
            // wrap
            if self.pos >= self.channel.len() {
                self.pos = 0;
            }

            Some(content)
        } else {
            None
        }
    }

    pub async fn async_recv(&mut self) -> Box<T> {
        struct Recv<'a, T> {
            receiver: &'a mut Receiver<T>,
        }

        impl<T> Future for Recv<'_, T> {
            type Output = Box<T>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if let Some(content) = self.receiver.try_recv() {
                    Poll::Ready(content)
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

        Recv {
            receiver: self,
        }.await
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = Box<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.recv())
    }
}
