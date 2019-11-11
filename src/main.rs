#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(compiler_builtins_lib)]
#![feature(never_type)]
#![feature(alloc_error_handler)]
// TODO: disallow unused/dead_code when code moves into a lib crate
#![allow(dead_code)]

extern crate alloc;
use alloc::{vec, vec::Vec, alloc::Layout};
use core::alloc::GlobalAlloc;
use core::ptr::NonNull;
use core::cell::RefCell;
use core::mem::{uninitialized, transmute};
use r0::zero_bss;
use compiler_builtins as _;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};
use smoltcp::iface::{NeighborCache, EthernetInterfaceBuilder, EthernetInterface};
use smoltcp::time::Instant;
use smoltcp::socket::SocketSet;
use linked_list_allocator::Heap;

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

    let mut ddr = zynq::ddr::DdrRam::new();
    println!("DDR: {:?}", ddr.status());
    ddr.memtest();

    unsafe {
        ALLOCATOR.0.borrow_mut()
            .init(ddr.ptr::<u8>() as usize, ddr.size());
    }

    let eth = zynq::eth::Eth::default(HWADDR.clone());
    println!("Eth on");

    const RX_LEN: usize = 8;
    let mut rx_descs = (0..RX_LEN)
        .map(|_| zynq::eth::rx::DescEntry::zeroed())
        .collect::<Vec<_>>();
    let mut rx_buffers = vec![[0u8; zynq::eth::MTU]; RX_LEN];
    // Number of transmission buffers (minimum is two because with
    // one, duplicate packet transmission occurs)
    const TX_LEN: usize = 8;
    let mut tx_descs = (0..TX_LEN)
        .map(|_| zynq::eth::tx::DescEntry::zeroed())
        .collect::<Vec<_>>();
    let mut tx_buffers = vec![[0u8; zynq::eth::MTU]; TX_LEN];
    let eth = eth.start_rx(&mut rx_descs, &mut rx_buffers);
    //let mut eth = eth.start_tx(&mut tx_descs, &mut tx_buffers);
    let mut eth = eth.start_tx(
        // HACK
        unsafe { transmute(tx_descs.as_mut_slice()) },
        unsafe { transmute(tx_buffers.as_mut_slice()) },
    );

    let ethernet_addr = EthernetAddress(HWADDR);
    // IP stack
    let local_addr = IpAddress::v4(192, 168, 1, 28);
    let mut ip_addrs = [IpCidr::new(local_addr, 24)];
    let mut neighbor_storage = vec![None; 256];
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

#[global_allocator]
static ALLOCATOR: HeapAlloc = HeapAlloc(RefCell::new(Heap::empty()));

/// LockedHeap doesn't locking properly
struct HeapAlloc(RefCell<Heap>);

/// FIXME: unsound; lock properly
unsafe impl Sync for HeapAlloc {}

unsafe impl GlobalAlloc for HeapAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.borrow_mut()
            .allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.borrow_mut()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    panic!("alloc_error")
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
