use libregister::{RegisterRW, RegisterW};
use libregister::{register, register_at, register_bit, register_bits};
use super::slcr;

pub struct ErrorLED {
    regs: RegisterBlock,
}

impl ErrorLED {
    #[cfg(feature = "target_kasli_soc")]
    pub fn error_led() -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            // Error LED at MIO pin 37
            slcr.mio_pin_37.write(
                slcr::MioPin37::zeroed()
                    .l3_sel(0b000)
                    .io_type(slcr::IoBufferType::Lvcmos25)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // reset
            slcr.gpio_rst_ctrl.reset_gpio();
        });

        Self::error_led_common(0xFFFF - 0x0080)
    }

    fn error_led_common(gpio_output_mask: u16) -> Self {
        // Setup register block
        let self_ = Self {
            regs: RegisterBlock::error_led(),
        };

        // Setup GPIO output mask
        self_.regs.gpio_output_mask.modify(|_, w| {
            w.mask(gpio_output_mask)
        });

        self_.regs.gpio_direction.modify(|_, w| {
            w.lederr(true)
        });

        self_
    }

    fn led_oe(&mut self, oe: bool) {
        self.regs.gpio_output_enable.modify(|_, w| {
             w.lederr(oe)
        })
    }

    fn led_o(&mut self, o: bool) {
        self.regs.gpio_output_mask.modify(|_, w| {
             w.lederr_o(o)
        })
    }

    pub fn toggle(&mut self, state: bool) {
        self.led_o(state);
        self.led_oe(state);
    }
}


pub struct RegisterBlock {
    pub gpio_output_mask: &'static mut GPIOOutputMask,
    pub gpio_direction: &'static mut GPIODirection,
    pub gpio_output_enable: &'static mut GPIOOutputEnable,
}

impl RegisterBlock {
    pub fn error_led() -> Self {
        Self {
            gpio_output_mask: GPIOOutputMask::new(),
            gpio_direction: GPIODirection::new(),
            gpio_output_enable: GPIOOutputEnable::new()
        }
    }
}

register!(gpio_output_mask,
            /// MASK_DATA_1_LSW:
            /// Maskable output data for MIO[47:32]
            GPIOOutputMask, RW, u32);
#[cfg(feature = "target_kasli_soc")]
register_at!(GPIOOutputMask, 0xE000A008, new);
#[cfg(feature = "target_kasli_soc")]
register_bit!(gpio_output_mask,
              /// Output for LED_ERR (MIO[37])
              lederr_o, 5);
#[cfg(feature = "target_kasli_soc")]
register_bits!(gpio_output_mask,
               mask, u16, 16, 31);

register!(gpio_direction,
          /// DIRM_1:
          /// Direction mode for MIO[53:32]; 0/1 = in/out
          GPIODirection, RW, u32);
#[cfg(feature = "target_kasli_soc")]
register_at!(GPIODirection, 0xE000A244, new);
#[cfg(feature = "target_kasli_soc")]
register_bit!(gpio_direction,
              /// Direction for LED_ERR
              lederr, 5);

register!(gpio_output_enable,
          /// OEN_1:
          /// Output enable for MIO[53:32]
          GPIOOutputEnable, RW, u32);
#[cfg(feature = "target_kasli_soc")]
register_at!(GPIOOutputEnable, 0xE000A248, new);
#[cfg(feature = "target_kasli_soc")]
register_bit!(gpio_output_enable,
              /// Output enable for LED_ERR
              lederr, 5);
              