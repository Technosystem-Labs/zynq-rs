use core::{
    ops::{Deref, DerefMut},
    mem::{align_of, size_of},
};
use alloc::alloc::{alloc_zeroed, dealloc, Layout, LayoutErr};
use crate::mmu::{L1_PAGE_SIZE, L1Table};

pub struct Uncached<'t, T: 't> {
    layout: Layout,
    data: &'t mut T,
}

impl<'t, T: 't> Uncached<'t, T> {
    /// allocates in chunks of 1 MB
    pub fn alloc(src: T) -> Result<Self, LayoutErr> {
        // round to full pages
        let size = (size_of::<T>() | (L1_PAGE_SIZE - 1)) + 1;
        let align = align_of::<T>()
            .max(L1_PAGE_SIZE);
        let layout = Layout::from_size_align(size, align)?;
        let ptr = unsafe { alloc::alloc::alloc(layout).cast::<T>() };
        assert_eq!((ptr as usize) & (L1_PAGE_SIZE - 1), 0);

        let start = ptr as usize;
        for page_start in (start..(start + size)).step_by(L1_PAGE_SIZE) {
            L1Table::get()
                .update(page_start as *const (), |l1_section| {
                    l1_section.cacheable = false;
                    l1_section.bufferable = false;
                });
        }
        
        // initialize
        unsafe {
            core::ptr::write(ptr, src);
        }
        let data = unsafe { &mut *ptr };
        Ok(Uncached { layout, data })
    }
}

impl<'t, T: 't> Drop for Uncached<'t, T> {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data as *mut _ as *mut u8, self.layout);
        }
    }
}

impl<'t, T: 't> Deref for Uncached<'t, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'t, T: 't> DerefMut for Uncached<'t, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}
