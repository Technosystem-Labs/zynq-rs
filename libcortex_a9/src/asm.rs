/// The classic no-op
#[inline]
pub fn nop() {
    unsafe { llvm_asm!("nop" :::: "volatile") }
}

/// Wait For Event
#[inline]
pub fn wfe() {
    unsafe { llvm_asm!("wfe" :::: "volatile") }
}

/// Send Event
#[inline]
pub fn sev() {
    unsafe { llvm_asm!("sev" :::: "volatile") }
}

/// Data Memory Barrier
#[inline]
pub fn dmb() {
    unsafe { llvm_asm!("dmb" :::: "volatile") }
}

/// Data Synchronization Barrier
#[inline]
pub fn dsb() {
    unsafe { llvm_asm!("dsb" :::: "volatile") }
}

/// Instruction Synchronization Barrier
#[inline]
pub fn isb() {
    unsafe { llvm_asm!("isb" :::: "volatile") }
}
