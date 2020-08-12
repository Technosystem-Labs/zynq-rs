#![no_std]
#![no_main]
#![feature(const_in_array_repeat_expressions)]
#![feature(naked_functions)]

extern crate alloc;

use alloc::collections::BTreeMap;
use libasync::{
    delay,
    smoltcp::{Sockets, TcpStream},
    task,
};
use libboard_zynq::{
    self as zynq,
    clocks::source::{ArmPll, ClockSource, IoPll},
    clocks::Clocks,
    print, println, stdio,
    mpcore,
    gic,
    smoltcp::{
        iface::{EthernetInterfaceBuilder, NeighborCache, Routes},
        time::Instant,
        wire::{EthernetAddress, IpAddress, IpCidr},
    },
    time::Milliseconds,
};
use libcortex_a9::{
    mutex::Mutex,
    sync_channel::{Sender, Receiver},
    sync_channel,
    regs::{MPIDR, SP},
    spin_lock_yield, notify_spin_lock,
    asm
};
use libregister::{RegisterR, RegisterW};
use libsupport_zynq::{
    boot, ram,
};
use log::{info, warn};
use core::sync::atomic::{AtomicBool, Ordering};

const HWADDR: [u8; 6] = [0, 0x23, 0xde, 0xea, 0xbe, 0xef];

static mut CORE1_REQ: (Sender<usize>, Receiver<usize>) = sync_channel!(usize, 10);
static mut CORE1_RES: (Sender<usize>, Receiver<usize>) = sync_channel!(usize, 10);

extern "C" {
    static mut __stack1_start: u32;
}

static CORE1_RESTART: AtomicBool = AtomicBool::new(false);

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn IRQ() {
    if MPIDR.read().cpu_id() == 1{
        let mpcore = mpcore::RegisterBlock::new();
        let mut gic = gic::InterruptController::gic(mpcore);
        let id = gic.get_interrupt_id();
        if id.0 == 0 {
            gic.end_interrupt(id);
            asm::exit_irq();
            SP.write(&mut __stack1_start as *mut _ as u32);
            asm::enable_irq();
            CORE1_RESTART.store(false, Ordering::Relaxed);
            notify_spin_lock();
            main_core1();
        }
    }
    stdio::drop_uart();
    println!("IRQ");
    loop {}
}

pub fn restart_core1() {
    let mut interrupt_controller = gic::InterruptController::gic(mpcore::RegisterBlock::new());
    CORE1_RESTART.store(true, Ordering::Relaxed);
    interrupt_controller.send_sgi(gic::InterruptId(0), gic::CPUCore::Core1.into());
    while CORE1_RESTART.load(Ordering::Relaxed) {
        spin_lock_yield();
    }
}

