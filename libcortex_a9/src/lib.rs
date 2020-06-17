#![no_std]
#![feature(llvm_asm, global_asm)]
#![feature(never_type)]

extern crate alloc;

pub mod asm;
pub mod regs;
pub mod cache;
pub mod mmu;
pub mod mutex;
pub mod sync_channel;
mod uncached;
pub use uncached::UncachedSlice;

global_asm!(include_str!("exceptions.s"));
