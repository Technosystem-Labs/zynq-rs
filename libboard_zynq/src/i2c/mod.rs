//! I2C Bit-banging Controller

mod regs;
pub mod eeprom;
use super::slcr;
use super::time::Microseconds;
use embedded_hal::timer::CountDown;
use libregister::{RegisterR, RegisterRW, RegisterW};

pub struct I2c {
    regs: regs::RegisterBlock,
    count_down: super::timer::global::CountDown<Microseconds>
}

impl I2c {
    #[cfg(any(feature = "target_zc706", feature = "target_kasli_soc"))]
    pub fn i2c0() -> Self {
        // Route I2C 0 SCL / SDA Signals to MIO Pins 50 / 51
        slcr::RegisterBlock::unlocked(|slcr| {
            // SCL
            slcr.mio_pin_50.write(
                slcr::MioPin50::zeroed()
                    .l3_sel(0b000)  // as GPIO 50
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // SDA
            slcr.mio_pin_51.write(
                slcr::MioPin51::zeroed()
                    .l3_sel(0b000)  // as GPIO 51
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // On Kasli-SoC prototype, leakage through the unconfigured I2C_SW_RESET
            // MIO pin develops enough voltage on the T21 gate to assert the reset.
            // Configure the pin to avoid this problem.
            #[cfg(feature = "target_kasli_soc")]
            slcr.mio_pin_33.write(
                slcr::MioPin33::zeroed()
                    .l3_sel(0b000)
                    .io_type(slcr::IoBufferType::Lvcmos33)
                    .pullup(false)
                    .disable_rcvr(true)
            );
            // Reset
            slcr.gpio_rst_ctrl.reset_gpio();
        });

        Self::i2c_common(0xFFFF - 0x000C)
    }

    fn i2c_common(gpio_output_mask: u16) -> Self {
        // Setup register block
        let self_ = Self {
            regs: regs::RegisterBlock::i2c(),
            count_down: unsafe { super::timer::GlobalTimer::get() }.countdown()
        };

        // Setup GPIO output mask
        self_.regs.gpio_output_mask.modify(|_, w| {
            w.mask(gpio_output_mask)
        });
        // Setup GPIO driver direction
        self_.regs.gpio_direction.modify(|_, w| {
            w.scl(true).sda(true)
        });

        self_
    }

    /// Delay for I2C operations, simple wrapper for nb.
    fn delay_us(&mut self, us: u64) {
        self.count_down.start(Microseconds(us));
        nb::block!(self.count_down.wait()).unwrap();
    }

    fn half_period(&mut self) { self.delay_us(100) }

    fn sda_i(&mut self) -> bool {
        self.regs.gpio_input.read().sda()
    }

    fn scl_i(&mut self) -> bool {
        self.regs.gpio_input.read().scl()
    }

    fn sda_oe(&mut self, oe: bool) {
        self.regs.gpio_output_enable.modify(|_, w| {
             w.sda(oe)
        })
    }

    fn sda_o(&mut self, o: bool) {
        self.regs.gpio_output_mask.modify(|_, w| {
             w.sda_o(o)
        })
    }

    fn scl_oe(&mut self, oe: bool) {
        self.regs.gpio_output_enable.modify(|_, w| {
             w.scl(oe)
        })
    }

    fn scl_o(&mut self, o: bool) {
        self.regs.gpio_output_mask.modify(|_, w| {
             w.scl_o(o)
        })
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        self.scl_oe(false);
        self.sda_oe(false);
        self.scl_o(false);
        self.sda_o(false);

        // Check the I2C bus is ready
        self.half_period();
        self.half_period();
        if !self.sda_i() {
            // Try toggling SCL a few times
            for _bit in 0..8 {
                self.scl_oe(true);
                self.half_period();
                self.scl_oe(false);
                self.half_period();
            }
        }

        if !self.sda_i() {
            return Err("SDA is stuck low and doesn't get unstuck");
        }
        if !self.scl_i() {
            return Err("SCL is stuck low and doesn't get unstuck");
        }
        // postcondition: SCL and SDA high
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), &'static str> {
        // precondition: SCL and SDA high
        if !self.scl_i() {
            return Err("SCL is stuck low and doesn't get unstuck");
        }
        if !self.sda_i() {
            return Err("SDA arbitration lost");
        }
        self.sda_oe(true);
        self.half_period();
        self.scl_oe(true);
        // postcondition: SCL and SDA low
        Ok(())
    }

    pub fn restart(&mut self) -> Result<(), &'static str> {
        // precondition SCL and SDA low
        self.sda_oe(false);
        self.half_period();
        self.scl_oe(false);
        self.half_period();
        self.start()?;
        // postcondition: SCL and SDA low
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), &'static str> {
        // precondition: SCL and SDA low
        self.half_period();
        self.scl_oe(false);
        self.half_period();
        self.sda_oe(false);
        self.half_period();
        if !self.sda_i() {
            return Err("SDA arbitration lost");
        }
        // postcondition: SCL and SDA high
        Ok(())
    }

    pub fn write(&mut self, data: u8) -> Result<bool, &'static str> {
        // precondition: SCL and SDA low
        // MSB first
        for bit in (0..8).rev() {
            self.sda_oe(data & (1 << bit) == 0);
            self.half_period();
            self.scl_oe(false);
            self.half_period();
            self.scl_oe(true);
        }
        self.sda_oe(false);
        self.half_period();
        self.scl_oe(false);
        self.half_period();
        // Read ack/nack
        let ack = !self.sda_i();
        self.scl_oe(true);
        self.sda_oe(true);
        // postcondition: SCL and SDA low

        Ok(ack)
    }

    pub fn read(&mut self, ack: bool) -> Result<u8, &'static str> {
        // precondition: SCL and SDA low
        self.sda_oe(false);

        let mut data: u8 = 0;

        // MSB first
        for bit in (0..8).rev() {
            self.half_period();
            self.scl_oe(false);
            self.half_period();
            if self.sda_i() { data |= 1 << bit }
            self.scl_oe(true);
        }
        // Send ack/nack
        self.sda_oe(ack);
        self.half_period();
        self.scl_oe(false);
        self.half_period();
        self.scl_oe(true);
        self.sda_oe(true);
        // postcondition: SCL and SDA low

        Ok(data)
    }

    pub fn pca9548_select(&mut self, address: u8, channels: u8) -> Result<(), &'static str> {
        self.start()?;
        if !self.write(address << 1)? {
            return Err("PCA9548 failed to ack write address")
        }
        if !self.write(channels)? {
            return Err("PCA9548 failed to ack control word")
        }
        self.stop()?;
        Ok(())
    }
}
