use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use super::asm::*;

#[inline]
fn wait_for_update() {
    wfe();
}

#[inline]
fn signal_update() {
    dsb();
    sev();
}

pub struct Mutex<T> {
    locked: AtomicBool,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Mutex{
            locked: AtomicBool::new(false),
            inner: UnsafeCell::new(inner),
        }
    }
    
    pub fn lock(&self) -> MutexGuard<T> {
        while self.locked.compare_and_swap(false, true, Ordering::Acquire) {
            while self.locked.load(Ordering::Relaxed) {
                wait_for_update();
            }
        }

        dmb();

        MutexGuard { mutex: self }
    }

    fn unlock(&self) {
        dmb();
        self.locked.store(false, Ordering::Release);
        signal_update();
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
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

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}
