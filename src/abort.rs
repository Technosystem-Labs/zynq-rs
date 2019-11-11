use crate::println;

#[no_mangle]
pub unsafe extern "C" fn PrefetchAbort() {
    println!("PrefetchAbort");
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn DataAbort() {
    println!("DataAbort");
    loop {}
}
