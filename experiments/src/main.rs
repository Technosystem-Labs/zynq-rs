#![no_std]
#![no_main]

use core::mem::transmute;
use libcortex_a9::mutex::Mutex;
use libboard_zynq::{
    print, println,
    self as zynq, clocks::Clocks, clocks::source::{ClockSource, ArmPll, IoPll},
    smoltcp::{
        wire::{EthernetAddress, IpAddress, IpCidr},
        iface::{NeighborCache, EthernetInterfaceBuilder},
        time::Instant,
        socket::SocketSet,
        socket::{TcpSocket, TcpSocketBuffer},
    },
};
use libsupport_zynq::{
    ram, alloc::{vec, vec::Vec},
    boot,
};
use libasync::task;

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
    IoPll::setup(700_000_000);
    libboard_zynq::stdio::drop_uart();
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

    for _ in 0..0x1000000 {
        let mut l = SHARED.lock();
        *l += 1;
    }
    while !*DONE.lock() {
        let x = { *SHARED.lock() };
        println!("shared: {:08X}", x);
    }
    let x = { *SHARED.lock() };
    println!("done shared: {:08X}", x);

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

    // taken from example code for smoltcp
    let mut tcp_server_rx_data = vec![0; 512 * 1024];
    let mut tcp_server_tx_data = vec![0; 512 * 1024];
    let tcp_rx_buffer = TcpSocketBuffer::new(&mut tcp_server_rx_data[..]);
    let tcp_tx_buffer = TcpSocketBuffer::new(&mut tcp_server_tx_data[..]);
    let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
    let tcp_handle = sockets.add(tcp_socket);
    /// `chargen`
    const TCP_PORT: u16 = 19;

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

        // (mostly) taken from smoltcp example: TCP echo server
        let mut socket = sockets.get::<TcpSocket>(tcp_handle);
        if !socket.is_open() {
            socket.listen(TCP_PORT).unwrap()
        }
        if socket.may_recv() && socket.can_send() {
            socket.recv(|buf| {
                let len = buf.len().min(4096);
                let buffer = buf[..len].iter().cloned().collect::<Vec<_>>();
                (len, buffer)
            })
                .and_then(|buffer| socket.send_slice(&buffer[..]))
                .map(|_| {})
                .unwrap_or_else(|e| println!("tcp: {:?}", e));

        }
    }

    // #[allow(unreachable_code)]
    // drop(tx_descs);
    // #[allow(unreachable_code)]
    // drop(tx_buffers);
}

static SHARED: Mutex<u32> = Mutex::new(0);
static DONE: Mutex<bool> = Mutex::new(false);

#[no_mangle]
pub fn main_core1() {
    println!("Hello from core1!");
    for _ in 0..0x1000000 {
        let mut l = SHARED.lock();
        *l += 1;
    }
    println!("core1 done!");
    *DONE.lock() = true;

    loop {}
}
