//! Quad-SPI Flash Controller

use crate::regs::{RegisterW, RegisterRW};
use super::slcr;
use super::clocks::CpuClocks;

pub mod regs;

/// Flash Interface Driver
pub struct Flash {
    regs: &'static mut regs::RegisterBlock,
}

impl Flash {
    pub fn new(clock: u32) -> Self {
        Self::enable_clocks(clock);
        Self::setup_signals();
        Self::reset();

        let regs = regs::RegisterBlock::qspi();
        let mut flash = Flash { regs };
        flash.configure();
        flash
    }

    fn enable_clocks(clock: u32) {
        let io_pll = CpuClocks::get().io;
        let divisor = ((clock - 1 + io_pll) / clock)
            .max(1).min(63) as u8;

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.lqspi_clk_ctrl.write(
                slcr::LqspiClkCtrl::zeroed()
                    .src_sel(slcr::PllSource::IoPll)
                    .divisor(divisor)
                    .clkact(true)
            );
        });
    }

    fn setup_signals() {
        // TODO
    }

    fn reset() {
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.lqspi_rst_ctrl.write(
                slcr::LqspiRstCtrl::zeroed()
                    .ref_rst(true)
                    .cpu1x_rst(true)
            );
            slcr.lqspi_rst_ctrl.write(
                slcr::LqspiRstCtrl::zeroed()
            );
        });
    }

    fn configure(&mut self) {
        self.regs.config.modify(|_, w| w
            .baud_rate_div(4 /* TODO */)
            .mode_sel(true)
            .leg_flsh(true)
            .endian(false)
            .fifo_width(0b11)
        );
    }
}
