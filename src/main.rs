#![no_std]
#![no_main]
#![feature(asm)]
//[feature(global_asm)]
#![feature(naked_functions)]

use core::fmt::Write;

use panic_abort as _;
use r0::zero_bss;

mod regs;
mod cortex_a9;
mod slcr;
mod uart;
use uart::Uart;
mod eth;

extern "C" {
    static mut __bss_start: u32;
    static mut __bss_end: u32;
    static mut __end: u32;
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _boot_cores() -> ! {
    use cortex_a9::{asm, regs::*};

    const CORE_MASK: u32 = 0x3;
    // End of OCM RAM
    const STACK_START: u32 = 256 << 10;

    match MPIDR.get() & CORE_MASK {
        0 => {
            SP.set(STACK_START);
            zero_bss(&mut __bss_start, &mut __bss_end);
            main();
            panic!("return from main");
        }
        _ => loop {
            // if not core0, infinitely wait for events
            asm::wfe();
        },
    }
}

fn main() {
    let mut uart = Uart::uart0();
    writeln!(uart, "Hello World\r").unwrap();

    let eth = eth::Eth::gem0();
    loop {
    }
}
