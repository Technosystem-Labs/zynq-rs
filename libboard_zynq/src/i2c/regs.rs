use volatile_register::{RO, WO, RW};

use libregister::{register, register_bit, register_bits};

// With reference to:
//
// artiq:artiq/gateware/targets/kasli.py:
// self.submodules.i2c = gpio.GPIOTristate([i2c.scl, i2c.sda])
//
// misoc:misoc/cores/gpio.py:
// class GPIOTristate(Module, AutoCSR):
//    def __init__(self, signals, reset_out=0, reset_oe=0):
//        l = len(signals)
//        self._in = CSRStatus(l)
//        self._out = CSRStorage(l, reset=reset_out)
//        self._oe = CSRStorage(l, reset=reset_oe)
// 
// Hence, using GPIOs as SCL and SDA GPIOs respectively.
//
// Current compatibility:
// zc706: GPIO 50, 51 == SCL, SDA

#[repr(C)]
pub struct RegisterBlock {
    pub gpio_output_mask: &'static mut GPIOOutputMask,
    pub gpio_input: &'static mut GPIOInput,
    pub gpio_output_enable: &'static mut GPIOOutputEnable,
}

impl RegisterBlock {
    pub unsafe fn new() -> Self {
        Self {
            gpio_output_mask: GPIOOutputMask::new(),
            gpio_input: GPIOInput::new(),
            gpio_output_enable: GPIOOutputEnable::new()
        }
    }
}

impl GPIOOutputMask {
    #[cfg(feature = "target_zc706")]
    pub unsafe fn new() -> &'static mut Self {
        &mut *(0xE000A00C as *mut _)
    }
}

impl GPIOInput {
    #[cfg(feature = "target_zc706")]
    pub unsafe fn new() -> &'static mut Self {
        &mut *(0xE000A064 as *mut _)
    }
}

impl GPIOOutputEnable {
    #[cfg(feature = "target_zc706")]
    pub unsafe fn new() -> &'static mut Self {
        &mut *(0xE000A248 as *mut _)
    }
}

// MASK_DATA_1_MSW:
// Maskable output data for MIO[53:48]
register!(gpio_output_mask, GPIOOutputMask, RW, u32);
// Output for SCL
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_mask, scl_o, 2);
// Output for SDA
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_mask, sda_o, 3);
// Mask for SCL; set to 1 to write to output
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_mask, scl_m, 18);
// Mask for SDA; set to 1 to write to output
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_mask, sda_m, 19);

// DATA_1_RO:
// Input data for MIO[53:32]
register!(gpio_input, GPIOInput, RO, u32);
// Input for SCL
#[cfg(feature = "target_zc706")]
register_bit!(gpio_input, scl, 8);
// Input for SDA
#[cfg(feature = "target_zc706")]
register_bit!(gpio_input, sda, 9);

// OEN_1:
// Output enable for MIO[53:32]
register!(gpio_output_enable, GPIOOutputEnable, RW, u32);
// Output enable for SCL
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_enable, scl, 8);
// Output enable for SDA
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_enable, sda, 9);
