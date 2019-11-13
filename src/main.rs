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
use alloc::{vec, vec::Vec};
use core::mem::transmute;
use compiler_builtins as _;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr};
use smoltcp::iface::{NeighborCache, EthernetInterfaceBuilder};
use smoltcp::time::Instant;
use smoltcp::socket::SocketSet;

mod boot;
mod regs;
mod cortex_a9;
mod abort;
mod panic;
mod zynq;
mod stdio;
mod ram;

const HWADDR: [u8; 6] = [0, 0x23, 0xde, 0xea, 0xbe, 0xef];

pub fn main() {
    println!("Main.");

    let mut ddr = zynq::ddr::DdrRam::new();
    println!("DDR: {:?}", ddr.status());
    ddr.memtest();
    ram::init_alloc(&mut ddr);

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
    let local_addr = IpAddress::v4(192, 168, 1, 51);
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
        let timestamp = Instant::from_millis(time);

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

