use libregister::{
    register, register_at,
    register_bit, register_bits
};

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

pub struct RegisterBlock {
    pub gpio_output_mask: &'static mut GPIOOutputMask,
    pub gpio_input: &'static mut GPIOInput,
    pub gpio_direction: &'static mut GPIODirection,
    pub gpio_output_enable: &'static mut GPIOOutputEnable,
}

impl RegisterBlock {
    pub fn i2c() -> Self {
        Self {
            gpio_output_mask: GPIOOutputMask::new(),
            gpio_input: GPIOInput::new(),
            gpio_direction: GPIODirection::new(),
            gpio_output_enable: GPIOOutputEnable::new()
        }
    }
}

// MASK_DATA_1_MSW:
// Maskable output data for MIO[53:48]
register!(gpio_output_mask, GPIOOutputMask, RW, u32);
#[cfg(feature = "target_zc706")]
register_at!(GPIOOutputMask, 0xE000A00C, new);
// Output for SCL
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_mask, scl_o, 2);
// Output for SDA
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_mask, sda_o, 3);
// Mask for keeping bits except SCL and SDA unchanged
#[cfg(feature = "target_zc706")]
register_bits!(gpio_output_mask, mask, u16, 16, 31);

// DATA_1_RO:
// Input data for MIO[53:32]
register!(gpio_input, GPIOInput, RO, u32);
#[cfg(feature = "target_zc706")]
register_at!(GPIOInput, 0xE000A064, new);
// Input for SCL
#[cfg(feature = "target_zc706")]
register_bit!(gpio_input, scl, 18);
// Input for SDA
#[cfg(feature = "target_zc706")]
register_bit!(gpio_input, sda, 19);

// DIRM_1:
// Direction mode for MIO[53:32]; 0/1 = in/out
register!(gpio_direction, GPIODirection, RW, u32);
#[cfg(feature = "target_zc706")]
register_at!(GPIODirection, 0xE000A244, new);
// Direction for SCL
#[cfg(feature = "target_zc706")]
register_bit!(gpio_direction, scl, 18);
// Direction for SDA
#[cfg(feature = "target_zc706")]
register_bit!(gpio_direction, sda, 19);

// OEN_1:
// Output enable for MIO[53:32]
register!(gpio_output_enable, GPIOOutputEnable, RW, u32);
#[cfg(feature = "target_zc706")]
register_at!(GPIOOutputEnable, 0xE000A248, new);
// Output enable for SCL
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_enable, scl, 18);
// Output enable for SDA
#[cfg(feature = "target_zc706")]
register_bit!(gpio_output_enable, sda, 19);
