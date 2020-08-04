#![no_std]
#![feature(llvm_asm, global_asm)]
#![feature(never_type)]
#![feature(const_fn)]

extern crate alloc;

pub mod asm;
pub mod regs;
pub mod cache;
pub mod mmu;
pub mod mutex;
pub mod sync_channel;
pub mod semaphore;
mod uncached;
mod fpu;
pub use uncached::UncachedSlice;
pub use fpu::enable_fpu;

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

