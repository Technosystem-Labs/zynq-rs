//! A logger for the `log` crate

use crate::{println, stdio, timer::GlobalTimer};

pub static LOGGER: Logger = Logger;

pub struct Logger;

pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER)
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let timestamp = unsafe {
                GlobalTimer::get()
            }.get_us();
            let seconds   = timestamp / 1_000_000;
            let micros    = timestamp % 1_000_000;

            println!("[{:6}.{:06}s] {:>5}({}): {}",
                     seconds, micros, record.level(), record.target(), record.args());
        }
    }
    fn flush(&self) {
        let uart = stdio::get_uart();
        while !uart.tx_idle() {}
    }
}
