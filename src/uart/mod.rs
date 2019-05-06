mod regs;
pub use regs::RegisterBlock;

pub struct Uart {
    regs: &'static mut RegisterBlock,
}

impl Uart {
    pub fn uart0() -> Self {
        let uart_rst_ctrl = super::slcr::UartRstCtrl::new();
        uart_rst_ctrl.reset_uart0();
        // TODO: Route UART 0 RxD/TxD Signals to MIO Pins

        // a. Clock divisor, slcr.UART_CLK_CTRL[DIVISOR] = 0x14.
        // b. Select the IO PLL, slcr.UART_CLK_CTRL[SRCSEL] = 0.
        // c. Enable the UART 0 Reference clock, slcr.UART_CLK_CTRL [CLKACT0] = 1.
        // d. Disable UART 1 Reference clock, slcr.UART_CLK_CTRL [CLKACT1] bit = 0.
        let uart_clk_ctrl = super::slcr::UartClkCtrl::new();
        uart_clk_ctrl.enable_uart0();
            
        Uart {
            regs: RegisterBlock::uart0(),
        }.init()
    }

    fn init(self) -> Self {
        self.regs.configure();
        self
    }

    pub fn write_byte(&self, v: u8) {
        unsafe {
            while self.regs.tx_fifo_full() {}

            self.regs.write_byte(v);
        }
    }
}
