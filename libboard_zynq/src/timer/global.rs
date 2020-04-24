use void::Void;
use libregister::{RegisterR, RegisterW};
use crate::{
    clocks::Clocks,
    mpcore,
    time::Milliseconds,
};

/// "uptime"
pub struct GlobalTimer {
    regs: &'static mut mpcore::RegisterBlock,
}

impl GlobalTimer {
    pub fn new() -> GlobalTimer {
        let regs = mpcore::RegisterBlock::new();
        GlobalTimer { regs }
    }

    pub fn reset(&mut self) {
        // Disable
        self.regs.global_timer_control.write(
            mpcore::GlobalTimerControl::zeroed()
        );

        // Reset counters
        self.regs.global_timer_counter0.write(
            mpcore::ValueRegister::zeroed()
        );
        self.regs.global_timer_counter1.write(
            mpcore::ValueRegister::zeroed()
        );

        // Start
        self.regs.global_timer_control.write(
            mpcore::GlobalTimerControl::zeroed()
                // maximum prescaler is still enough for millisecond
                // precision while overflowing after centuries.
                .prescaler(255)
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

    /// return a handle that has implements
    /// `embedded_hal::timer::CountDown`
    pub fn countdown(&self) -> CountDown {
        CountDown {
            timer: &self,
            timeout: Milliseconds(0),
        }
    }
}

#[derive(Clone)]
pub struct CountDown<'a> {
    timer: &'a GlobalTimer,
    timeout: Milliseconds,
}

impl embedded_hal::timer::CountDown for CountDown<'_> {
    type Time = Milliseconds;

    fn start<T: Into<Self::Time>>(&mut self, count: T) {
        self.timeout = count.into();
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if self.timer.get_time() < self.timeout {
            Err(nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }
}
