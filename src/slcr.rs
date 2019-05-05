use core::ops::RangeInclusive;
use volatile_register::{RO, WO, RW};
use bit_field::BitField;


#[repr(packed)]
pub struct UartClkCtrl {
    pub reg: RW<u32>,
}

impl UartClkCtrl {
    const ADDR: *mut Self = 0xF8000154 as *mut _;

    pub fn new() -> &'static mut Self {
        unsafe { &mut *Self::ADDR }
    }

    const DIVISOR: RangeInclusive<usize> = 8..=13;
    const CLKACT1: usize = 1;
    const CLKACT0: usize = 0;
    
    pub fn enable_uart0(&self) {
        unsafe {
            self.reg.modify(|mut x| {
                x.set_bits(Self::DIVISOR, 0x14);
                x.set_bit(Self::CLKACT0, true);
                x
            })
        }
    }
}

#[repr(packed)]
pub struct UartRstCtrl {
    pub reg: RW<u32>,
}

impl UartRstCtrl {
    const ADDR: *mut Self = 0xF8000228 as *mut _;

    pub fn new() -> &'static mut Self {
        unsafe { &mut *Self::ADDR }
    }

    const UART1_REF_RST: usize = 3;
    const UART0_REF_RST: usize = 2;
    const UART1_CPU1X_RST: usize = 1;
    const UART0_CPU1X_RST: usize = 0;

    pub fn reset_uart0(&self) {
        unsafe { toggle(&self.reg, Self::UART0_REF_RST); }
    }

    pub fn reset_uart1(&self) {
        unsafe { toggle(&self.reg, Self::UART1_REF_RST); }
    }
}

unsafe fn toggle<T: BitField + Copy>(reg: &RW<T>, bit: usize) {
    reg.modify(|x| {
        let mut x = x.clone();
        x.set_bit(bit, true);
        x
    });
    reg.modify(|x| {
        let mut x = x.clone();
        x.set_bit(bit, false);
        x
    });
}
