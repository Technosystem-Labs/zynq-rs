#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(compiler_builtins_lib)]
#![feature(never_type)]

use core::mem::uninitialized;

use r0::zero_bss;
use compiler_builtins as _;

mod regs;
mod cortex_a9;
mod slcr;
mod uart;
mod stdio;
mod eth;

use crate::regs::{RegisterR, RegisterW};
use crate::cortex_a9::{asm, regs::*, mmu};

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

    match MPIDR.read() & CORE_MASK {
        0 => {
            SP.write(&mut __stack_start as *mut _ as u32);
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

    let mmu_table = mmu::L1Table::get()
        .setup_flat_layout();
    mmu::with_mmu(mmu_table, || {
        main();
        panic!("return from main");
    });
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
}

fn main() {
    println!("Main.");

    let mut eth = eth::Eth::default([0x0, 0x17, 0xde, 0xea, 0xbe, 0xef]);
    println!("Eth on");
    match eth::phy::Phy::find(&mut eth) {
        Some((addr, phy)) => {
            println!("Found {} PHY at addr {}", phy.name(), addr);
        }
        None => {
            use eth::phy::PhyAccess;
            for addr in 1..32 {
                match eth::phy::id::identify_phy(&mut eth, addr) {
                    Some(identifier) => {
                        println!("phy {}: {:?}", addr, identifier);
                    }
                    None => {}
                }
            }
        }
    }

    let mut rx_buffers = [[0u8; 1536]; eth::rx::DESCS];
    let mut rx_buffer_ptrs: [&mut [u8]; eth::rx::DESCS] = unsafe {
        uninitialized()
    };
    for (i, (ptr, buf)) in rx_buffer_ptrs.iter_mut().zip(rx_buffers.iter_mut()).enumerate() {
        *ptr = buf;
    }
    let mut eth = eth.start_rx(rx_buffer_ptrs);

    loop {
        match eth.recv_next() {
            None => {}
            Some(pkt) => {
                println!("eth: received {} bytes", pkt.len());
            }
        }
    }
    panic!("End");
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\nPanic: {}", info);

    slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn PrefetchAbort() {
    panic!("PrefetchAbort");
}

#[no_mangle]
pub unsafe extern "C" fn DataAbort() {
    panic!("DataAbort");
}
