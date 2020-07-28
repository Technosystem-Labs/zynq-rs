use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::cell::UnsafeCell;
use core::ptr::NonNull;
use libboard_zynq::ddr::DdrRam;
use libcortex_a9::regs::MPIDR;
use libregister::RegisterR;
use linked_list_allocator::Heap;

#[global_allocator]
static mut ALLOCATOR: CortexA9Alloc = CortexA9Alloc(
    UnsafeCell::new(Heap::empty()),
    UnsafeCell::new(Heap::empty()),
);

struct CortexA9Alloc(UnsafeCell<Heap>, UnsafeCell<Heap>);

unsafe impl Sync for CortexA9Alloc {}

unsafe impl GlobalAlloc for CortexA9Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if cfg!(not(feature = "alloc_core")) || MPIDR.read().cpu_id() == 0 {
            self.0.get().as_mut()
        } else {
            self.1.get().as_mut()
        }
        .unwrap()
        .allocate_first_fit(layout)
        .ok()
        .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if cfg!(not(feature = "alloc_core")) || MPIDR.read().cpu_id() == 0 {
            self.0.get().as_mut()
        } else {
            self.1.get().as_mut()
        }
        .unwrap()
        .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

#[cfg(not(feature = "alloc_core"))]
pub fn init_alloc_ddr(ddr: &mut DdrRam) {
    unsafe {
        ALLOCATOR
            .0
            .get()
            .as_mut()
            .unwrap()
            .init(ddr.ptr::<u8>() as usize, ddr.size());
    }
}

#[cfg(feature = "alloc_core")]
extern "C" {
    static __heap0_start: usize;
    static __heap0_end: usize;
    static __heap1_start: usize;
    static __heap1_end: usize;
}

#[cfg(feature = "alloc_core")]
pub fn init_alloc_core0() {
    unsafe {
        let start = &__heap0_start as *const usize as usize;
        let end = &__heap0_end as *const usize as usize;
        ALLOCATOR.0.get().as_mut().unwrap().init(start, end - start);
    }
}

#[cfg(feature = "alloc_core")]
pub fn init_alloc_core1() {
    unsafe {
        let start = &__heap1_start as *const usize as usize;
        let end = &__heap1_end as *const usize as usize;
        ALLOCATOR.1.get().as_mut().unwrap().init(start, end - start);
    }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    let id = MPIDR.read().cpu_id();
    let heap = unsafe {
        if cfg!(not(feature = "alloc_core")) || id == 0 {
            ALLOCATOR.0.get()
        } else {
            ALLOCATOR.1.get()
        }
        .as_mut()
        .unwrap()
    };
    panic!(
        "Core {} alloc_error, layout: {:?}, used memory: {}",
        id,
        layout,
        heap.used()
    );
}
