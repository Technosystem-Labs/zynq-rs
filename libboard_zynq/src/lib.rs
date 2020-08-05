#![no_std]

extern crate alloc;

/// Re-export so that dependents can always use the same version
pub use smoltcp;

pub mod slcr;
pub mod clocks;
pub mod uart;
pub mod devc;
pub mod stdio;
pub mod eth;
pub mod axi_hp;
pub mod axi_gp;
pub mod ddr;
pub mod mpcore;
pub mod gic;
pub mod flash;
pub mod time;
pub mod timer;
pub mod sdio;
pub mod i2c;
pub mod logger;
pub mod ps7_init;
