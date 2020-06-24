use libregister::RegisterR;
use libcortex_a9::regs::{DFSR, MPIDR};
use libboard_zynq::{println, slcr, stdio};

#[no_mangle]
pub unsafe extern "C" fn PrefetchAbort() {
    stdio::drop_uart();

    println!("PrefetchAbort");

    slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn DataAbort() {
    stdio::drop_uart();

    const CORE_MASK: u32 = 0x3;
    println!("DataAbort on core {}", MPIDR.read() & CORE_MASK);
    println!("DFSR: {:03X}", DFSR.read());

    slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    loop {}
}
