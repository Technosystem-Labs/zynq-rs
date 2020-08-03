use libregister::{RegisterR, RegisterW};
use libcortex_a9::regs::{DFSR, MPIDR, SP};
use libcortex_a9::asm;
use libboard_zynq::{println, stdio, gic, mpcore};

extern "C" {
    fn main_core1();
    static mut __stack1_start: u32;
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn UndefinedInstruction() {
    stdio::drop_uart();
    println!("UndefinedInstruction");
    loop {}
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn SoftwareInterrupt() {
    stdio::drop_uart();
    println!("SoftwareInterrupt");
    loop {}
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn PrefetchAbort() {
    stdio::drop_uart();
    println!("PrefetchAbort");
    loop {}
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn DataAbort() {
    stdio::drop_uart();

    println!("DataAbort on core {}", MPIDR.read().cpu_id());
    println!("DFSR: {:03X}", DFSR.read());

    loop {}
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn ReservedException() {
    stdio::drop_uart();
    println!("ReservedException");
    loop {}
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn IRQ() {
    if MPIDR.read().cpu_id() == 1{
        let mpcore = mpcore::RegisterBlock::new();
        let mut gic = gic::InterruptController::new(mpcore);
        let id = gic.get_interrupt_id();
        if id.0 == 0 {
            gic.end_interrupt(id);
            asm::exit_irq();
            SP.write(&mut __stack1_start as *mut _ as u32);
            asm::enable_irq();
            main_core1();
        }
    }
    stdio::drop_uart();
    println!("IRQ");
    loop {}
}

#[link_section = ".text.boot"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn FIQ() {
    stdio::drop_uart();
    println!("FIQ");
    loop {}
}
