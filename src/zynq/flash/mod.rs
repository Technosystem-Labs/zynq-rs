//! Quad-SPI Flash Controller

use core::marker::PhantomData;
use crate::regs::{RegisterR, RegisterW, RegisterRW};
use super::slcr;
use super::clocks::CpuClocks;

mod regs;
mod bytes;
pub use bytes::{BytesTransferExt, BytesTransfer};

const FLASH_BAUD_RATE: u32 = 50_000_000;
const SINGLE_CAPACITY: u32 = 16 * 1024 * 1024;

/// Instruction: Read Configure Register
const INST_RDCR: u8 = 0x35;
/// Instruction: Read Status Register-1
const INST_RDSR1: u8 = 0x05;
/// Instruction: Read Identification
const INST_RDID: u8 = 0x9F;

#[derive(Clone)]
pub enum SpiWord {
    W8(u8),
    W16(u16),
    W24(u32),
    W32(u32),
}

impl From<u8> for SpiWord {
    fn from(x: u8) -> Self {
        SpiWord::W8(x)
    }
}

impl From<u16> for SpiWord {
    fn from(x: u16) -> Self {
        SpiWord::W16(x)
    }
}

impl From<u32> for SpiWord {
    fn from(x: u32) -> Self {
        SpiWord::W32(x)
    }
}

/// Memory-mapped mode
pub struct LinearAddressing;
/// Manual I/O mode
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

    fn wait_tx_fifo_flush(&mut self) {
        while !self.regs.intr_status.read().tx_fifo_not_full() {}
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
            self.regs.rx_thres.write(1);
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
            .endian(true)
        );

        self.regs.lqspi_cfg.write(regs::LqspiCfg::zeroed()
            .mode_bits(0xFF)
            .dummy_byte(0x2)
            .mode_en(true)
            // 2 devices
            .two_mem(true)
            .sep_bus(true)
            .u_page(chip_index != 0)
            // Manual I/O mode
            .lq_mode(false)
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

    /// Read Configuration Register
    pub fn rdcr(&mut self) -> u8 {
        let args = Some((INST_RDCR as u32) << 24);
        self.transfer(args.into_iter(), 4)
            .bytes_transfer().skip(1)
            .next().unwrap() as u8
    }

    /// Read Status Register-1
    pub fn rdsr1(&mut self) -> u8 {
        let args = Some(INST_RDSR1 as u8);
        self.transfer(args.into_iter(), 2)
            .bytes_transfer().skip(1)
            .next().unwrap()
    }

    /// Read Identifiaction
    pub fn rdid(&mut self) -> core::iter::Skip<BytesTransfer<Transfer<core::option::IntoIter<u32>, u32>>> {
        let args = Some((INST_RDID as u32) << 24);
        self.transfer(args.into_iter(), 0x44)
            .bytes_transfer().skip(1)
    }

    /// Read flash data
    pub fn read(&mut self, offset: u32, len: usize
    ) -> core::iter::Take<core::iter::Skip<BytesTransfer<Transfer<core::option::IntoIter<u32>, u32>>>>
    {
        let args = Some(((INST_READ as u32) << 24) | (offset as u32));
        self.transfer(args.into_iter(), len + 6)
            .bytes_transfer().skip(6).take(len)
    }

    pub fn transfer<'s: 't, 't, Args, W>(&'s mut self, args: Args, len: usize) -> Transfer<'t, Args, W>
    where
        Args: Iterator<Item = W>,
        W: Into<SpiWord>,
    {
        Transfer::new(self, args, len)
    }
}

pub struct Transfer<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> {
    flash: &'a mut Flash<Manual>,
    args: Args,
    sent: usize,
    received: usize,
    len: usize,
}

impl<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> Transfer<'a, Args, W> {
    pub fn new(flash: &'a mut Flash<Manual>, args: Args, len: usize) -> Self {
        flash.regs.config.modify(|_, w| w.pcs(false));
        flash.regs.enable.write(
            regs::Enable::zeroed()
                .spi_en(true)
        );

        let mut xfer = Transfer {
            flash,
            args,
            sent: 0,
            received: 0,
            len,
        };
        xfer.fill_tx_fifo();
        xfer.flash.regs.config.modify(|_, w| w.man_start_com(true));
        xfer
    }

    fn fill_tx_fifo(&mut self) {
        while self.sent < self.len && !self.flash.regs.intr_status.read().tx_fifo_full() {
            let arg = self.args.next()
                .map(|n| n.into())
                .unwrap_or(SpiWord::W32(0));
            match arg {
                SpiWord::W32(w) => {
                    // println!("txd0 {:08X}", w);
                    unsafe {
                        self.flash.regs.txd0.write(w);
                    }
                    self.sent += 4;
                }
                // Only txd0 can be used without flushing
                _ => {
                    if !self.flash.regs.intr_status.read().tx_fifo_not_full() {
                        // Flush if neccessary
                        self.flash.regs.config.modify(|_, w| w.man_start_com(true));
                        self.flash.wait_tx_fifo_flush();
                    }

                    match arg {
                        SpiWord::W8(w) => {
                            // println!("txd1 {:02X}", w);
                            unsafe {
                                self.flash.regs.txd1.write(u32::from(w) << 24);
                            }
                            self.sent += 1;
                        }
                        SpiWord::W16(w) => {
                            unsafe {
                                self.flash.regs.txd2.write(u32::from(w) << 16);
                            }
                            self.sent += 2;
                        }
                        SpiWord::W24(w) => {
                            unsafe {
                                self.flash.regs.txd3.write(w << 8);
                            }
                            self.sent += 3;
                        }
                        SpiWord::W32(_) => unreachable!(),
                    }

                    self.flash.regs.config.modify(|_, w| w.man_start_com(true));
                    self.flash.wait_tx_fifo_flush();
                }
            }
        }
    }

    fn can_read(&mut self) -> bool {
        self.flash.regs.intr_status.read().rx_fifo_not_empty()
    }

    fn read(&mut self) -> u32 {
        let rx = self.flash.regs.rx_data.read();
        self.received += 4;
        rx
    }
}

impl<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> Drop for Transfer<'a, Args, W> {
    fn drop(&mut self) {
        // Discard remaining rx_data
        while self.can_read() {
            self.read();
        }

        // Stop
        self.flash.regs.enable.write(
            regs::Enable::zeroed()
                .spi_en(false)
        );
        self.flash.regs.config.modify(|_, w| w
            .pcs(true)
            .man_start_com(false)
        );
    }
}

impl<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> Iterator for Transfer<'a, Args, W> {
    type Item = u32;

    fn next<'s>(&'s mut self) -> Option<u32> {
        if self.received >= self.len {
            return None;
        }

        self.fill_tx_fifo();

        while !self.can_read() {}
        Some(self.read())
    }
}
