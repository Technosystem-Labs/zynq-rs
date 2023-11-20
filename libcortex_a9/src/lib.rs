#![no_std]
#![feature(llvm_asm, global_asm)]
#![feature(never_type)]
#![feature(const_fn)]

extern crate alloc;

pub mod asm;
pub mod cache;
mod fpu;
pub mod l2c;
pub mod mmu;
pub mod mutex;
pub mod regs;
pub mod semaphore;
pub mod sync_channel;
mod uncached;
pub use fpu::enable_fpu;
pub use uncached::UncachedSlice;

global_asm!(include_str!("exceptions.s"));

#[inline]
pub fn spin_lock_yield() {
    #[cfg(feature = "power_saving")]
    asm::wfe();
}

#[inline]
pub fn notify_spin_lock() {
    #[cfg(feature = "power_saving")]
    {
        asm::dsb();
        asm::sev();
    }
}

#[macro_export]
/// Interrupt handler, which setup the stack and preserve registers before jumping to actual interrupt handler.
/// Registers r0-r12, PC, SP and CPSR are restored after the actual handler.
/// 
/// - `name` is the name of the interrupt, should be the same as the one defined in vector table.
/// - `name2` is the name for the actual handler, should be different from name.
/// - `stack0` is the stack for the interrupt handler when called from core0.
/// - `stack1` is the stack for the interrupt handler when called from core1.
/// - `body` is the body of the actual interrupt handler, should be a normal unsafe rust function
/// body.
///
/// Note that the interrupt handler would use the same stack as normal programs by default.
macro_rules! interrupt_handler {
    ($name:ident, $name2:ident, $stack0:ident, $stack1:ident, $body:block) => {
        #[link_section = ".text.boot"]
        #[no_mangle]
        #[naked]
        pub unsafe extern "C" fn $name() -> ! {
            asm!(
                // setup SP, depending on CPU 0 or 1
                // and preserve registers
                "sub lr, lr, #4",
                "stmfd sp!, {{r0-r12, lr}}",
                "mrc p15, #0, r0, c0, c0, #5",
                concat!("movw r1, :lower16:", stringify!($stack0)),
                concat!("movt r1, :upper16:", stringify!($stack0)),
                "tst r0, #3",
                concat!("movwne r1, :lower16:", stringify!($stack1)),
                concat!("movtne r1, :upper16:", stringify!($stack1)),
                "mov r0, sp",
                "mov sp, r1",
                "push {{r0, r1}}",                                      // 2 registers are pushed to maintain 8 byte stack alignment 
                concat!("bl ", stringify!($name2)),
                "pop {{r0, r1}}",
                "mov sp, r0",
                "ldmfd sp!, {{r0-r12, pc}}^",                           // caret ^ : copy SPSR to the CPSR  
                options(noreturn)
            );
        }

        #[no_mangle]
        pub unsafe extern "C" fn $name2() $body
    };
}
