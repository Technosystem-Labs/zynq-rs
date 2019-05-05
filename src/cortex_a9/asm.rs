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
