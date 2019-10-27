use core::mem::{align_of, size_of};

pub trait RAM {
    #[inline(always)]
    fn addr(&self) -> usize;
    #[inline(always)]
    fn size(&self) -> usize;

    #[inline(always)]
    fn into_memory_region(&self) -> MemoryRegion {
        MemoryRegion {
            addr: self.addr(),
            size: self.size(),
        }
    }

}

pub struct MemoryRegion {
    addr: usize,
    size: usize,
}

impl MemoryRegion {
    #[inline(always)]
    fn split(&mut self, offset: usize) -> MemoryRegion {
        if offset > self.size {
            panic!("StaticAllocator: unable to split {}/{} bytes", offset, self.size);
        }
        
        let result = MemoryRegion {
            addr: self.addr,
            size: offset,
        };
        self.addr += offset;
        self.size -= offset;
        result
    }

    unsafe fn clear<T>(self) -> *mut T {
        let result = self.addr as *mut T;
        for b in core::slice::from_raw_parts_mut::<u8>(result as *mut u8, self.size) {
            *b = 0;
        }
        result
    }

    pub fn align(&mut self, alignment: usize) {
        let mask = alignment - 1;
        let addr = (self.addr | mask) + 1;
        self.size -= addr - self.addr;
        self.addr = addr;
    }
}

pub struct StaticAllocator {
    free: MemoryRegion,
}

impl StaticAllocator {
    pub fn new<R: RAM>(r: R) -> Self {
        StaticAllocator {
            free: r.into_memory_region(),
        }
    }

    pub fn alloc<'a: 'b, 'b, T: Sized>(&'a mut self) -> &'b mut T {
        self.free.align(align_of::<T>());

        let size = size_of::<T>();
        let region = self.free.split(size);

        unsafe { &mut *region.clear() }
    }
}