#[no_mangle]
pub fn main_core0() {
    // zynq::clocks::CpuClocks::enable_io(1_250_000_000);
    println!("\nzc706 main");
    let mut interrupt_controller = gic::InterruptController::gic(mpcore::RegisterBlock::new());
    interrupt_controller.enable_interrupts();
    // ps7_init::apply();
    libboard_zynq::stdio::drop_uart();

    libboard_zynq::logger::init().unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    info!(
        "Boot mode: {:?}",
        zynq::slcr::RegisterBlock::new()
            .boot_mode
            .read()
            .boot_mode_pins()
    );

    #[cfg(feature = "target_zc706")]
    const CPU_FREQ: u32 = 800_000_000;
    #[cfg(feature = "target_cora_z7_10")]
    const CPU_FREQ: u32 = 650_000_000;

    info!("Setup clock sources...");
    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    #[cfg(feature = "target_zc706")]
    {
        IoPll::setup(1_000_000_000);
        libboard_zynq::stdio::drop_uart();
    }
    #[cfg(feature = "target_cora_z7_10")]
    {
        IoPll::setup(1_000_000_000);
        libboard_zynq::stdio::drop_uart();
    }
    info!("PLLs set up");
    let clocks = zynq::clocks::Clocks::get();
    info!(
        "CPU Clocks: {}/{}/{}/{}",
        clocks.cpu_6x4x(),
        clocks.cpu_3x2x(),
        clocks.cpu_2x(),
        clocks.cpu_1x()
    );

    let mut flash = zynq::flash::Flash::flash(200_000_000).linear_addressing_mode();
    let flash_ram: &[u8] = unsafe { core::slice::from_raw_parts(flash.ptr(), flash.size()) };
    for i in 0..=1 {
        print!("Flash {}:", i);
        for b in &flash_ram[(i * 16 * 1024 * 1024)..][..128] {
            print!(" {:02X}", *b);
        }
        println!("");
    }
    let _flash = flash.stop();

    let timer = libboard_zynq::timer::GlobalTimer::start();

    let mut ddr = zynq::ddr::DdrRam::ddrram();
    #[cfg(not(feature = "target_zc706"))]
    ddr.memtest();
    ram::init_alloc_ddr(&mut ddr);

    #[cfg(dev)]
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
            flash_io.program(0, [0x23054223; 0x100 >> 2].iter().cloned());
        });

        flash = flash_io.stop();
    }

    boot::Core1::start(false);

    let core1_req = unsafe { &mut CORE1_REQ.0 };
    let core1_res = unsafe { &mut CORE1_RES.1 };
    task::block_on(async {
        for i in 0..10 {
            restart_core1();
            core1_req.async_send(i).await;
            let j = core1_res.async_recv().await;
            println!("{} -> {}", i, j);
        }
    });
    unsafe {
        core1_req.drop_elements();
    }
    
    // Test I2C
    #[cfg(feature = "target_zc706")]
    {
        let mut i2c = zynq::i2c::I2c::i2c0();
        i2c.init();
        println!("I2C bit-banging enabled");
        let mut eeprom = zynq::i2c::eeprom::EEPROM::new(&mut i2c, 16);
        // Write to 0x00 and 0x08
        let eeprom_buffer: [u8; 22] = [
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 
            0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 
            0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01,
        ];
        eeprom.write(0x00, &eeprom_buffer[0..6]);
        eeprom.write(0x08, &eeprom_buffer[6..22]);
        println!("Data written to EEPROM");
        let mut eeprom_buffer = [0u8; 24];
        // Read from 0x00
        eeprom.read(0x00, &mut eeprom_buffer);
        print!("Data read from EEPROM @ 0x00: (hex) ");
        for i in 0..6 {
            print!("{:02x} ", eeprom_buffer[i]);
        }
        println!("");
        // Read from 0x08
        eeprom.read(0x08, &mut eeprom_buffer);
        print!("Data read from EEPROM @ 0x08: (hex) ");
        for i in 0..16 {
            print!("{:02x} ", eeprom_buffer[i]);
        }
        println!("");
    }

    let eth = zynq::eth::Eth::eth0(HWADDR.clone());
    println!("Eth on");

    const RX_LEN: usize = 4096;
    // Number of transmission buffers (minimum is two because with
    // one, duplicate packet transmission occurs)
    const TX_LEN: usize = 4096;
    let eth = eth.start_rx(RX_LEN);
    let mut eth = eth.start_tx(TX_LEN);

    let ethernet_addr = EthernetAddress(HWADDR);
    // IP stack
    let local_addr = IpAddress::v4(192, 168, 1, 51);
    let mut ip_addrs = [IpCidr::new(local_addr, 24)];
    let routes = Routes::new(BTreeMap::new());
    let neighbor_cache = NeighborCache::new(BTreeMap::new());
    let mut iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .neighbor_cache(neighbor_cache)
        .finalize();

    Sockets::init(32);

    const TCP_PORT: u16 = 19;
    // (rx, tx)
    let stats = alloc::rc::Rc::new(core::cell::RefCell::new((0, 0)));
    let stats_tx = stats.clone();
    task::spawn(async move {
        while let Ok(stream) = TcpStream::accept(TCP_PORT, 0x10_0000, 0x10_0000).await {
            let stats_tx = stats_tx.clone();
            task::spawn(async move {
                let tx_data = (0..=255).take(4096).collect::<alloc::vec::Vec<u8>>();
                loop {
                    // const CHUNK_SIZE: usize = 65536;
                    // match stream.send((0..=255).cycle().take(CHUNK_SIZE)).await {
                    match stream.send_slice(&tx_data[..]).await {
                        Ok(len) => stats_tx.borrow_mut().1 += tx_data.len(), //CHUNK_SIZE,
                        Err(e) => {
                            warn!("tx: {:?}", e);
                            break
                        }
                    }
                }
            });
        }
    });
    let stats_rx = stats.clone();
    task::spawn(async move {
        while let Ok(stream) = TcpStream::accept(TCP_PORT+1, 0x10_0000, 0x10_0000).await {
            let stats_rx = stats_rx.clone();
            task::spawn(async move {
                loop {
                    match stream.recv(|buf| (buf.len(), buf.len())).await {
                        Ok(len) => stats_rx.borrow_mut().0 += len,
                        Err(e) => {
                            warn!("rx: {:?}", e);
                            break
                        }
                    }
                }
            });
        }
    });

    let mut countdown = timer.countdown();
    task::spawn(async move {
        loop {
            delay(&mut countdown, Milliseconds(1000)).await;

            let timestamp = timer.get_us().0;
            let seconds = timestamp / 1_000_000;
            let micros = timestamp % 1_000_000;
            let (rx, tx) = {
                let mut stats = stats.borrow_mut();
                let result = *stats;
                *stats = (0, 0);
                result
            };
            info!("time: {:6}.{:06}s, rx: {}k/s, tx: {}k/s", seconds, micros, rx / 1024, tx / 1024);
        }
    });

    Sockets::run(&mut iface, || {
        Instant::from_millis(timer.get_time().0 as i64)
    })
}

static DONE: Mutex<bool> = Mutex::new(false);

#[no_mangle]
pub fn main_core1() {
    println!("Hello from core1!");
    let mut interrupt_controller = gic::InterruptController::gic(mpcore::RegisterBlock::new());
    interrupt_controller.enable_interrupts();
    let req = unsafe { &mut CORE1_REQ.1 };
    let res = unsafe { &mut CORE1_RES.0 };

    for i in req {
        res.send(i * i);
    }

    println!("core1 done!");
    *DONE.lock() = true;

    loop {}
}
