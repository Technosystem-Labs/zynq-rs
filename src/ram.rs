use core::alloc::GlobalAlloc;
use core::ptr::NonNull;
use alloc::alloc::Layout;
use linked_list_allocator::Heap;
use crate::cortex_a9::mutex::Mutex;
use crate::zynq::ddr::DdrRam;

#[global_allocator]
static ALLOCATOR: CortexA9Alloc = CortexA9Alloc(Mutex::new(Heap::empty()));

/// LockedHeap doesn't locking properly
struct CortexA9Alloc(Mutex<Heap>);

unsafe impl Sync for CortexA9Alloc {}

unsafe impl GlobalAlloc for CortexA9Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock()
            .allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

pub fn init_alloc(ddr: &mut DdrRam) {
    unsafe {
        ALLOCATOR.0.lock()
            .init(ddr.ptr::<u8>() as usize, ddr.size());
    }
}


#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    panic!("alloc_error")
}
