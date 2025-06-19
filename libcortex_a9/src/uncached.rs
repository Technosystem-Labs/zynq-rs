use alloc::alloc::{Layout, LayoutError, dealloc};
use core::{mem::{align_of, size_of},
           ops::{Deref, DerefMut}};

use crate::mmu::{L1_PAGE_SIZE, L1Table};

pub struct UncachedSlice<T: 'static> {
    layout: Layout,
    slice: &'static mut [T],
}

impl<T> UncachedSlice<T> {
    /// allocates in chunks of 1 MB
    pub fn new<F: Fn() -> T>(len: usize, default: F) -> Result<Self, LayoutError> {
        // round to full pages
        let size = ((len * size_of::<T>() - 1) | (L1_PAGE_SIZE - 1)) + 1;
        let align = align_of::<T>().max(L1_PAGE_SIZE);
        let layout = Layout::from_size_align(size, align)?;
        let ptr = unsafe { alloc::alloc::alloc(layout).cast::<T>() };
        assert!(!ptr.is_null());
        let start = ptr as usize;

        for page_start in (start..(start + size)).step_by(L1_PAGE_SIZE) {
            // non-shareable device
            L1Table::get().update(page_start as *const (), |l1_section| {
                l1_section.tex = 0b10;
                l1_section.cacheable = true;
                l1_section.bufferable = false;
            });
        }

        let slice = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        for e in slice.iter_mut() {
            *e = default();
        }
        Ok(UncachedSlice { layout, slice })
    }
}

/// Does not yet mark the pages cachable again
impl<T> Drop for UncachedSlice<T> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.slice.as_mut_ptr() as *mut _ as *mut u8, self.layout);
        }
    }
}

impl<T> Deref for UncachedSlice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

impl<T> DerefMut for UncachedSlice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice
    }
}
