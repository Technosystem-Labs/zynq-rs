#![no_std]

extern crate alloc;

/// Re-export so that dependents can always use the same version
pub use smoltcp;

pub mod axi_gp;
pub mod axi_hp;
pub mod clocks;
pub mod ddr;
pub mod devc;
#[cfg(feature = "target_kasli_soc")]
pub mod error_led;
pub mod eth;
pub mod gic;
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
pub mod i2c;
pub mod logger;
pub mod mpcore;
pub mod ps7_init;
pub mod sdio;
pub mod slcr;
pub mod stdio;
pub mod time;
pub mod timer;
pub mod uart;
