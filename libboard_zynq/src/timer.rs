use embedded_hal::timer::CountDown;
use libregister::{RegisterR, RegisterW};
use void::Void;

use crate::{clocks::Clocks, mpcore};

#[derive(Clone)]
pub struct Timer<T> {
    pub timeout: T,
}

impl Timer<Milliseconds> {
    pub fn millis() -> Self {
        Self {
            timeout: Milliseconds(0),
        }
    }
}

impl Timer<Microseconds> {
    pub fn micros() -> Self {
        Self {
            timeout: Microseconds(0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Milliseconds(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct Microseconds(pub u64);

/// embedded-hal async API
impl embedded_hal::timer::CountDown for Timer<Milliseconds> {
    type Time = Milliseconds;

    fn start<T: Into<Self::Time>>(&mut self, count: T) {
        self.timeout = Milliseconds(get_ms() + count.into().0);
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if get_ms() <= self.timeout.0 {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}

impl embedded_hal::timer::CountDown for Timer<Microseconds> {
    type Time = Microseconds;

    fn start<T: Into<Self::Time>>(&mut self, count: T) {
        self.timeout = Microseconds(get_us() + count.into().0);
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if get_us() <= self.timeout.0 {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}

pub fn start() {
    let regs = mpcore::RegisterBlock::mpcore();
    // Disable
    regs.global_timer_control.write(mpcore::GlobalTimerControl::zeroed());

    // Reset counters
    regs.global_timer_counter0.write(mpcore::ValueRegister::zeroed());
    regs.global_timer_counter1.write(mpcore::ValueRegister::zeroed());

    // find a prescaler value that matches CPU speed / 2 to us
    let clocks = Clocks::get();
    let mut prescaler = clocks.cpu_3x2x() / 1_000_000;
    while prescaler > 256 {
        prescaler /= 2;
    }

    // Start
    regs.global_timer_control.write(
        mpcore::GlobalTimerControl::zeroed()
            .prescaler((prescaler - 1) as u8)
            .auto_increment_mode(true)
            .timer_enable(true),
    );
}

/// read the raw counter value
pub fn get_counter() -> u64 {
    let regs = mpcore::RegisterBlock::mpcore();
    loop {
        let c1_pre = regs.global_timer_counter1.read().value();
        let c0 = regs.global_timer_counter0.read().value();
        let c1_post = regs.global_timer_counter1.read().value();

        if c1_pre == c1_post {
            return ((c1_pre as u64) << 32) | (c0 as u64);
        }
        // retry if c0 has wrapped while reading.
    }
}

/// read and convert to time
pub fn get_ms() -> u64 {
    let regs = mpcore::RegisterBlock::mpcore();
    let prescaler = regs.global_timer_control.read().prescaler() as u64;
    let clocks = Clocks::get();

    get_counter() * (prescaler + 1) / (clocks.cpu_3x2x() as u64 / 1000)
}

/// read with high precision
pub fn get_us() -> u64 {
    let regs = mpcore::RegisterBlock::mpcore();
    let prescaler = regs.global_timer_control.read().prescaler() as u64;
    let clocks = Clocks::get();

    1_000_000 * get_counter() * (prescaler + 1) / clocks.cpu_3x2x() as u64
}

pub fn delay_ms(ms: u64) {
    let mut timer = Timer::millis();
    timer.start(Milliseconds(ms));
    nb::block!(timer.wait()).unwrap();
}

pub fn delay_us(us: u64) {
    let mut timer = Timer::micros();
    timer.start(Microseconds(us));
    nb::block!(timer.wait()).unwrap();
}
