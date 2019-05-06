use volatile_register::{RO, WO, RW};

use crate::{register, register_bit, register_bits, regs::Register};

register!(uart_clk_ctrl, UartClkCtrl, u32);
register_bit!(uart_clk_ctrl, clkact0, 0);
register_bit!(uart_clk_ctrl, clkact1, 1);
register_bits!(uart_clk_ctrl, divisor, u8, 8, 13);
impl UartClkCtrl {
    const ADDR: *mut Self = 0xF8000154 as *mut _;

    pub fn new() -> &'static mut Self {
        unsafe { &mut *Self::ADDR }
    }
    
    pub fn enable_uart0(&self) {
        self.modify(|_, w| {
            w.clkact0(true)
             .divisor(0x14)
        })
    }
}

register!(uart_rst_ctrl, UartRstCtrl, u32);
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
