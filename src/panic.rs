use crate::{println, zynq};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\nPanic: {}", info);

    zynq::slcr::RegisterBlock::unlocked(|slcr| slcr.soft_reset());
    loop {}
}
