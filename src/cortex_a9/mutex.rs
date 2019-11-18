use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};
use core::cell::UnsafeCell;
use super::asm::*;

const LOCKED: u32 = 1;
const UNLOCKED: u32 = 0;

pub struct Mutex<T> {
    locked: AtomicU32,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Mutex{
            locked: AtomicU32::new(UNLOCKED),
            inner: UnsafeCell::new(inner),
        }
    }
    
    pub fn lock(&self) -> MutexGuard<T> {
        while self.locked.compare_and_swap(UNLOCKED, LOCKED, Ordering::Acquire) != UNLOCKED {}
        dmb();
        MutexGuard { mutex: self }
    }

    fn unlock(&self) {
        dmb();
        self.locked.store(UNLOCKED, Ordering::Release);
        dsb();
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
