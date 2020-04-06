//! Quad-SPI Flash Controller

use crate::{print, println};
use core::marker::PhantomData;
use libregister::{RegisterR, RegisterW, RegisterRW};
use super::slcr;
use super::clocks::source::{IoPll, ClockSource};

mod regs;
mod bytes;
pub use bytes::{BytesTransferExt, BytesTransfer};
mod spi_flash_register;
use spi_flash_register::*;
mod transfer;
use transfer::Transfer;

const FLASH_BAUD_RATE: u32 = 50_000_000;
/// 16 MB
pub const SINGLE_CAPACITY: u32 = 0x1000000;
pub const SECTOR_SIZE: u32 = 0x10000;
pub const PAGE_SIZE: u32 = 0x100;

/// Instruction: Read Identification
const INST_RDID: u8 = 0x9F;
/// Instruction: Read
const INST_READ: u8 = 0x03;
/// Instruction: Quad I/O Fast Read
const INST_4IO_FAST_READ: u8 = 0xEB;
/// Instruction: Write Disable
const INST_WRDI: u8 = 0x04;
/// Instruction: Write Enable
const INST_WREN: u8 = 0x06;
/// Instruction: Program page
const INST_PP: u8 = 0x02;
/// Instruction: Erase 4K Block
const INST_BE_4K: u8 = 0x20;

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
        self.regs.config.modify(|_, w| w.man_start_com(true));
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

    /// typical: `200_000_000` Hz
    fn enable_clocks(clock: u32) {
        let io_pll = IoPll::freq();
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
            );

            // Option: Add Second Serial Clock
            // 5. Configure MIO pin 9 for serial clock 1 output.
            slcr.mio_pin_09.write(
                slcr::MioPin09::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );

            // Option: Add 4-bit Data
            // 6. Configure MIO pins 10 through 13 for I/O.
            slcr.mio_pin_10.write(
                slcr::MioPin10::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            slcr.mio_pin_11.write(
                slcr::MioPin11::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            slcr.mio_pin_12.write(
                slcr::MioPin12::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            slcr.mio_pin_13.write(
                slcr::MioPin13::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );

            // Option: Add Feedback Output Clock
            // 7. Configure MIO pin 8 for feedback clock.
            slcr.mio_pin_08.write(
                slcr::MioPin08::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
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
            .inst_code(INST_4IO_FAST_READ)
            .dummy_mask(0x2)
            .mode_en(false)
            .mode_bits(0xFF)
            // 2 devices
            .two_mem(true)
            .u_page(false)
            // Quad SPI mode
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
            // Quad I/O Fast Read
            .inst_code(INST_READ)
            .dummy_mask(0x2)
            .mode_en(false)
            .mode_bits(0xFF)
            // 2 devices
            .two_mem(true)
            .u_page(false)
            // Quad SPI mode
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

    pub fn read_reg<R: SpiFlashRegister>(&mut self) -> R {
        let args = Some(R::inst_code());
        let transfer = self.transfer(args.into_iter(), 2)
            .bytes_transfer();
        R::new(transfer.skip(1).next().unwrap())
    }

    pub fn read_reg_until<R, F, A>(&mut self, f: F) -> A
    where
        R: SpiFlashRegister,
        F: Fn(R) -> Option<A>,
    {
        let mut result = None;
        while result.is_none() {
            let args = Some(R::inst_code());
            for b in self.transfer(args.into_iter(), 32)
                .bytes_transfer().skip(1) {
                    result = f(R::new(b));
                    
                    if result.is_none() {
                        break;
                    }
                }
        }
        result.unwrap()
    }

    /// Status Register-1 remains `0x00` immediately after invoking a command.
    fn wait_while_sr1_zeroed(&mut self) -> SR1 {
        self.read_reg_until::<SR1, _, SR1>(|sr1|
            if sr1.is_zeroed() {
                None
            } else {
                Some(sr1)
            }
        )
    }

    /// Read Identification
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

    pub fn erase(&mut self, offset: u32) {
        let args = Some(((INST_BE_4K as u32) << 24) | (offset as u32));
        self.transfer(args.into_iter(), 4);

        let sr1 = self.wait_while_sr1_zeroed();

        if sr1.e_err() {
            println!("E_ERR");
        } else if sr1.p_err() {
            println!("P_ERR");
        } else if sr1.wip() {
            print!("Erase in progress");
            while self.read_reg::<SR1>().wip() {
                print!(".");
            }
            println!("");
        } else {
            println!("erased? sr1={:02X}", sr1.inner);
        }
    }

    pub fn program<I: Iterator<Item=u32>>(&mut self, offset: u32, data: I) {
        {
            let len = 4 + 4 * data.size_hint().0;
            let args = Some(SpiWord::W32(((INST_PP as u32) << 24) | (offset as u32))).into_iter()
                .chain(data.map(SpiWord::W32));
            self.transfer(args, len);
        }

        // let sr1 = self.wait_while_sr1_zeroed();
        let sr1 = self.read_reg::<SR1>();

        if sr1.e_err() {
            println!("E_ERR");
        } else if sr1.p_err() {
            println!("P_ERR");
        } else if sr1.wip() {
            println!("Program in progress");
            while self.read_reg::<SR1>().wip() {
                print!(".");
            }
            println!("");
        } else {
            println!("programmed? sr1={:02X}", sr1.inner);
        }
    }

    pub fn write_enabled<F: Fn(&mut Self) -> R, R>(&mut self, f: F) -> R {
        // Write Enable
        let args = Some(INST_WREN);
        self.transfer(args.into_iter(), 1);
        self.regs.gpio.modify(|_, w| w.wp_n(true));
        let sr1 = self.wait_while_sr1_zeroed();
        if !sr1.wel() {
            panic!("Cannot write-enable flash");
        }

        let result = f(self);

        // Write Disable
        let args = Some(INST_WRDI);
        self.transfer(args.into_iter(), 1);
        self.regs.gpio.modify(|_, w| w.wp_n(false));

        result
    }

    pub fn transfer<'s: 't, 't, Args, W>(&'s mut self, args: Args, len: usize) -> Transfer<'t, Args, W>
    where
        Args: Iterator<Item = W>,
        W: Into<SpiWord>,
    {
        Transfer::new(self, args, len)
    }

    pub fn dump(&mut self, label: &'_ str, inst_code: u8) {
        print!("{}:", label);

        let args = Some(u32::from(inst_code) << 24);
        for b in self.transfer(args.into_iter(), 32).bytes_transfer() {
            print!(" {:02X}", b);
        }
        println!("");
    }
}
