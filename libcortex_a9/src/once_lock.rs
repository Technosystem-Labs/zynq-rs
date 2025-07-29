use core::{cell::UnsafeCell,
           sync::atomic::{AtomicBool, Ordering}};

use crate::asm::{enter_critical, exit_critical};

pub struct OnceLock<T> {
    once: AtomicBool,
    data: UnsafeCell<Option<T>>,
}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        Self {
            once: AtomicBool::new(false),
            data: UnsafeCell::new(None),
        }
    }

    pub fn set(&self, data: T) -> Result<(), ()> {
        let irq = unsafe { enter_critical() };
        let res = self
            .once
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_| ())
            .map(|_| unsafe { *self.data.get() = Some(data) });
        unsafe { exit_critical(irq) };
        res
    }

    pub fn get(&self) -> Option<&T> {
        unsafe { &*self.data.get() }.as_ref()
    }
}

unsafe impl<T> Sync for OnceLock<T> {}
unsafe impl<T> Send for OnceLock<T> {}
