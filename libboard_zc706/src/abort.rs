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

    println!("DataAbort");

    slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    loop {}
}
