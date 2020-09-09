use core::ops::{Deref, DerefMut};
use libcortex_a9::{asm, mutex::{Mutex, MutexGuard}};
use crate::uart::Uart;

const UART_RATE: u32 = 115_200;
static mut UART: Mutex<LazyUart> = Mutex::new(LazyUart::Uninitialized);

#[doc(hidden)]
pub fn get_uart<'a>() -> MutexGuard<'a, LazyUart> {
    unsafe { UART.lock() }
}

/// Deinitialize so that the Uart will be reinitialized on next
/// output.
///
/// Delays so that an outstanding transmission can finish.
pub fn drop_uart() {
    for _ in 0..1_000_000 {
        asm::nop();
    }

    unsafe { UART = Mutex::new(LazyUart::Uninitialized); }
}

/// Initializes the UART on first use through `.deref_mut()` for debug
/// output through the `print!` and `println!` macros.
pub enum LazyUart {
    Uninitialized,
    Initialized(Uart),
}

impl Deref for LazyUart {
    type Target = Uart;
    fn deref(&self) -> &Uart {
        match self {
            LazyUart::Uninitialized =>
                panic!("stdio not initialized!"),
            LazyUart::Initialized(uart) =>
                uart,
        }
    }
}

impl DerefMut for LazyUart {
    fn deref_mut(&mut self) -> &mut Uart {
        match self {
            LazyUart::Uninitialized => {
                #[cfg(any(feature = "target_cora_z7_10", feature = "target_redpitaya"))]
                let uart = Uart::uart0(UART_RATE);
                #[cfg(feature = "target_zc706")]
                let uart = Uart::uart1(UART_RATE);
                *self = LazyUart::Initialized(uart);
                self
            }
            LazyUart::Initialized(uart) =>
                uart,
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut uart = $crate::stdio::get_uart();
        let _ = write!(uart, $($arg)*);
    })
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut uart = $crate::stdio::get_uart();
        let _ = write!(uart, $($arg)*);
        let _ = write!(uart, "\n");
        // flush after the newline
        while !uart.tx_idle() {}
    })
}
