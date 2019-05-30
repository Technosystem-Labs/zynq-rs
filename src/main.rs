#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]

use core::fmt::Write;

use r0::zero_bss;

mod regs;
mod cortex_a9;
mod slcr;
mod uart;
use uart::Uart;
mod eth;

use crate::cortex_a9::{asm, regs::*};

extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
    static mut __stack_start: u32;
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _boot_cores() -> ! {
    const CORE_MASK: u32 = 0x3;

    match MPIDR.get() & CORE_MASK {
        0 => {
            SP.set(&mut __stack_start as *mut _ as u32);
            boot_core0();
        }
        _ => loop {
            // if not core0, infinitely wait for events
            asm::wfe();
        },
    }
}

#[naked]
#[inline(never)]
unsafe fn boot_core0() -> ! {
    l1_cache_init();
    zero_bss(&mut __bss_start, &mut __bss_end);

    main();
    panic!("return from main");
}

fn l1_cache_init() {
    // Invalidate TLBs
    tlbiall();
    // Invalidate I-Cache
    iciallu();
    // Invalidate Branch Predictor Array
    bpiall();
    // Invalidate D-Cache
    dccisw();

    // (Initialize MMU)

    // Enable I-Cache and D-Cache
    sctlr();

    // Synchronization barriers
    // Allows MMU to start
    asm::dsb();
    // Flushes pre-fetch buffer
    asm::isb();
}

const UART_RATE: u32 = 115_200;

fn main() {
    let mut uart = Uart::serial(UART_RATE);
    loop {
        for i in 0.. {
            writeln!(uart, "i={}\r", i);
        }
    }

    let eth = eth::Eth::default();
    loop {
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut uart = Uart::serial(UART_RATE);
    writeln!(uart, "\r\nPanic: {}\r", info);
    while !uart.tx_fifo_empty() {}

    slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    unreachable!()
}

#[no_mangle]
pub unsafe extern "C" fn PrefetchAbort() {
    panic!("PrefetchAbort");
}

#[no_mangle]
pub unsafe extern "C" fn DataAbort() {
    panic!("DataAbort");
}
