use core::arch::asm;

/// The classic no-op
#[inline]
pub fn nop() {
    unsafe { asm!("nop") }
}

/// Wait For Event
#[inline]
pub fn wfe() {
    unsafe { asm!("wfe") }
}

/// Send Event
#[inline]
pub fn sev() {
    unsafe { asm!("sev") }
}

/// Data Memory Barrier
#[inline]
pub fn dmb() {
    unsafe { asm!("dmb") }
}

/// Data Synchronization Barrier
#[inline]
pub fn dsb() {
    unsafe { asm!("dsb") }
}

/// Instruction Synchronization Barrier
#[inline]
pub fn isb() {
    unsafe { asm!("isb") }
}

/// Enable FIQ
#[inline]
pub unsafe fn enable_fiq() {
    asm!("cpsie f");
}

/// Enable IRQ
#[inline]
pub unsafe fn enable_irq() {
    asm!("cpsie i");
}

/// Disable IRQ, return if IRQ was originally enabled.
#[inline]
pub unsafe fn enter_critical() -> bool {
    let mut cpsr: u32;
    asm!(
        "mrs {}, cpsr
         cpsid i", lateout(reg) cpsr);
    (cpsr & (1 << 7)) == 0
}

#[inline]
pub unsafe fn exit_critical(enable: bool) {
    // https://stackoverflow.com/questions/40019929/temporarily-disable-interrupts-on-arm
    let mask: u32 = if enable { 1 << 7 } else { 0 };
    asm!(
        "mrs r1, cpsr
         bic r1, r1, {}
         msr cpsr_c, r1"
         , in(reg) mask, out("r1") _);
}

/// Exiting IRQ
#[inline]
pub unsafe fn exit_irq() {
    asm!("
        mrs r0, SPSR
        msr CPSR, r0
        ", out("r0") _);
}
