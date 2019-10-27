#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(compiler_builtins_lib)]
#![feature(never_type)]
// TODO: disallow unused/dead_code when code moves into a lib crate
#![allow(dead_code)]

use core::mem::{uninitialized, transmute};
use r0::zero_bss;
use compiler_builtins as _;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};
use smoltcp::iface::{NeighborCache, EthernetInterfaceBuilder, EthernetInterface};
use smoltcp::time::Instant;
use smoltcp::socket::SocketSet;

mod regs;
mod cortex_a9;
mod stdio;
mod zynq;

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
    use crate::cortex_a9::cache::*;

    // Invalidate TLBs
    tlbiall();
    // Invalidate I-Cache
    iciallu();
    // Invalidate Branch Predictor Array
    bpiall();
    // Invalidate D-Cache
    //
    // NOTE: It is both faster and correct to only invalidate instead
    //       of also flush the cache (as was done before with
    //       `dccisw()`) and it is correct to perform this operation
    //       for all of the L1 data cache rather than a (previously
    //       unspecified) combination of one cache set and one cache
    //       way.
    dciall();
}

const HWADDR: [u8; 6] = [0, 0x23, 0xde, 0xea, 0xbe, 0xef];

fn main() {
    println!("Main.");

    zynq::clocks::CpuClocks::enable_ddr(1_066_666_666);
    let pll_status = zynq::slcr::RegisterBlock::new().pll_status.read();
    println!("PLLs: {}", pll_status);
    let clocks = zynq::clocks::CpuClocks::get();
    println!("Clocks: {:?}", clocks);
    println!("CPU speeds: {}/{}/{}/{} MHz",
             clocks.cpu_6x4x() / 1_000_000,
             clocks.cpu_3x2x() / 1_000_000,
             clocks.cpu_2x() / 1_000_000,
             clocks.cpu_1x() / 1_000_000);
    let mut ddr = zynq::ddr::DdrRam::new();
    println!("DDR: {:?}", ddr.status());
    ddr.memtest();

    let eth = zynq::eth::Eth::default(HWADDR.clone());
    println!("Eth on");

    const RX_LEN: usize = 2;
    let mut rx_descs: [zynq::eth::rx::DescEntry; RX_LEN] = unsafe { uninitialized() };
    let mut rx_buffers = [[0u8; zynq::eth::MTU]; RX_LEN];
    // Number of transmission buffers (minimum is two because with
    // one, duplicate packet transmission occurs)
    const TX_LEN: usize = 2;
    let mut tx_descs: [zynq::eth::tx::DescEntry; TX_LEN] = unsafe { uninitialized() };
    let mut tx_buffers = [[0u8; zynq::eth::MTU]; TX_LEN];
    let eth = eth.start_rx(&mut rx_descs, &mut rx_buffers);
    //let mut eth = eth.start_tx(&mut tx_descs, &mut tx_buffers);
    let mut eth = eth.start_tx(
        // HACK
        unsafe { transmute(tx_descs.as_mut()) },
        unsafe { transmute(tx_buffers.as_mut()) },
    );

    let ethernet_addr = EthernetAddress(HWADDR);
    // IP stack
    let local_addr = IpAddress::v4(10, 0, 0, 1);
    let mut ip_addrs = [IpCidr::new(local_addr, 24)];
    let mut neighbor_storage = [None; 16];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .neighbor_cache(neighbor_cache)
        .finalize();
    let mut sockets_storage = [
        None, None, None, None,
        None, None, None, None
    ];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);

    let mut time = 0u32;
    loop {
        time += 1;
        let timestamp = Instant::from_millis(time.into());

        match iface.poll(&mut sockets, timestamp) {
            Ok(_) => {},
            Err(e) => {
                println!("poll error: {}", e);
            }
        }

        // match eth.recv_next() {
        //     Ok(Some(pkt)) => {
        //         print!("eth: rx {} bytes", pkt.len());
        //         for b in pkt.iter() {
        //             print!(" {:02X}", b);
        //         }
        //         println!("");
        //     }
        //     Ok(None) => {}
        //     Err(e) => {
        //         println!("eth rx error: {:?}", e);
        //     }
        // }

        // match eth.send(512) {
        //     Some(mut pkt) => {
        //         let mut x = 0;
        //         for b in pkt.iter_mut() {
        //             *b = x;
        //             x += 1;
        //         }
        //         println!("eth tx {} bytes", pkt.len());
        //     }
        //     None => println!("eth tx shortage"),
        // }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\nPanic: {}", info);

    zynq::slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn PrefetchAbort() {
    println!("PrefetchAbort");
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn DataAbort() {
    println!("DataAbort");
    loop {}
}
