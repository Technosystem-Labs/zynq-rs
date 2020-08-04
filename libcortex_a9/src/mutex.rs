use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};
use core::cell::UnsafeCell;
use super::{
    spin_lock_yield, notify_spin_lock,
    asm::{dmb, enter_critical, exit_critical}
};

const LOCKED: u32 = 1;
const UNLOCKED: u32 = 0;

/// Mutex implementation for Cortex-A9
///
/// [ARM Synchronization Primitives Development Article: Implementing a mutex](http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dht0008a/ch01s03s02.html)
pub struct Mutex<T> {
    locked: AtomicU32,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    /// Constructor, const-fn
    pub const fn new(inner: T) -> Self {
        Mutex{
            locked: AtomicU32::new(UNLOCKED),
            inner: UnsafeCell::new(inner),
        }
    }

    /// Lock the Mutex, blocks when already locked
    pub fn lock(&self) -> MutexGuard<T> {
        let mut irq = unsafe { enter_critical() };
        while self.locked.compare_and_swap(UNLOCKED, LOCKED, Ordering::Acquire) != UNLOCKED {
            unsafe {
                exit_critical(irq);
                spin_lock_yield();
                irq = enter_critical();
            }
        }
        dmb();
        MutexGuard { mutex: self, irq }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        let irq = unsafe { enter_critical() };
        if self.locked.compare_and_swap(UNLOCKED, LOCKED, Ordering::Acquire) != UNLOCKED {
            unsafe { exit_critical(irq) };
            None
        } else {
            dmb();
            Some(MutexGuard { mutex: self, irq })
        }
    }

    fn unlock(&self) {
        dmb();
        self.locked.store(UNLOCKED, Ordering::Release);

        notify_spin_lock();
    }
}

/// Returned by `Mutex.lock()`, allows access to data via
/// `Deref`/`DerefMutx`
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    irq: bool,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.inner.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.inner.get() }
    }
}

/// Automatically `Mutex.unlock()` when this reference is dropped
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
        unsafe { exit_critical(self.irq) };
    }
}
