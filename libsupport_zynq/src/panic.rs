use libboard_zynq::{print, println};
#[cfg(feature = "target_kasli_soc")]
use libboard_zynq::error_led::ErrorLED;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("panic at ");
    if let Some(location) = info.location() {
        print!("{}:{}:{}", location.file(), location.line(), location.column());
    } else {
        print!("unknown location");
    }
    println!(": {}", info.message());
    
    #[cfg(feature = "target_kasli_soc")]
    {
        let mut err_led = ErrorLED::error_led();
        err_led.toggle(true);
    }
    loop {}
}
