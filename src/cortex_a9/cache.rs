/// Invalidate TLBs
#[inline(always)]
pub fn tlbiall() {
    unsafe {
        asm!("mcr p15, 0, $0, c8, c7, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate I-Cache
#[inline(always)]
pub fn iciallu() {
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate Branch Predictor Array
#[inline(always)]
pub fn bpiall() {
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 6" :: "r" (0) :: "volatile");
    }
}

#[inline(always)]
pub fn dcisw(setway: u32) {
    unsafe {
        // acc. to ARM Architecture Reference Manual, Figure B3-32;
        // also see example code (for DCCISW, but DCISW will be
        // analogous) "Example code for cache maintenance operations"
        // on pages B2-1286 and B2-1287.
        asm!("mcr p15, 0, $0, c7, c6, 2" :: "r" (setway) :: "volatile");
    }
}

/// A made-up "instruction": invalidate all of the L1 D-Cache
#[inline(always)]
pub fn dciall() {
    // the cache associativity could be read from a register, but will
    // always be 4 in L1 data cache of a cortex a9
    let ways = 4;
    let bit_pos_of_way = 30; // 32 - log2(ways)

    // the cache sets could be read from a register, but are always
    // 256 for the cores in the zync-7000; in general, 128 or 512 are
    // also possible.
    let sets = 256;
    let bit_pos_of_set = 5; // for a line size of 8 words = 2^5 bytes

    // select L1 data cache
    unsafe {
        asm!("mcr p15, 2, $0, c0, c0, 0" :: "r" (0) :: "volatile");
    }

    // Invalidate entire D-Cache by iterating every set and every way
    for set in 0..sets {
        for way in 0..ways {
            dcisw((set << bit_pos_of_set) | (way << bit_pos_of_way));
        }
    }
}

/// Data cache clear and invalidate by memory virtual address. This
/// flushes data out to the point of coherency, and invalidates the
/// corresponding cache line (as appropriate when DMA is meant to be
/// writing into it).
#[inline(always)]
pub fn dccimva(addr: usize) {
    unsafe {
        asm!("mcr p15, 0, $0, c7, c14, 1" :: "r" (addr) :: "volatile");
    }
}

/// The DCCIVMA (data cache clear and invalidate) applied to the
/// region of memory occupied by the argument. This does not modify
/// the argument, but due to the invalidate part (only ever needed if
/// external write access is to be granted, e.g. by DMA) it only makes
/// sense if the caller has exclusive access to it as otherwise other
/// accesses might just bring it back into the data cache.
pub fn dcci<T>(object: &mut T) {
    let cache_line = 0x20;
    let first_addr =
        (object as *mut _ as *const _ as usize) & !(cache_line - 1);
    let beyond_addr = (
        (object as *mut _ as *const _ as usize)
            + core::mem::size_of_val(object)
            + (cache_line - 1)
    ) & !(cache_line - 1);
    for addr in (first_addr..beyond_addr).step_by(cache_line) {
        dccimva(addr);
    }
}

pub fn dcci_slice_content<T>(slice: &mut [T]) {
    if slice.len() == 0 {
        return;
    }
    let cache_line = 0x20;
    let first_addr =
        (&slice[0] as *const _ as usize) & !(cache_line - 1);
    let beyond_addr = (
        (&slice[slice.len() - 1] as *const _ as usize)
            + (cache_line - 1)
    ) & !(cache_line - 1);
    for addr in (first_addr..beyond_addr).step_by(cache_line) {
        dccimva(addr);
    }
}

pub fn dcci_slice_content_unmut<T>(slice: &[T]) {
    if slice.len() == 0 {
        return;
    }
    let cache_line = 0x20;
    let first_addr =
        (&slice[0] as *const _ as usize) & !(cache_line - 1);
    let beyond_addr = (
        (&slice[slice.len() - 1] as *const _ as usize)
            + (cache_line - 1)
    ) & !(cache_line - 1);
    for addr in (first_addr..beyond_addr).step_by(cache_line) {
        dccimva(addr);
    }
}

/// Data cache invalidate by memory virtual address. This and
/// invalidates the cache line containing the given address. Super
/// unsafe, as this discards a write-back cache line, potentially
/// affecting more data than intended.
#[inline(always)]
pub unsafe fn dcimva(addr: usize) {
    asm!("mcr p15, 0, $0, c7, c6, 1" :: "r" (addr) :: "volatile");
}

/// Data cache invalidate for an object. Panics if not properly
/// aligned and properly sized to be contained in an exact number of
/// cache lines.
pub fn dci<T>(object: &mut T) {
    let cache_line = 0x20;
    let first_addr = object as *mut _ as *const _ as usize;
    let beyond_addr = (object as *mut _ as *const _ as usize) +
        core::mem::size_of_val(object);
    assert_eq!((first_addr & (cache_line - 1)), 0x00);
    assert_eq!((beyond_addr & (cache_line - 1)), 0x00);
    for addr in (first_addr..beyond_addr).step_by(cache_line) {
        unsafe {
            dcimva(addr);
        }
    }
}

/// Data cache invalidate for the contents of a slice. Panics if not
/// properly aligned and properly sized to be contained in an exact
/// number of cache lines.
pub fn dci_slice_content<T>(slice: &mut [T]) {
    if slice.len() == 0 {
        return;
    }
    let cache_line = 0x20;
    let first_addr = &slice[0] as *const _ as usize;
    let beyond_addr = (&slice[slice.len() - 1] as *const _ as usize)
        + core::mem::size_of::<T>();
    assert_eq!((first_addr & (cache_line - 1)), 0x00);
    assert_eq!((beyond_addr & (cache_line - 1)), 0x00);
    for addr in (first_addr..beyond_addr).step_by(cache_line) {
        unsafe {
            dcimva(addr);
        }
    }
}

pub unsafe fn dci_more_than_slice_content<T>(slice: &mut [T]) {
    if slice.len() == 0 {
        return;
    }
    let cache_line = 0x20;
    let first_addr =
        (&slice[0] as *const _ as usize) & !(cache_line - 1);
    let beyond_addr = (
        (&slice[slice.len() - 1] as *const _ as usize)
            + (cache_line - 1)
    ) & !(cache_line - 1);
    assert_eq!((first_addr & (cache_line - 1)), 0x00);
    assert_eq!((beyond_addr & (cache_line - 1)), 0x00);
    for addr in (first_addr..beyond_addr).step_by(cache_line) {
        dcimva(addr);
    }
}

pub unsafe fn dci_more_than_slice_content_nonmut<T>(slice: &[T]) {
    if slice.len() == 0 {
        return;
    }
    let cache_line = 0x20;
    let first_addr =
        (&slice[0] as *const _ as usize) & !(cache_line - 1);
    let beyond_addr = (
        (&slice[slice.len() - 1] as *const _ as usize)
            + (cache_line - 1)
    ) & !(cache_line - 1);
    assert_eq!((first_addr & (cache_line - 1)), 0x00);
    assert_eq!((beyond_addr & (cache_line - 1)), 0x00);
    for addr in (first_addr..beyond_addr).step_by(cache_line) {
        dcimva(addr);
    }
}
