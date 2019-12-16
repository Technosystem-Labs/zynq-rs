use crate::{print, println, zynq};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("panic at ");
    if let Some(location) = info.location() {
        print!("{}:{}:{}", location.file(), location.line(), location.column());
    } else {
        print!("unknown location");
    }
    if let Some(message) = info.message() {
        println!(": {}", message);
    } else {
        println!("");
    }

    zynq::slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    loop {}
}
