use core::fmt;

use crate::regs::*;
use super::slcr;
use super::clocks::CpuClocks;

mod regs;
mod baud_rate_gen;

pub struct Uart {
    regs: &'static mut regs::RegisterBlock,
}

impl Uart {
    #[cfg(feature = "target_zc706")]
    pub fn serial(baudrate: u32) -> Self {
        let slcr = slcr::RegisterBlock::new();
        // Route UART 1 RxD/TxD Signals to MIO Pins
        // TX pin
        slcr.mio_pin_48.write(
            slcr::MioPin48::zeroed()
                .l3_sel(0b111)
                .io_type(slcr::IoBufferType::Lvcmos18)
                .pullup(true)
        );
        // RX pin
        slcr.mio_pin_49.write(
            slcr::MioPin49::zeroed()
                .tri_enable(true)
                .l3_sel(0b111)
                .io_type(slcr::IoBufferType::Lvcmos18)
                .pullup(true)
        );
        Self::uart1(baudrate)
    }

    #[cfg(feature = "target_cora_z7_10")]
    pub fn serial(baudrate: u32) -> Self {
        let slcr = slcr::RegisterBlock::new();
        // Route UART 0 RxD/TxD Signals to MIO Pins
        // TX pin
        slcr.mio_pin_15.write(
            slcr::MioPin15::zeroed()
                .l3_sel(0b111)
                .io_type(slcr::IoBufferType::Lvcmos33)
                .pullup(true)
        );
        // RX pin
        slcr.mio_pin_14.write(
            slcr::MioPin14::zeroed()
                .tri_enable(true)
                .l3_sel(0b111)
                .io_type(slcr::IoBufferType::Lvcmos33)
                .pullup(true)
        );
        Self::uart0(baudrate)
    }

    pub fn uart0(baudrate: u32) -> Self {
        let slcr = slcr::RegisterBlock::new();
        slcr.uart_rst_ctrl.reset_uart0();
        slcr.aper_clk_ctrl.enable_uart0();
        slcr.uart_clk_ctrl.enable_uart0();
        let mut self_ = Uart {
            regs: regs::RegisterBlock::uart0(),
        };
        self_.configure(baudrate);
        self_
    }

    pub fn uart1(baudrate: u32) -> Self {
        let slcr = slcr::RegisterBlock::new();
        slcr.uart_rst_ctrl.reset_uart1();
        slcr.aper_clk_ctrl.enable_uart1();
        slcr.uart_clk_ctrl.enable_uart1();
        let mut self_ = Uart {
            regs: regs::RegisterBlock::uart1(),
        };
        self_.configure(baudrate);
        self_
    }

    pub fn write_byte(&mut self, value: u8) {
        while self.tx_fifo_full() {}

        self.regs.tx_rx_fifo.write(
            regs::TxRxFifo::zeroed()
                .data(value.into())
        );
    }

    pub fn configure(&mut self, baudrate: u32) {
        // Configure UART character frame
        // * Disable clock-divider
        // * 8-bit
        // * 1 stop bit
        // * Normal channel mode
        // * No parity
        self.regs.mode.write(
            regs::Mode::zeroed()
                .par(regs::ParityMode::None)
                .chmode(regs::ChannelMode::Normal)
        );

        // Configure the Baud Rate
        self.disable_rx();
        self.disable_tx();

        let clocks = CpuClocks::get();
        baud_rate_gen::configure(self.regs, clocks.uart_ref_clk(), baudrate);

        // Enable controller
        self.reset_rx();
        self.reset_tx();
        self.wait_reset();
        self.enable_rx();
        self.enable_tx();

        self.set_rx_timeout(false);
        self.set_break(false, true);
    }

    fn disable_rx(&mut self) {
        self.regs.control.modify(|_, w| {
            w.rxen(false)
             .rxdis(true)
        })
    }

    fn disable_tx(&mut self) {
        self.regs.control.modify(|_, w| {
            w.txen(false)
             .txdis(true)
        })
    }

    fn enable_rx(&mut self) {
        self.regs.control.modify(|_, w| {
            w.rxen(true)
             .rxdis(false)
        })
    }

    fn enable_tx(&mut self) {
        self.regs.control.modify(|_, w| {
            w.txen(true)
             .txdis(false)
        })
    }

    fn reset_rx(&mut self) {
        self.regs.control.modify(|_, w| {
            w.rxrst(true)
        })
    }

    fn reset_tx(&mut self) {
        self.regs.control.modify(|_, w| {
            w.txrst(true)
        })
    }

    /// Wait for `reset_rx()` or `reset_tx()` to complete
    fn wait_reset(&self) {
        let mut pending = true;
        while pending {
            let control = self.regs.control.read();
            pending = control.rxrst() || control.txrst();
        }
    }

    fn set_break(&mut self, startbrk: bool, stopbrk: bool) {
        self.regs.control.modify(|_, w| {
            w.sttbrk(startbrk)
             .stpbrk(stopbrk)
        })
    }

    // 0 disables
    fn set_rx_timeout(&mut self, enable: bool) {
        self.regs.control.modify(|_, w| {
            w.rstto(enable)
        })
    }

    pub fn tx_fifo_full(&self) -> bool {
        self.regs.channel_sts.read().txfull()
    }

    pub fn tx_fifo_empty(&self) -> bool {
        self.regs.channel_sts.read().txempty()
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        while !self.tx_fifo_empty() {}

        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}
