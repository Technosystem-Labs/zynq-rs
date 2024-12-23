#![no_std]

#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(naked_functions)]

pub extern crate alloc;
pub extern crate compiler_builtins;

pub mod boot;
pub mod exception_vectors;
#[cfg(feature = "panic_handler")]
mod panic;
pub mod ram;

