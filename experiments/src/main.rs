#![no_std]
#![no_main]
#![feature(const_in_array_repeat_expressions)]
#![feature(naked_functions)]
#![feature(asm)]

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
    println, stdio,
    mpcore,
    gic,
    smoltcp::{
        iface::{EthernetInterfaceBuilder, NeighborCache, Routes},
        time::Instant,
        wire::{EthernetAddress, IpAddress, IpCidr},
    },
    time::Milliseconds,
};
#[cfg(feature = "target_zc706")]
use libboard_zynq::print;
use libcortex_a9::{
    mutex::Mutex,
    l2c::enable_l2_cache,
    sync_channel::{Sender, Receiver},
    sync_channel,
    regs::{MPIDR, SP},
    spin_lock_yield, notify_spin_lock,
    asm, interrupt_handler
};
use libregister::{RegisterR, RegisterW};
use libsupport_zynq::{
    boot, exception_vectors, ram,
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

interrupt_handler!(IRQ, irq, __irq_stack0_start, __irq_stack1_start, {
    let mpcore = mpcore::RegisterBlock::mpcore();
    let mut gic = gic::InterruptController::gic(mpcore);
    let id = gic.get_interrupt_id();
    match MPIDR.read().cpu_id(){
        0 => {
            if id.0 == 0 {
                println!("Interrupting core0...");
                gic.end_interrupt(id);
                return;
            }
        },
        1 => {
            if id.0 == 0 {
                gic.end_interrupt(id);
                asm::exit_irq();
                SP.write(&mut __stack1_start as *mut _ as u32);
                asm::enable_irq();
                CORE1_RESTART.store(false, Ordering::Relaxed);
                notify_spin_lock();
                main_core1();
            }
        },
        _ => {}
    }
    stdio::drop_uart();
    println!("IRQ");
    loop {}
});

pub fn restart_core1() {
    let mut interrupt_controller = gic::InterruptController::gic(mpcore::RegisterBlock::mpcore());
    CORE1_RESTART.store(true, Ordering::Relaxed);
    interrupt_controller.send_sgi(gic::InterruptId(0), gic::CPUCore::Core1.into());
    while CORE1_RESTART.load(Ordering::Relaxed) {
        spin_lock_yield();
    }
}

#[no_mangle]
pub fn main_core0() {
    exception_vectors::set_vector_table(0x0);
    // zynq::clocks::CpuClocks::enable_io(1_250_000_000);
    enable_l2_cache(0x8);
    println!("\nZynq experiments");
    let mut interrupt_controller = gic::InterruptController::gic(mpcore::RegisterBlock::mpcore());
    interrupt_controller.enable_interrupts();

    libboard_zynq::logger::init().unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    info!(
        "Boot mode: {:?}",
        zynq::slcr::RegisterBlock::slcr()
            .boot_mode
            .read()
            .boot_mode_pins()
    );

    #[cfg(any(
        feature = "target_zc706",
        feature = "target_redpitaya",
        feature = "target_kasli_soc",
    ))]
    const CPU_FREQ: u32 = 800_000_000;
    #[cfg(feature = "target_coraz7")]
    const CPU_FREQ: u32 = 650_000_000;

    info!("Setup clock sources...");
    ArmPll::setup(2 * CPU_FREQ);
    Clocks::set_cpu_freq(CPU_FREQ);
    IoPll::setup(1_000_000_000);
    libboard_zynq::stdio::drop_uart();
    info!("PLLs set up");
    let clocks = zynq::clocks::Clocks::get();
    info!(
        "CPU Clocks: {}/{}/{}/{}",
        clocks.cpu_6x4x(),
        clocks.cpu_3x2x(),
        clocks.cpu_2x(),
        clocks.cpu_1x()
    );

    let timer = libboard_zynq::timer::GlobalTimer::start();

    let mut ddr = zynq::ddr::DdrRam::ddrram();
    #[cfg(not(feature = "target_zc706"))]
    ddr.memtest();
    ram::init_alloc_ddr(&mut ddr);

    info!("Send software interrupt to core0");
    interrupt_controller.send_sgi(gic::InterruptId(0), gic::CPUCore::Core0.into());
    info!("Core0 returned from interrupt");

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
        i2c.init().unwrap();
        println!("I2C bit-banging enabled");
        let mut eeprom = zynq::i2c::eeprom::EEPROM::new(&mut i2c, 16);
        // Write to 0x00 and 0x08
        let eeprom_buffer: [u8; 22] = [
            0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
            0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01,
        ];
        eeprom.write(0x00, &eeprom_buffer[0..6]).unwrap();
        eeprom.write(0x08, &eeprom_buffer[6..22]).unwrap();
        println!("Data written to EEPROM");
        let mut eeprom_buffer = [0u8; 24];
        // Read from 0x00
        eeprom.read(0x00, &mut eeprom_buffer).unwrap();
        print!("Data read from EEPROM @ 0x00: (hex) ");
        for i in 0..6 {
            print!("{:02x} ", eeprom_buffer[i]);
        }
        println!("");
        // Read from 0x08
        eeprom.read(0x08, &mut eeprom_buffer).unwrap();
        print!("Data read from EEPROM @ 0x08: (hex) ");
        for i in 0..16 {
            print!("{:02x} ", eeprom_buffer[i]);
        }
        println!("");
    }

    #[cfg(feature = "target_kasli_soc")]
    {
        let mut err_cdwn = timer.countdown();
        let mut err_state = true;
        let mut led = zynq::error_led::ErrorLED::error_led();
        task::spawn( async move { 
            loop {
                led.toggle(err_state);
                err_state = !err_state;
                delay(&mut err_cdwn, Milliseconds(1000)).await;
            }
        });
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
                let tx_data = (0..=255).cycle().take(4096).collect::<alloc::vec::Vec<u8>>();
                loop {
                    // const CHUNK_SIZE: usize = 65536;
                    // match stream.send((0..=255).cycle().take(CHUNK_SIZE)).await {
                    match stream.send_slice(&tx_data[..]).await {
                        Ok(_len) => stats_tx.borrow_mut().1 += tx_data.len(), //CHUNK_SIZE,
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
    let mut interrupt_controller = gic::InterruptController::gic(mpcore::RegisterBlock::mpcore());
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
