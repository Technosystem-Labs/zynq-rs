use crate::uart::Uart;

const UART_RATE: u32 = 115_200;
static mut UART: Option<Uart> = None;

// TODO: locking for SMP
#[doc(hidden)]
pub fn get_uart() -> &'static mut Uart {
    unsafe {
        match &mut UART {
            None => {
                let mut uart = Uart::serial(UART_RATE);
                UART = Some(uart);
                UART.as_mut().unwrap()
            }
            Some(uart) => uart,
        }
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let uart = crate::stdio::get_uart();
        write!(uart, $($arg)*);
        write!(uart, "\r\n");
        while !uart.tx_fifo_empty() {}
    })
}
