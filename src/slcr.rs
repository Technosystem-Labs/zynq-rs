#[allow(unused)]

use crate::{register, register_bit, register_bits, regs::RegisterRW};

pub enum PllSource {
    IoPll  = 0b00,
    ArmPll = 0b10,
    DdrPll = 0b11,
}

register!(uart_clk_ctrl, UartClkCtrl, RW, u32);
register_bit!(uart_clk_ctrl, clkact0, 0);
register_bit!(uart_clk_ctrl, clkact1, 1);
register_bits!(uart_clk_ctrl, divisor, u8, 8, 13);
register_bits!(uart_clk_ctrl, srcsel, u8, 4, 5);
impl UartClkCtrl {
    const ADDR: *mut Self = 0xF8000154 as *mut _;

    pub fn new() -> &'static mut Self {
        unsafe { &mut *Self::ADDR }
    }
    
    pub fn enable_uart0(&self) {
        self.modify(|_, w| {
            // a. Clock divisor, slcr.UART_CLK_CTRL[DIVISOR] = 0x14.
            // b. Select the IO PLL, slcr.UART_CLK_CTRL[SRCSEL] = 0.
            // c. Enable the UART 0 Reference clock, slcr.UART_CLK_CTRL [CLKACT0] = 1.
            w.divisor(0x14)
             .srcsel(PllSource::IoPll as u8)
             .clkact0(true)
        })
    }
}

register!(uart_rst_ctrl, UartRstCtrl, RW, u32);
register_bit!(uart_rst_ctrl, uart0_ref_rst, 3);
register_bit!(uart_rst_ctrl, uart1_ref_rst, 2);
register_bit!(uart_rst_ctrl, uart0_cpu1x_rst, 1);
register_bit!(uart_rst_ctrl, uart1_cpu1x_rst, 0);
impl UartRstCtrl {
    const ADDR: *mut Self = 0xF8000228 as *mut _;

    pub fn new() -> &'static mut Self {
        unsafe { &mut *Self::ADDR }
    }

    pub fn reset_uart0(&self) {
        self.modify(|_, w| w.uart0_ref_rst(true));
        self.modify(|_, w| w.uart0_ref_rst(false));
    }

    pub fn reset_uart1(&self) {
        self.modify(|_, w| w.uart1_ref_rst(true));
        self.modify(|_, w| w.uart1_ref_rst(false));
    }
}
