/// The classic no-op
#[inline]
pub fn nop() {
    unsafe { asm!("nop" :::: "volatile") }
}

/// Wait For Event
#[inline]
pub fn wfe() {
    unsafe { asm!("wfe" :::: "volatile") }
}

/// Send Event
#[inline]
pub fn sev() {
    unsafe { asm!("sev" :::: "volatile") }
}

/// Data Memory Barrier
#[inline]
pub fn dmb() {
    unsafe { asm!("dmb" :::: "volatile") }
}

/// Data Synchronization Barrier
#[inline]
pub fn dsb() {
    unsafe { asm!("dsb" :::: "volatile") }
}

/// Instruction Synchronization Barrier
#[inline]
pub fn isb() {
    unsafe { asm!("isb" :::: "volatile") }
}
