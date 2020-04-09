use core::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
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
}
