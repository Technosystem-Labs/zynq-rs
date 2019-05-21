#![allow(unused)]

use core::fmt;
use volatile_register::RW;

use crate::regs::*;

mod regs;
mod baud_rate_gen;

pub struct Uart {
    regs: &'static mut regs::RegisterBlock,
}

impl Uart {
    pub fn uart1() -> Self {
        super::slcr::with_slcr(|| {
            let uart_rst_ctrl = super::slcr::UartRstCtrl::new();
            uart_rst_ctrl.reset_uart1();

            // Route UART 1 RxD/TxD Signals to MIO Pins
            unsafe {
                // TX pin
                let mio_pin_48 = &*(0xF80007C0 as *const RW<u32>);
                mio_pin_48.write(0x0000_12E0);
                // RX pin
                let mio_pin_49 = &*(0xF80007C4 as *const RW<u32>);
                mio_pin_49.write(0x0000_12E1);
            }

            let aper_clk_ctrl = super::slcr::AperClkCtrl::new();
            aper_clk_ctrl.enable_uart1();
            let uart_clk_ctrl = super::slcr::UartClkCtrl::new();
            uart_clk_ctrl.enable_uart1();
        });
        let self_ = Uart {
            regs: regs::RegisterBlock::uart1(),
        };
        self_.configure();
        self_
    }

    pub fn write_byte(&self, value: u8) {
        while self.tx_fifo_full() {}

        self.regs.tx_rx_fifo.write(
            regs::TxRxFifo::zeroed()
                .data(value.into())
        );
    }

    pub fn configure(&self) {
        // Configure UART character frame
        // * Disable clock-divider
        // * 8-bit
        // * 1 stop bit
        // * Normal channel mode
        // * No parity
        let parity_mode = regs::ParityMode::None;
        self.regs.mode.write(
            regs::Mode::zeroed()
                .par(parity_mode as u8)
                .chmode(regs::ChannelMode::Normal as u8)
        );

        // Configure the Baud Rate
        self.disable_rx();
        self.disable_tx();

        baud_rate_gen::configure(&self.regs, 50_000_000, 9_600);

        // Enable controller
        self.reset_rx();
        self.reset_tx();
        self.wait_reset();
        self.enable_rx();
        self.enable_tx();

        self.set_rx_timeout(false);
        self.set_break(false, true);
    }

    fn disable_rx(&self) {
        self.regs.control.modify(|_, w| {
            w.rxen(false)
             .rxdis(true)
        })
    }

    fn disable_tx(&self) {
        self.regs.control.modify(|_, w| {
            w.txen(false)
             .txdis(true)
        })
    }

    fn enable_rx(&self) {
        self.regs.control.modify(|_, w| {
            w.rxen(true)
             .rxdis(false)
        })
    }

    fn enable_tx(&self) {
        self.regs.control.modify(|_, w| {
            w.txen(true)
             .txdis(false)
        })
    }

    fn reset_rx(&self) {
        self.regs.control.modify(|_, w| {
            w.rxrst(true)
        })
    }

    fn reset_tx(&self) {
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

    fn set_break(&self, startbrk: bool, stopbrk: bool) {
        self.regs.control.modify(|_, w| {
            w.sttbrk(startbrk)
             .stpbrk(stopbrk)
        })
    }

    // 0 disables
    fn set_rx_timeout(&self, enable: bool) {
        self.regs.control.modify(|_, w| {
            w.rstto(enable)
        })
    }

    pub fn tx_fifo_full(&self) -> bool {
        self.regs.channel_sts.read().txfull()
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for b in s.bytes() {
            self.write_byte(b);
        }
        Ok(())
    }
}
