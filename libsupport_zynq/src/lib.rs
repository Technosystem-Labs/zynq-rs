#![no_std]

#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(naked_functions)]
#![feature(strict_provenance)]
#![feature(raw_ref_op)]

pub extern crate alloc;

pub mod boot;
pub mod exception_vectors;
#[cfg(feature = "panic_handler")]
mod panic;
pub mod ram;

