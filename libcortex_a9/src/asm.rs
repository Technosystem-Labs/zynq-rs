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

/// Enable IRQ
#[inline]
pub unsafe fn enable_irq() {
    llvm_asm!("cpsie i":::: "volatile");
}

/// Disable IRQ, return if IRQ was originally enabled.
#[inline]
pub unsafe fn enter_critical() -> bool {
    let mut cpsr: u32;
    llvm_asm!(
        "mrs $0, cpsr
         cpsid i"
         : "=r"(cpsr) ::: "volatile");
    (cpsr & (1 << 7)) == 0
}

#[inline]
pub unsafe fn exit_critical(enable: bool) {
    // https://stackoverflow.com/questions/40019929/temporarily-disable-interrupts-on-arm
    let mask: u32 = if enable {
        1 << 7
    } else {
        0
    };
    llvm_asm!(
        "mrs r1, cpsr
         bic r1, r1, $0
         msr cpsr_c, r1"
         :: "r"(mask) : "r1");
}

/// Exiting IRQ
#[inline]
pub unsafe fn exit_irq() {
    llvm_asm!("
        mrs r0, SPSR
        msr CPSR, r0
        " ::: "r0");
}
