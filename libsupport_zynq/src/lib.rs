#![no_std]

#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

pub extern crate alloc;
pub extern crate compiler_builtins;

pub mod boot;
mod abort;
#[cfg(feature = "panic_handler")]
mod panic;
pub mod ram;
