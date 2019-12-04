//! Quad-SPI Flash Controller

use core::marker::PhantomData;
use crate::regs::{RegisterR, RegisterW, RegisterRW};
use super::slcr;
use super::clocks::CpuClocks;

mod regs;
mod instr;

const FLASH_BAUD_RATE: u32 = 50_000_000;
const SINGLE_CAPACITY: u32 = 16 * 1024 * 1024;

pub struct LinearAddressing;
pub struct Manual;

/// Flash Interface Driver
///
/// For 2x Spansion S25FL128SAGMFIR01
pub struct Flash<MODE> {
    regs: &'static mut regs::RegisterBlock,
    _mode: PhantomData<MODE>,
}

impl<MODE> Flash<MODE> {
    fn transition<TO>(self) -> Flash<TO> {
        Flash {
            regs: self.regs,
            _mode: PhantomData,
        }
    }

    fn disable_interrupts(&mut self) {
        self.regs.intr_dis.write(
            regs::IntrDis::zeroed()
                .rx_overflow(true)
                .tx_fifo_not_full(true)
                .tx_fifo_full(true)
                .rx_fifo_not_empty(true)
                .rx_fifo_full(true)
                .tx_fifo_underflow(true)
        );
    }

    fn enable_interrupts(&mut self) {
        self.regs.intr_en.write(
            regs::IntrEn::zeroed()
                .rx_overflow(true)
                .tx_fifo_not_full(true)
                .tx_fifo_full(true)
                .rx_fifo_not_empty(true)
                .rx_fifo_full(true)
                .tx_fifo_underflow(true)
        );
    }

    fn clear_rx_fifo(&self) {
        while self.regs.intr_status.read().rx_fifo_not_empty() {
            let _ = self.regs.rx_data.read();
        }
    }

    fn clear_interrupt_status(&mut self) {
        self.regs.intr_status.write(
            regs::IntrStatus::zeroed()
                .rx_overflow(true)
                .tx_fifo_underflow(true)
        );
    }
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
        // Disable
        self.regs.enable.write(
            regs::Enable::zeroed()
        );
        self.disable_interrupts();
        self.regs.lqspi_cfg.write(
            regs::LqspiCfg::zeroed()
        );
        self.clear_rx_fifo();
        self.clear_interrupt_status();

        // for a baud_rate_div=1 LPBK_DLY_ADJ would be required
        let mut baud_rate_div = 2u32;
        while baud_rate_div < 7 && 2u32.pow(1 + baud_rate_div) < divider {
            baud_rate_div += 1;
        }

        self.regs.config.write(regs::Config::zeroed()
            .baud_rate_div(baud_rate_div as u8)
            .mode_sel(true)
            .leg_flsh(true)
            .holdb_dr(true)
            // 32 bits TX FIFO width
            .fifo_width(0b11)
        );

        // Initialize RX/TX pipes thresholds
        unsafe {
            self.regs.rx_thres.write(32);
            self.regs.tx_thres.write(1);
        }
    }

    pub fn linear_addressing_mode(self) -> Flash<LinearAddressing> {
        // Set manual start enable to auto mode.
        // Assert the chip select.
        self.regs.config.modify(|_, w| w
            .man_start_en(false)
            .pcs(false)
            .manual_cs(false)
        );

        self.regs.lqspi_cfg.write(regs::LqspiCfg::zeroed()
            // Quad I/O Fast Read
            .inst_code(0xEB)
            .mode_bits(0xFF)
            .dummy_byte(0x2)
            .mode_en(true)
            // 2 devices
            .two_mem(true)
            .u_page(false)
            // Linear Addressing Mode
            .lq_mode(true)
        );

        self.regs.enable.write(
            regs::Enable::zeroed()
                .spi_en(true)
        );

        self.transition()
    }

    pub fn manual_mode(self, chip_index: usize) -> Flash<Manual> {
        self.regs.config.modify(|_, w| w
            .man_start_en(true)
            .manual_cs(true)
        );

        self.regs.lqspi_cfg.write(regs::LqspiCfg::zeroed()
            .mode_bits(0xFF)
            .dummy_byte(0x2)
            .mode_en(true)
            // 2 devices
            .two_mem(true)
            .u_page(chip_index != 0)
            // Manual I/O mode
            .lq_mode(false)
        );

        self.regs.enable.write(
            regs::Enable::zeroed()
                .spi_en(true)
        );
        self.transition()
    }
}

