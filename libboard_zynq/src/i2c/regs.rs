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
// kasli_soc: GPIO 50, 51 == SCL, SDA; GPIO 33 == I2C_SW_RESET
// ebaz4205: GPIO (EMIO)

pub struct RegisterBlock {
    pub gpio_output_mask: &'static mut GPIOOutputMask,
    pub gpio_input: &'static mut GPIOInput,
    pub gpio_direction: &'static mut GPIODirection,
    pub gpio_output_enable: &'static mut GPIOOutputEnable,
    #[cfg(feature = "target_kasli_soc")]
    pub gpio_output_mask_lower: &'static mut GPIOOutputMaskLower,
}

impl RegisterBlock {
    pub fn i2c() -> Self {
        Self {
            gpio_output_mask: GPIOOutputMask::new(),
            gpio_input: GPIOInput::new(),
            gpio_direction: GPIODirection::new(),
            gpio_output_enable: GPIOOutputEnable::new(),
            #[cfg(feature = "target_kasli_soc")]
            gpio_output_mask_lower: GPIOOutputMaskLower::new(),
        }
    }
}

register!(gpio_output_mask,
          /// MASK_DATA_1_MSW:
          /// Maskable output data for MIO[53:48]
          GPIOOutputMask, RW, u32);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_at!(GPIOOutputMask, 0xE000A00C, new);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_output_mask,
              /// Output for SCL
              scl_o, 2);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_output_mask,
              /// Output for SDA
              sda_o, 3);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bits!(gpio_output_mask,
               /// Mask for keeping bits except SCL and SDA unchanged
               mask, u16, 16, 31);


register!(gpio_output_mask_lower,
            /// MASK_DATA_1_LSW:
            /// Maskable output data for MIO[47:32]
            GPIOOutputMaskLower, RW, u32);
#[cfg(feature = "target_kasli_soc")]
register_at!(GPIOOutputMaskLower, 0xE000A008, new);
#[cfg(feature = "target_kasli_soc")]
register_bit!(gpio_output_mask_lower,
              /// Output for I2C_SW_RESET (MIO[33])
              i2cswr_o, 1);
#[cfg(feature = "target_kasli_soc")]
register_bits!(gpio_output_mask_lower,
               mask, u16, 16, 31);

register!(gpio_input,
          /// DATA_1_RO:
          /// Input data for MIO[53:32]
          GPIOInput, RO, u32);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_at!(GPIOInput, 0xE000A064, new);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_input,
              /// Input for SCL
              scl, 18);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_input,
              /// Input for SDA
              sda, 19);


register!(gpio_direction,
          /// DIRM_1:
          /// Direction mode for MIO[53:32]; 0/1 = in/out
          GPIODirection, RW, u32);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_at!(GPIODirection, 0xE000A244, new);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_direction,
              /// Direction for SCL
              scl, 18);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_direction,
              /// Direction for SDA
              sda, 19);
#[cfg(feature = "target_kasli_soc")]
register_bit!(gpio_direction,
              /// Direction for I2C_SW_RESET
              i2cswr, 1);

register!(gpio_output_enable,
          /// OEN_1:
          /// Output enable for MIO[53:32]
          GPIOOutputEnable, RW, u32);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_at!(GPIOOutputEnable, 0xE000A248, new);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_output_enable,
              /// Output enable for SCL
              scl, 18);
#[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
register_bit!(gpio_output_enable,
              /// Output enable for SDA
              sda, 19);
#[cfg(feature = "target_kasli_soc")]
register_bit!(gpio_output_enable,
              /// Output enable for I2C_SW_RESET
              i2cswr, 1);
              