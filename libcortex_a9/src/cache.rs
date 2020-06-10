/// Invalidate TLBs
#[inline(always)]
pub fn tlbiall() {
    unsafe {
        llvm_asm!("mcr p15, 0, $0, c8, c7, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate I-Cache
#[inline(always)]
pub fn iciallu() {
    unsafe {
        llvm_asm!("mcr p15, 0, $0, c7, c5, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate Branch Predictor Array
#[inline(always)]
pub fn bpiall() {
    unsafe {
        llvm_asm!("mcr p15, 0, $0, c7, c5, 6" :: "r" (0) :: "volatile");
    }
}

/// Data cache clean by set/way
#[inline(always)]
pub fn dccsw(setway: u32) {
    unsafe {
        llvm_asm!("mcr p15, 0, $0, c7, c10, 2" :: "r" (setway) :: "volatile");
    }
}

/// Data cache invalidate by set/way
#[inline(always)]
pub fn dcisw(setway: u32) {
    unsafe {
        // acc. to ARM Architecture Reference Manual, Figure B3-32;
        // also see example code (for DCCISW, but DCISW will be
        // analogous) "Example code for cache maintenance operations"
        // on pages B2-1286 and B2-1287.
        llvm_asm!("mcr p15, 0, $0, c7, c6, 2" :: "r" (setway) :: "volatile");
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
        llvm_asm!("mcr p15, 2, $0, c0, c0, 0" :: "r" (0) :: "volatile");
    }

    // Invalidate entire D-Cache by iterating every set and every way
    for set in 0..sets {
        for way in 0..ways {
            dcisw((set << bit_pos_of_set) | (way << bit_pos_of_way));
        }
    }
}


const CACHE_LINE: usize = 0x20;
const CACHE_LINE_MASK: usize = CACHE_LINE - 1;

#[inline]
fn cache_line_addrs(first_addr: usize, beyond_addr: usize) -> impl Iterator<Item = usize> {
    let first_addr = first_addr & !CACHE_LINE_MASK;
    let beyond_addr = (beyond_addr | CACHE_LINE_MASK) + 1;

    (first_addr..beyond_addr).step_by(CACHE_LINE)
}

fn object_cache_line_addrs<T>(object: &T) -> impl Iterator<Item = usize> {
    let first_addr = object as *const _ as usize;
    let beyond_addr = (object as *const _ as usize) + core::mem::size_of_val(object);
    cache_line_addrs(first_addr, beyond_addr)
}

fn slice_cache_line_addrs<T>(slice: &[T]) -> impl Iterator<Item = usize> {
    let first_addr = &slice[0] as *const _ as usize;
    let beyond_addr = (&slice[slice.len() - 1] as *const _ as usize) +
        core::mem::size_of_val(&slice[slice.len() - 1]);
    cache_line_addrs(first_addr, beyond_addr)
}

/// Data cache clean and invalidate by memory virtual address. This
/// flushes data out to the point of coherency, and invalidates the
/// corresponding cache line (as appropriate when DMA is meant to be
/// writing into it).
#[inline(always)]
pub fn dccimvac(addr: usize) {
    unsafe {
        llvm_asm!("mcr p15, 0, $0, c7, c14, 1" :: "r" (addr) :: "volatile");
    }
}

/// Data cache clean and invalidate for an object.
pub fn dcci<T>(object: &T) {
    for addr in object_cache_line_addrs(object) {
        dccimvac(addr);
    }
}

pub fn dcci_slice<T>(slice: &[T]) {
    for addr in slice_cache_line_addrs(slice) {
        dccimvac(addr);
    }
}

/// Data cache clean by memory virtual address.
#[inline(always)]
pub fn dccmvac(addr: usize) {
    unsafe {
        llvm_asm!("mcr p15, 0, $0, c7, c10, 1" :: "r" (addr) :: "volatile");
    }
}

/// Data cache clean for an object.
pub fn dcc<T>(object: &T) {
    for addr in object_cache_line_addrs(object) {
        dccmvac(addr);
    }
}

/// Data cache clean for an object. Panics if not properly
/// aligned and properly sized to be contained in an exact number of
/// cache lines.
pub fn dcc_slice<T>(slice: &[T]) {
    for addr in slice_cache_line_addrs(slice) {
        dccmvac(addr);
    }
}

/// Data cache invalidate by memory virtual address. This and
/// invalidates the cache line containing the given address. Super
/// unsafe, as this discards a write-back cache line, potentially
/// affecting more data than intended.
#[inline(always)]
pub unsafe fn dcimvac(addr: usize) {
    llvm_asm!("mcr p15, 0, $0, c7, c6, 1" :: "r" (addr) :: "volatile");
}

/// Data cache clean and invalidate for an object.
pub unsafe fn dci<T>(object: &mut T) {
    let first_addr = object as *const _ as usize;
    let beyond_addr = (object as *const _ as usize) + core::mem::size_of_val(object);
    assert_eq!(first_addr & CACHE_LINE_MASK, 0, "dci object first_addr must be aligned");
    assert_eq!(beyond_addr & CACHE_LINE_MASK, 0, "dci object beyond_addr must be aligned");

    for addr in (first_addr..beyond_addr).step_by(CACHE_LINE) {
        dcimvac(addr);
    }
}

pub unsafe fn dci_slice<T>(slice: &mut [T]) {
    let first_addr = &slice[0] as *const _ as usize;
    let beyond_addr = (&slice[slice.len() - 1] as *const _ as usize) +
        core::mem::size_of_val(&slice[slice.len() - 1]);
    assert_eq!(first_addr & CACHE_LINE_MASK, 0, "dci slice first_addr must be aligned");
    assert_eq!(beyond_addr & CACHE_LINE_MASK, 0, "dci slice beyond_addr must be aligned");

    for addr in (first_addr..beyond_addr).step_by(CACHE_LINE) {
        dcimvac(addr);
    }
}
