use void::Void;
use libregister::{RegisterR, RegisterW};
use crate::{
    clocks::Clocks,
    mpcore,
    time::Milliseconds,
};

/// "uptime"
#[derive(Clone, Copy)]
pub struct GlobalTimer {
    regs: &'static mpcore::RegisterBlock,
}

impl GlobalTimer {
    /// Get the potentially uninitialized timer
    pub unsafe fn get() -> GlobalTimer {
        let mut regs = mpcore::RegisterBlock::new();
        GlobalTimer { regs }
    }

    /// Get the timer with a reset
    pub fn start() -> GlobalTimer {
        let mut regs = mpcore::RegisterBlock::new();
        Self::reset(&mut regs);
        GlobalTimer { regs }
    }

    fn reset(regs: &mut mpcore::RegisterBlock) {
        // Disable
        regs.global_timer_control.write(
            mpcore::GlobalTimerControl::zeroed()
        );

        // Reset counters
        regs.global_timer_counter0.write(
            mpcore::ValueRegister::zeroed()
        );
        regs.global_timer_counter1.write(
            mpcore::ValueRegister::zeroed()
        );

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
                .timer_enable(true)
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
    pub fn get_us(&self) -> u64 {
        let prescaler = self.regs.global_timer_control.read().prescaler() as u64;
        let clocks = Clocks::get();

        1_000_000 * self.get_counter() * (prescaler + 1) / clocks.cpu_3x2x() as u64
    }

    /// return a handle that has implements
    /// `embedded_hal::timer::CountDown`
    pub fn countdown(&self) -> CountDown {
        CountDown {
            timer: self.clone(),
            timeout: Milliseconds(0),
        }
    }
}

#[derive(Clone)]
pub struct CountDown {
    timer: GlobalTimer,
    timeout: Milliseconds,
}

impl embedded_hal::timer::CountDown for CountDown {
    type Time = Milliseconds;

    fn start<T: Into<Self::Time>>(&mut self, count: T) {
        self.timeout = self.timer.get_time() + count.into();
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.timer.get_time() < self.timeout {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}
