use core::ops::Add;

use libregister::{RegisterR, RegisterW};
use void::Void;

use crate::{clocks::Clocks,
            mpcore,
            time::{Microseconds, Milliseconds, TimeSource}};

/// "uptime"
#[derive(Clone, Copy)]
pub struct GlobalTimer {
    regs: &'static mpcore::RegisterBlock,
}

impl GlobalTimer {
    /// Get the potentially uninitialized timer
    pub unsafe fn get() -> GlobalTimer {
        let regs = mpcore::RegisterBlock::mpcore();
        GlobalTimer { regs }
    }

    /// Get the timer with a reset
    pub fn start() -> GlobalTimer {
        let mut regs = mpcore::RegisterBlock::mpcore();
        Self::reset(&mut regs);
        GlobalTimer { regs }
    }

    fn reset(regs: &mut mpcore::RegisterBlock) {
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
    pub fn get_counter(&self) -> u64 {
        loop {
            let c1_pre = self.regs.global_timer_counter1.read().value();
            let c0 = self.regs.global_timer_counter0.read().value();
            let c1_post = self.regs.global_timer_counter1.read().value();

            if c1_pre == c1_post {
                return ((c1_pre as u64) << 32) | (c0 as u64);
            }
            // retry if c0 has wrapped while reading.
        }
    }

    /// read and convert to time
    pub fn get_time(&self) -> Milliseconds {
        let prescaler = self.regs.global_timer_control.read().prescaler() as u64;
        let clocks = Clocks::get();

        Milliseconds(self.get_counter() * (prescaler + 1) / (clocks.cpu_3x2x() as u64 / 1000))
    }

    /// read with high precision
    pub fn get_us(&self) -> Microseconds {
        let prescaler = self.regs.global_timer_control.read().prescaler() as u64;
        let clocks = Clocks::get();

        Microseconds(1_000_000 * self.get_counter() * (prescaler + 1) / clocks.cpu_3x2x() as u64)
    }

    /// return a handle that has implements
    /// `embedded_hal::timer::CountDown`
    pub fn countdown<U>(&self) -> CountDown<U>
    where Self: TimeSource<U> {
        CountDown {
            timer: self.clone(),
            timeout: self.now(),
        }
    }
}

impl TimeSource<Milliseconds> for GlobalTimer {
    fn now(&self) -> Milliseconds {
        self.get_time()
    }
}

impl TimeSource<Microseconds> for GlobalTimer {
    fn now(&self) -> Microseconds {
        self.get_us()
    }
}

#[derive(Clone)]
pub struct CountDown<U> {
    timer: GlobalTimer,
    timeout: U,
}

/// embedded-hal async API
impl<U: Add<Output = U> + PartialOrd> embedded_hal::timer::CountDown for CountDown<U>
where GlobalTimer: TimeSource<U>
{
    type Time = U;

    fn start<T: Into<Self::Time>>(&mut self, count: T) {
        self.timeout = self.timer.now() + count.into();
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.timer.now() <= self.timeout {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}

impl<U: PartialOrd> CountDown<U>
where GlobalTimer: TimeSource<U>
{
    pub fn waiting(&self) -> bool {
        self.timer.now() <= self.timeout
    }
}

/// embedded-hal sync API
impl embedded_hal::blocking::delay::DelayMs<u64> for GlobalTimer {
    fn delay_ms(&mut self, ms: u64) {
        use embedded_hal::timer::CountDown;

        let mut countdown = self.countdown::<Milliseconds>();
        countdown.start(Milliseconds(ms));
        nb::block!(countdown.wait()).unwrap();
    }
}

/// embedded-hal sync API
impl embedded_hal::blocking::delay::DelayUs<u64> for GlobalTimer {
    fn delay_us(&mut self, us: u64) {
        use embedded_hal::timer::CountDown;

        let mut countdown = self.countdown::<Microseconds>();
        countdown.start(Microseconds(us));
        nb::block!(countdown.wait()).unwrap();
    }
}
