#![allow(unused)]

use core::fmt;

use crate::regs::*;

mod regs;

#[repr(u8)]
pub enum ParityMode {
    EvenParity = 0b000,
    OddParity  = 0b001,
    ForceTo0   = 0b010,
    ForceTo1   = 0b011,
}

pub struct Uart {
    regs: &'static mut regs::RegisterBlock,
}

impl Uart {
    pub fn uart0() -> Self {
        let uart_rst_ctrl = super::slcr::UartRstCtrl::new();
        uart_rst_ctrl.reset_uart0();
        // TODO: Route UART 0 RxD/TxD Signals to MIO Pins

        let uart_clk_ctrl = super::slcr::UartClkCtrl::new();
        uart_clk_ctrl.enable_uart0();

        let self_ = Uart {
            regs: regs::RegisterBlock::uart0(),
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
        // Confiugre UART character frame
        // * Disable clock-divider
        // * 8-bit
        // * 1 stop bit
        // * Normal channel mode
        // * no parity
        let parity_mode = ParityMode::ForceTo0;
        self.regs.mode.write(
            regs::Mode::zeroed()
                .par(parity_mode as u8)
        );

        // Configure the Baud Rate
        self.disable_rx();
        self.disable_tx();

        // 9,600 baud
        self.regs.baud_rate_gen.write(regs::BaudRateGen::zeroed().cd(651));
        self.regs.baud_rate_divider.write(regs::BaudRateDiv::zeroed().bdiv(7));

        self.reset_rx();
        self.reset_tx();
        self.enable_rx();
        self.enable_tx();
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
