#![no_std]
#![no_main]

extern crate alloc;

use core::{mem::transmute, task::Poll};
use alloc::{borrow::ToOwned, collections::BTreeMap, format};
use libcortex_a9::{mutex::Mutex, sync_channel::{self, sync_channel}};
use libboard_zynq::{
    print, println,
    self as zynq, clocks::Clocks, clocks::source::{ClockSource, ArmPll, IoPll},
    smoltcp::{
        self,
        wire::{EthernetAddress, IpAddress, IpCidr},
        iface::{NeighborCache, EthernetInterfaceBuilder, Routes},
        time::Instant,
        socket::SocketSet,
        socket::{TcpSocket, TcpSocketBuffer},
    },
};
use libsupport_zynq::{
    ram, alloc::{vec, vec::Vec},
    boot,
};
use libasync::{smoltcp::{Sockets, TcpStream}, task};

mod ps7_init;

const HWADDR: [u8; 6] = [0, 0x23, 0xde, 0xea, 0xbe, 0xef];

static mut STACK_CORE1: [u32; 512] = [0; 512];

#[no_mangle]
pub fn main_core0() {
    // zynq::clocks::CpuClocks::enable_io(1_250_000_000);
    println!("\nzc706 main");
    {
        use libregister::RegisterR;
        println!("Boot mode: {:?}", zynq::slcr::RegisterBlock::new().boot_mode.read().boot_mode_pins());
    }

    #[cfg(feature = "target_zc706")]
    const CPU_FREQ: u32 = 800_000_000;
    #[cfg(feature = "target_cora_z7_10")]
    const CPU_FREQ: u32 = 650_000_000;

    println!("Setup clock sources...");
    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    #[cfg(feature = "target_zc706")]
    {
        IoPll::setup(1_000_000_000);
        libboard_zynq::stdio::drop_uart();
    }
    println!("PLLs set up");
    let clocks = zynq::clocks::Clocks::get();
    println!("CPU Clocks: {}/{}/{}/{}", clocks.cpu_6x4x(), clocks.cpu_3x2x(), clocks.cpu_2x(), clocks.cpu_1x());

    let mut flash = zynq::flash::Flash::new(200_000_000).linear_addressing_mode();
    let flash_ram: &[u8] = unsafe { core::slice::from_raw_parts(flash.ptr(), flash.size()) };
    for i in 0..=1 {
        print!("Flash {}:", i);
        for b in &flash_ram[(i * 16 * 1024 * 1024)..][..128] {
            print!(" {:02X}", *b);
        }
        println!("");
    }
    let mut flash = flash.stop();

    let mut ddr = zynq::ddr::DdrRam::new();
    #[cfg(not(feature = "target_zc706"))]
    ddr.memtest();
    ram::init_alloc(&mut ddr);

    for i in 0..=1 {
        let mut flash_io = flash.manual_mode(i);
        // println!("rdcr={:02X}", flash_io.rdcr());
        print!("Flash {} ID:", i);
        for b in flash_io.rdid() {
            print!(" {:02X}", b);
        }
        println!("");
        print!("Flash {} I/O:", i);
        for o in 0..8 {
            const CHUNK: u32 = 8;
            for b in flash_io.read(CHUNK * o, CHUNK as usize) {
                print!(" {:02X}", b);
            }
        }
        println!("");

        flash_io.dump("Read cr1", 0x35);
        flash_io.dump("Read Autoboot", 0x14);
        flash_io.dump("Read Bank", 0x16);
        flash_io.dump("DLP Bank", 0x16);
        flash_io.dump("Read ESig", 0xAB);
        flash_io.dump("OTP Read", 0x4B);
        flash_io.dump("DYB Read", 0xE0);
        flash_io.dump("PPB Read", 0xE2);
        flash_io.dump("ASP Read", 0x2B);
        flash_io.dump("Password Read", 0xE7);

        flash_io.write_enabled(|flash_io| {
            flash_io.erase(0);
        });
        flash_io.write_enabled(|flash_io| {
            flash_io.program(0, [0x23054223; (0x100 >> 2)].iter().cloned());
        });

        flash = flash_io.stop();
    }

    task::spawn(async {
        println!("outer task");
    });
    task::spawn(async {
        for i in 1..=3 {
            println!("outer task2: {}", i);
            task::r#yield().await;
        }
    });
    task::block_on(async {
        task::spawn(async {
            println!("inner task");
        });

        for i in 1..=10 {
            println!("yield {}", i);
            task::r#yield().await;
        }
    });

    let core1_stack = unsafe { &mut STACK_CORE1[..] };
    println!("{} bytes stack for core1", core1_stack.len());
    let core1 = boot::Core1::start(core1_stack);


    let (tx, mut rx) = sync_channel(1000);
    *SHARED.lock() = Some(tx);
    let mut i = 0u32;
    loop {
        let r = rx.recv();
        // println!("Recvd {}", r);
        if i != *r {
            println!("Expected {}, received {}", i, r);
        }
        if i % 100000 == 0 {
            println!("{} Ok", i);
        }

        i += 1;
    }
    core1.reset();

    libcortex_a9::asm::dsb();
    print!("Core1 stack [{:08X}..{:08X}]:", &core1.stack[0] as *const _ as u32, &core1.stack[core1.stack.len() - 1] as *const _ as u32);
    for w in core1.stack {
        print!(" {:08X}", w);
    }
    println!(".");

    let eth = zynq::eth::Eth::default(HWADDR.clone());
    println!("Eth on");

    const RX_LEN: usize = 8;
    let mut rx_descs = (0..RX_LEN)
        .map(|_| zynq::eth::rx::DescEntry::zeroed())
        .collect::<Vec<_>>();
    let mut rx_buffers = vec![zynq::eth::Buffer::new(); RX_LEN];
    // Number of transmission buffers (minimum is two because with
    // one, duplicate packet transmission occurs)
    const TX_LEN: usize = 8;
    let mut tx_descs = (0..TX_LEN)
        .map(|_| zynq::eth::tx::DescEntry::zeroed())
        .collect::<Vec<_>>();
    let mut tx_buffers = vec![zynq::eth::Buffer::new(); TX_LEN];
    let eth = eth.start_rx(&mut rx_descs, &mut rx_buffers);
    // let mut eth = eth.start_tx(&mut tx_descs, &mut tx_buffers);
    let mut eth = eth.start_tx(
        // HACK
        unsafe { transmute(tx_descs.as_mut_slice()) },
        unsafe { transmute(tx_buffers.as_mut_slice()) },
    );
    // loop {
    //     match eth.recv_next() {
    //         Ok(None) => {},
    //         Ok(Some(pkt)) => println!("received {} bytes", pkt.len()),
    //         Err(e) => println!("e: {:?}", e),
    //     }
    // }

    println!("iface...");
    let ethernet_addr = EthernetAddress(HWADDR);
    // IP stack
    let local_addr = IpAddress::v4(192, 168, 1, 51);
    let mut ip_addrs = [IpCidr::new(local_addr, 24)];
    let mut routes_storage = vec![None; 4];
    let routes = Routes::new(/*BTreeMap::new()*/ &mut routes_storage[..]);
    let mut neighbor_storage = vec![None; 256];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .neighbor_cache(neighbor_cache)
        .finalize();

    // TODO: compare with ps7_init
    
    Sockets::init(32);
    /// `chargen`
    const TCP_PORT: u16 = 19;
    async fn handle_connection(stream: TcpStream) -> smoltcp::Result<()> {
        stream.send("Enter your name: ".bytes()).await?;
        let name = stream.recv(|buf| {
            for (i, b) in buf.iter().enumerate() {
                if *b == '\n' as u8 {
                    return match core::str::from_utf8(&buf[0..i]) {
                        Ok(name) =>
                            Poll::Ready((i + 1, Some(name.to_owned()))),
                        Err(_) =>
                            Poll::Ready((i + 1, None))
                    };
                }
            }
            if buf.len() > 100 {
                // Too much input, consume all
                Poll::Ready((buf.len(), None))
            } else {
                Poll::Pending
            }
        }).await?;
        match name {
            Some(name) =>
                stream.send(format!("Hello {}!\n", name).bytes()).await?,
            None =>
                stream.send("I had trouble reading your name.\n".bytes()).await?,
        }
        stream.flush().await;
        Ok(())
    }

    TcpStream::listen(TCP_PORT, 2048, 2048, 8, |stream| async {
        handle_connection(stream)
            .await
            .map_err(|e| println!("Connection: {:?}", e));
    });

    let mut time = 0u32;
    Sockets::run(&mut iface, || {
        time += 1;
        Instant::from_millis(time)
    });
}

static SHARED: Mutex<Option<sync_channel::Sender<u32>>> = Mutex::new(None);
static DONE: Mutex<bool> = Mutex::new(false);

#[no_mangle]
pub fn main_core1() {
    println!("Hello from core1!");

    let mut tx = None;
    while tx.is_none() {
        tx = SHARED.lock().take();
    }
    println!("Core1 got tx");
    let mut tx = tx.unwrap();

    for i in 0.. {
        // println!("S {}", i);
        tx.send(i);
    }

    println!("core1 done!");
    *DONE.lock() = true;

    loop {}
}
