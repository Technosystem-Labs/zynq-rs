use core::{
    pin::Pin,
    future::Future,
    ptr::drop_in_place,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
    task::{Context, Poll},
};
use alloc::boxed::Box;
use super::{spin_lock_yield, notify_spin_lock};

pub struct Sender<'a, T> where T: Clone {
    list: &'a [AtomicPtr<T>],
    write: &'a AtomicUsize,
    read: &'a AtomicUsize,
}

pub struct Receiver<'a, T> where T: Clone {
    list: &'a [AtomicPtr<T>],
    write: &'a AtomicUsize,
    read: &'a AtomicUsize,
}

impl<'a, T> Sender<'a, T> where T: Clone {
    pub const fn new(list: &'static [AtomicPtr<T>], write: &'static AtomicUsize, read: &'static AtomicUsize) -> Self {
        Sender {list, write, read}
    }

    pub fn try_send<B: Into<Box<T>>>(&mut self, content: B) -> Result<(), B> {
        let write = self.write.load(Ordering::Relaxed);
        if (write + 1) % self.list.len() == self.read.load(Ordering::Acquire) {
            Err(content)
        } else {
            let ptr = Box::into_raw(content.into());
            let entry = &self.list[write];
            let prev = entry.swap(ptr, Ordering::Relaxed);
            // we allow other end get it first
            self.write.store((write + 1) % self.list.len(), Ordering::Release);
            notify_spin_lock();
            if !prev.is_null() {
                unsafe {
                    drop_in_place(prev);
                }
            }
            Ok(())
        }
    }

    pub fn send<B: Into<Box<T>>>(&mut self, content: B) {
        let mut content = content;
        while let Err(back) = self.try_send(content) {
            content = back;
            spin_lock_yield();
        }
    }

    pub async fn async_send<B: Into<Box<T>>>(&mut self, content: B) {
        struct Send<'a, 'b, T> where T: Clone, 'b: 'a {
            sender: &'a mut Sender<'b, T>,
            content: Result<(), Box<T>>,
        }

        impl<T> Future for Send<'_, '_, T> where T: Clone {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match core::mem::replace(&mut self.content, Ok(())) {
                    Err(content) => {
                        if let Err(content) = self.sender.try_send(content) {
                            // failure
                            self.content = Err(content);
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        } else {
                            // success
                            Poll::Ready(())
                        }
                    }
                    Ok(_) => panic!("Send future polled after success"),
                }
            }
        }

        Send {
            sender: self,
            content: Err(content.into()),
        }.await
    }

    /// free all items in the queue. It is the user's responsibility to
    /// ensure no reader is trying to copy the data.
    pub unsafe fn drop_elements(&mut self) {
        for v in self.list.iter() {
            let original = v.swap(core::ptr::null_mut(), Ordering::Relaxed);
            if !original.is_null() {
                drop_in_place(original);
            }
        }
    }

    /// Reset the `sync_channel`, *forget* all items in the queue. Affects both the sender and
    /// receiver.
    pub unsafe fn reset(&mut self) {
        self.write.store(0, Ordering::Relaxed);
        self.read.store(0, Ordering::Relaxed);
        for v in self.list.iter() {
            v.store(core::ptr::null_mut(), Ordering::Relaxed);
        }
    }
}

impl<'a, T> Receiver<'a, T> where T: Clone {
    pub const fn new(list: &'static [AtomicPtr<T>], write: &'static AtomicUsize, read: &'static AtomicUsize) -> Self {
        Receiver {list, write, read}
    }

    pub fn try_recv(&mut self) -> Result<T, ()> {
        let read = self.read.load(Ordering::Relaxed);
        if read == self.write.load(Ordering::Acquire) {
            Err(())
        } else {
            let entry = &self.list[read];
            let data = unsafe {
                // we cannot deallocate the box
                Box::leak(Box::from_raw(entry.load(Ordering::Relaxed)))
            };
            let result = data.clone();
            self.read.store((read + 1) % self.list.len(), Ordering::Release);
            notify_spin_lock();
            Ok(result)
        }
    }

    pub fn recv(&mut self) -> T {
        loop {
            if let Ok(data) = self.try_recv() {
                return data;
            }
            spin_lock_yield();
        }
    }

    pub async fn async_recv(&mut self) -> T {
        struct Recv<'a, 'b, T> where T: Clone, 'b: 'a {
            receiver: &'a mut Receiver<'b, T>,
        }

        impl<T> Future for Recv<'_, '_, T> where T: Clone {
            type Output = T;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if let Ok(content) = self.receiver.try_recv() {
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

impl<'a, T> Iterator for Receiver<'a, T> where T: Clone {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.recv())
    }
}

#[macro_export]
/// Macro for initializing the sync_channel with static buffer and indexes.
/// Note that this requires `#![feature(const_in_array_repeat_expressions)]`
macro_rules! sync_channel {
    ($t: ty, $cap: expr) => {
        {
            use core::sync::atomic::{AtomicUsize, AtomicPtr};
            use $crate::sync_channel::{Sender, Receiver};
            static LIST: [AtomicPtr<$t>; $cap + 1] = [AtomicPtr::new(core::ptr::null_mut()); $cap + 1];
            static WRITE: AtomicUsize = AtomicUsize::new(0);
            static READ: AtomicUsize = AtomicUsize::new(0);
            (Sender::new(&LIST, &WRITE, &READ), Receiver::new(&LIST, &WRITE, &READ))
        }
    };
}
