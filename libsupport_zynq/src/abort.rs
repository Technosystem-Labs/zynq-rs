use libregister::RegisterR;
use libcortex_a9::regs::{DFSR, MPIDR};
use libboard_zynq::{println, slcr, stdio};

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

    const CORE_MASK: u32 = 0x3;
    println!("DataAbort on core {}", MPIDR.read() & CORE_MASK);
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
