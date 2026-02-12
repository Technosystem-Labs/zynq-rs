#![no_std]
#![no_main]

extern crate alloc;
extern crate log;

mod serial_sd;

use libboard_zynq::{self as zynq,
                    clocks::{Clocks,
                             source::{ArmPll, ClockSource, IoPll}},
                    logger, println, timer};
use libsupport_zynq::ram;
use log::info;

#[no_mangle]
pub fn main_core0() {
    timer::start();
    logger::init().unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    println!(
        r#"

                     __________   __
                    / ___/__  /  / /
                    \__ \  / /  / /
                   ___/ / / /__/ /___
                  /____/ /____/_____/

                 (C) 2020-2025 M-Labs
"#
    );
    info!("Simple Zynq Loader starting...");

    #[cfg(not(any(feature = "target_kasli_soc", feature = "target_ebaz4205")))]
    const CPU_FREQ: u32 = 800_000_000;

    #[cfg(feature = "target_kasli_soc")]
    const CPU_FREQ: u32 = 1_000_000_000;

    #[cfg(feature = "target_ebaz4205")]
    const CPU_FREQ: u32 = 666_666_666;

    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    IoPll::setup(1_000_000_000);
    libboard_zynq::stdio::drop_uart(); // reinitialize UART after clocking change
    let _ddr = zynq::ddr::DdrRam::ddrram();
    ram::init_alloc_core0();

    serial_sd::run();
}

#[no_mangle]
pub fn main_core1() {
    panic!("core1 started but should not have");
}