impl Flash<LinearAddressing> {
    /// Stop linear addressing mode
    pub fn stop(self) -> Flash<()> {
        self.regs.enable.modify(|_, w| w.spi_en(false));
        // De-assert chip select.
        self.regs.config.modify(|_, w| w.pcs(true));

        self.transition()
    }

    pub fn ptr<T>(&mut self) -> *mut T {
        0xFC00_0000 as *mut _
    }

    pub fn size(&self) -> usize {
        2 * (SINGLE_CAPACITY as usize)
    }
}

impl Flash<Manual> {
    pub fn stop(self) -> Flash<()> {
        self.transition()
    }

    pub fn rdcr(&mut self) -> u8 {
        self.transfer(instr::RdCr)
    }

    pub fn transfer<I: instr::Instruction>(&mut self, input: I) -> I::Result {
        let mut t = Transfer::new(&mut self.regs, I::inst_code(), input.args());
        (&mut t).into()
    }

    pub fn read(&mut self, offset: u32, dest: &mut [u8]) {

        // Quad Read
        let instr = 0xEB;
        unsafe {
            self.regs.txd0.write(
                instr |
                (offset << 8)
            );
        }
        let mut n = 0;
        while !self.regs.intr_status.read().tx_fifo_full() {
            unsafe {
                self.regs.txd0.write(0);
            }
            n += 1;
        }

        self.regs.config.modify(|_, w| w
            .pcs(false)
            .man_start_com(true)
        );

        let mut offset = 0;
        while offset < dest.len() {
            while !self.regs.intr_status.read().rx_fifo_not_empty() {}

            // TODO: drops data?
            let rx = self.regs.rx_data.read();
            if offset < dest.len() {
                dest[offset] = rx as u8;
            }
            offset += 1;
            if offset < dest.len() {
                dest[offset] = (rx >> 8) as u8;
            }
            offset += 1;
            if offset < dest.len() {
                dest[offset] = (rx >> 16) as u8;
            }
            offset += 1;
            if offset < dest.len() {
                dest[offset] = (rx << 24) as u8;
            }
            offset += 1;

            // Output dummy byte to generate clock for further RX
            unsafe {
                self.regs.txd0.write(0);
            }
        }
        self.regs.config.modify(|_, w| w
            .pcs(true)
            .man_start_com(false)
        );
    }
}

pub struct Transfer<'r, Args: Iterator<Item = u32>> {
    regs: &'r mut regs::RegisterBlock,
    args: Args,
}

impl<'r, Args: Iterator<Item = u32>> Transfer<'r, Args> {
    pub fn new(regs: &'r mut regs::RegisterBlock, inst_code: u8, mut args: Args) -> Self
    where
        Args: Iterator<Item = u32>,
    {
        unsafe {
            regs.txd1.write(inst_code.into());
        }
        while !regs.intr_status.read().tx_fifo_full() {
            let arg = args.next().unwrap_or(0);
            unsafe {
                regs.txd0.write(arg);
            }
        }

        regs.config.modify(|_, w| w
                           .pcs(false)
                           .man_start_com(true)
        );
        Transfer { regs, args }
    }

    pub fn recv(&mut self) -> u32 {
        while !self.regs.intr_status.read().rx_fifo_not_empty() {}
        let rx = self.regs.rx_data.read();

        let arg = self.args.next().unwrap_or(0);
        unsafe {
            self.regs.txd0.write(arg);
        }

        rx
    }
}

impl<'r, Args: Iterator<Item = u32>> Drop for Transfer<'r, Args> {
    fn drop(&mut self) {
        self.regs.config.modify(|_, w| w
            .pcs(true)
            .man_start_com(false)
        );
    }
}


impl<'t, Args: Iterator<Item = u32>> From<&'t mut Transfer<'_, Args>> for u32 {
    fn from(t: &mut Transfer<'_, Args>) -> u32 {
        t.recv()
    }
}

impl<Args: Iterator<Item = u32>> From<&mut Transfer<'_, Args>> for u8 {
    fn from(t: &mut Transfer<'_, Args>) -> u8 {
        u32::from(t) as u8
    }
}

impl<Args: Iterator<Item = u32>> From<&mut Transfer<'_, Args>> for u16 {
    fn from(t: &mut Transfer<'_, Args>) -> u16 {
        u32::from(t) as u16
    }
}
