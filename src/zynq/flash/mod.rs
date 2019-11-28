//! Quad-SPI Flash Controller

use core::marker::PhantomData;
use crate::regs::{RegisterW, RegisterRW};
use super::slcr;
use super::clocks::CpuClocks;

pub mod regs;

const FLASH_BAUD_RATE: u32 = 50_000_000;

pub struct LinearAddressing;

/// Flash Interface Driver
///
/// For 2x Spansion S25FL128SAGMFIR01
pub struct Flash<MODE> {
    regs: &'static mut regs::RegisterBlock,
    _mode: PhantomData<MODE>,
}

impl Flash<()> {
    pub fn new(clock: u32) -> Self {
        Self::enable_clocks(clock);
        Self::setup_signals();
        Self::reset();

        let regs = regs::RegisterBlock::qspi();
        let mut flash = Flash { regs, _mode: PhantomData };
        flash.configure((FLASH_BAUD_RATE - 1 + clock) / FLASH_BAUD_RATE);
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
        slcr::RegisterBlock::unlocked(|slcr| {
            // 1. Configure MIO pin 1 for chip select 0 output.
            slcr.mio_pin_01.write(
                slcr::MioPin01::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );

            // Configure MIO pins 2 through 5 for I/O.
            slcr.mio_pin_02.write(
                slcr::MioPin02::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );
            slcr.mio_pin_03.write(
                slcr::MioPin03::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );
            slcr.mio_pin_04.write(
                slcr::MioPin04::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );
            slcr.mio_pin_05.write(
                slcr::MioPin05::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );

            // 3. Configure MIO pin 6 for serial clock 0 output.
            slcr.mio_pin_06.write(
                slcr::MioPin06::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );

            // Option: Add Second Device Chip Select
            // 4. Configure MIO pin 0 for chip select 1 output.
            slcr.mio_pin_00.write(
                slcr::MioPin00::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );

            // Option: Add Second Serial Clock
            // 5. Configure MIO pin 9 for serial clock 1 output.
            slcr.mio_pin_09.write(
                slcr::MioPin09::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );

            // Option: Add 4-bit Data
            // 6. Configure MIO pins 10 through 13 for I/O.
            slcr.mio_pin_10.write(
                slcr::MioPin10::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );
            slcr.mio_pin_11.write(
                slcr::MioPin11::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );
            slcr.mio_pin_12.write(
                slcr::MioPin12::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );
            slcr.mio_pin_13.write(
                slcr::MioPin13::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );

            // Option: Add Feedback Output Clock
            // 7. Configure MIO pin 8 for feedback clock.
            slcr.mio_pin_08.write(
                slcr::MioPin08::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
            );
        });
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

    fn configure(&mut self, divider: u32) {
        // for a baud_rate_div=1 LPBK_DLY_ADJ would be required
        let mut baud_rate_div = 2u32;
        while baud_rate_div < 7 && 2u32.pow(1 + baud_rate_div) < divider {
            baud_rate_div += 1;
        }

        self.regs.config.write(regs::Config::zeroed()
            .baud_rate_div(baud_rate_div as u8)
            .mode_sel(true)
            .leg_flsh(true)
            .fifo_width(0b11)
        );
    }

    pub fn linear_addressing_mode(self) -> Flash<LinearAddressing> {
        // Set manual start enable to auto mode.
        // Assert the chip select.
        self.regs.config.modify(|_, w| w
            .man_start_en(false)
            .pcs(false)
        );

        self.regs.lqspi_cfg.write(regs::LqspiCfg::zeroed()
            .inst_code(0x3)
            .u_page(false)
            .sep_bus(false)
            .two_mem(false)
            .lq_mode(true)
        );

        self.regs.enable.modify(|_, w| w.spi_en(true));

        Flash {
            regs: self.regs,
            _mode: PhantomData,
        }
    }
}

impl Flash<LinearAddressing> {
    pub fn ptr<T>(&mut self) -> *mut T {
        0xFC00_0000 as *mut _
    }
}
