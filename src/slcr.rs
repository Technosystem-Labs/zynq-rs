#[allow(unused)]

use crate::{register, register_bit, register_bits, register_at, regs::RegisterW, regs::RegisterRW};

pub enum PllSource {
    IoPll  = 0b00,
    ArmPll = 0b10,
    DdrPll = 0b11,
}

pub fn with_slcr<F: FnMut() -> R, R>(mut f: F) -> R {
    unsafe { SlcrUnlock::new() }.unlock();
    let r = f();
    unsafe { SlcrLock::new() }.lock();
    r
}

register!(slcr_lock, SlcrLock, WO, u32);
register_bits!(slcr_lock, lock_key, u16, 0, 15);
register_at!(SlcrLock, 0xF8000004, new);
impl SlcrLock {
    pub fn lock(&self) {
        unsafe {
            self.write(
                Self::zeroed()
                    .lock_key(0x767B)
            );
        }
    }
}

register!(slcr_unlock, SlcrUnlock, WO, u32);
register_bits!(slcr_unlock, unlock_key, u16, 0, 15);
register_at!(SlcrUnlock, 0xF8000008, new);
impl SlcrUnlock {
    pub fn unlock(&self) {
        unsafe {
            self.write(
                Self::zeroed()
                    .unlock_key(0xDF0D)
            );
        }
    }
}

register!(aper_clk_ctrl, AperClkCtrl, RW, u32);
register_bit!(aper_clk_ctrl, uart1_cpu_1xclkact, 21);
register_bit!(aper_clk_ctrl, uart0_cpu_1xclkact, 20);
register_at!(AperClkCtrl, 0xF800012C, new);
impl AperClkCtrl {
    pub fn enable_uart0(&self) {
        self.modify(|_, w| w.uart0_cpu_1xclkact(true));
    }

    pub fn enable_uart1(&self) {
        self.modify(|_, w| w.uart1_cpu_1xclkact(true));
    }
}


register!(uart_clk_ctrl, UartClkCtrl, RW, u32);
register_bit!(uart_clk_ctrl, clkact0, 0);
register_bit!(uart_clk_ctrl, clkact1, 1);
register_bits!(uart_clk_ctrl, divisor, u8, 8, 13);
register_bits!(uart_clk_ctrl, srcsel, u8, 4, 5);
register_at!(UartClkCtrl, 0xF8000154, new);
impl UartClkCtrl {
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

    pub fn enable_uart1(&self) {
        self.modify(|_, w| {
            // a. Clock divisor, slcr.UART_CLK_CTRL[DIVISOR] = 0x14.
            // b. Select the IO PLL, slcr.UART_CLK_CTRL[SRCSEL] = 0.
            // c. Enable the UART 1 Reference clock, slcr.UART_CLK_CTRL [CLKACT1] = 1.
            w.divisor(0x14)
             .srcsel(PllSource::IoPll as u8)
             .clkact1(true)
        })
    }
}

register!(uart_rst_ctrl, UartRstCtrl, RW, u32);
register_bit!(uart_rst_ctrl, uart0_ref_rst, 3);
register_bit!(uart_rst_ctrl, uart1_ref_rst, 2);
register_bit!(uart_rst_ctrl, uart0_cpu1x_rst, 1);
register_bit!(uart_rst_ctrl, uart1_cpu1x_rst, 0);
register_at!(UartRstCtrl, 0xF8000228, new);
impl UartRstCtrl {
    pub fn reset_uart0(&self) {
        self.modify(|_, w| w.uart0_ref_rst(true));
        self.modify(|_, w| w.uart0_ref_rst(false));
    }

    pub fn reset_uart1(&self) {
        self.modify(|_, w| w.uart1_ref_rst(true));
        self.modify(|_, w| w.uart1_ref_rst(false));
    }
}
