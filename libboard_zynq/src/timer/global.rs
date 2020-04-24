use libregister::{RegisterR, RegisterW};
use crate::{
    clocks::Clocks,
    mpcore,
    time::Milliseconds,
};

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
                .prescaler(255)
                .auto_increment_mode(true)
                .timer_enable(true)
        );
    }

    pub fn get_counter(&self) -> u64 {
        loop {
            let c1 = self.regs.global_timer_counter1.read().value();
            let c0 = self.regs.global_timer_counter0.read().value();
            let c1_post = self.regs.global_timer_counter1.read().value();

            if c1 == c1_post {
                return ((c1 as u64) << 32) | (c0 as u64);
            }
        }
    }

    pub fn get_time(&self) -> Milliseconds {
        let prescaler = self.regs.global_timer_control.read().prescaler() as u64;
        let clocks = Clocks::get();

        Milliseconds(self.get_counter() * (prescaler + 1) / (clocks.cpu_3x2x() as u64 / 1000))
    }
}
