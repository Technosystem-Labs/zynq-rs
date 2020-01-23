use libboard_zynq::{println, stdio};

#[no_mangle]
pub unsafe extern "C" fn PrefetchAbort() {
    stdio::drop_uart();

    println!("PrefetchAbort");
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn DataAbort() {
    stdio::drop_uart();

    println!("DataAbort");
    loop {}
}
