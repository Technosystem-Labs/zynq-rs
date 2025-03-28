//! I2C Bit-banging Controller

mod regs;
pub mod eeprom;
#[cfg(not(feature = "target_ebaz4205"))]
use super::slcr;
use super::time::Microseconds;
use embedded_hal::timer::CountDown;
use libregister::{RegisterR, RegisterRW};
#[cfg(not(feature = "target_ebaz4205"))]
use libregister::RegisterW;
#[cfg(feature = "target_kasli_soc")]
use log::info;
use log::error;

pub enum I2cMultiplexer {
    PCA9548 = 0,
    #[cfg(feature = "target_kasli_soc")]
    PCA9547 = 1,
}

#[derive(Debug)]
pub enum Error {
    Nack,
    SCLLow,
    SDALow,
    ArbitrationLost,
    UnknownSwitch,
    PollingTimeout,
    OtherError,
}

impl From<Error> for &str {
    fn from(err: Error) -> &'static str {
        match err {
            Error::Nack => "I2C write was not ACKed",
            Error::SCLLow => "SCL stuck low",
            Error::SDALow => "SDA stuck low",
            Error::ArbitrationLost => "SDA arbitration lost",
            Error::UnknownSwitch => "Unknown response for PCA954X autodetect",
            Error::PollingTimeout => "I2C polling timeout",
            Error::OtherError => "other error",
        }
    }
}

pub struct I2c {
    regs: regs::RegisterBlock,
    count_down: super::timer::global::CountDown<Microseconds>,
    pca_type: I2cMultiplexer
}

impl I2c {
    #[cfg(any(feature = "target_zc706", feature = "target_kasli_soc", feature = "target_ebaz4205"))]
    pub fn i2c0() -> Self {
        // Route I2C 0 SCL / SDA Signals to MIO Pins 50 / 51
        #[cfg(not(feature = "target_ebaz4205"))]
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
        });

        Self::i2c_common(0xFFFF - 0x000C, 0xFFFF - 0x0002)
    }

    fn i2c_common(gpio_output_mask: u16, _gpio_output_mask_lower: u16) -> Self {
        // Setup register block
        let self_ = Self {
            regs: regs::RegisterBlock::i2c(),
            count_down: unsafe { super::timer::GlobalTimer::get() }.countdown(),
            pca_type: I2cMultiplexer::PCA9548 //default for zc706
        };

        // Setup GPIO output mask
        self_.regs.gpio_output_mask.modify(|_, w| {
            w.mask(gpio_output_mask)
        });
        // Setup GPIO driver direction
        self_.regs.gpio_direction.modify(|_, w| {
            w.scl(true).sda(true)
        });

        //Kasli-SoC only: I2C_SW_RESET configuration
        #[cfg(feature = "target_kasli_soc")]
        {
            self_.regs.gpio_output_mask_lower.modify(|_, w| {
                w.mask(_gpio_output_mask_lower)
            });
            self_.regs.gpio_direction.modify(|_, w| {
                w.i2cswr(true)
            });
        }

        self_
    }

    /// Delay for I2C operations, simple wrapper for nb.
    fn delay_us(&mut self, us: u64) {
        self.count_down.start(Microseconds(us));
        nb::block!(self.count_down.wait()).unwrap();
    }

    fn unit_delay(&mut self) { self.delay_us(100) }

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

    #[cfg(feature = "target_kasli_soc")]
    fn i2cswr_oe(&mut self, oe: bool) {
        self.regs.gpio_output_enable.modify(|_, w| {
             w.i2cswr(oe)
        })
    }

    #[cfg(feature = "target_kasli_soc")]
    fn i2cswr_o(&mut self, o: bool) {
        self.regs.gpio_output_mask_lower.modify(|_, w| {
             w.i2cswr_o(o)
        })
    }

    #[cfg(feature = "target_kasli_soc")]
    fn pca_autodetect(&mut self) -> Result<I2cMultiplexer, Error> {
        // start with resetting the PCA954X
        // SDA must be clear (before start)
        // reset time is 500ns, unit_delay (100us) to account for propagation
        self.i2cswr_o(true);
        self.unit_delay();
        self.i2cswr_o(false);
        self.unit_delay();
    
        let pca954x_read_addr = (0x71 << 1) | 0x01;

        self.start()?;
        // read the config register
        self.write(pca954x_read_addr).map_err(|err| {
                match err {
                    Error::Nack => error!("PCA954X failed to ack read address"),
                    _ => ()
                }
                err
            }
        )?;
        let config = self.read(false)?;

        let pca = match config {
            0x00 => { info!("PCA9548 detected"); I2cMultiplexer::PCA9548 },
            0x08 => { info!("PCA9547 detected"); I2cMultiplexer::PCA9547 },
            _ => { return Err(Error::UnknownSwitch)},
        };
        self.stop()?;
        Ok(pca)
    }

    pub fn init(&mut self) -> Result<(), Error> {
        self.scl_oe(false);
        self.sda_oe(false);
        self.scl_o(false);
        self.sda_o(false);

        // Check the I2C bus is ready
        self.unit_delay();
        self.unit_delay();
        if !self.sda_i() {
            // Try toggling SCL a few times
            for _bit in 0..8 {
                self.scl_oe(true);
                self.unit_delay();
                self.scl_oe(false);
                self.unit_delay();
            }
        }

        if !self.sda_i() {
            return Err(Error::SDALow);
        }
        if !self.scl_i() {
            return Err(Error::SCLLow);
        }
        // postcondition: SCL and SDA high
        
        #[cfg(feature = "target_kasli_soc")]
        {
            self.i2cswr_oe(true);
            self.pca_type = self.pca_autodetect()?;
        }
        
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), Error> {
        // precondition: SCL and SDA high
        if !self.scl_i() {
            return Err(Error::SCLLow);
        }
        if !self.sda_i() {
            return Err(Error::ArbitrationLost);
        }
        self.sda_oe(true);
        self.unit_delay();
        self.scl_oe(true);
        self.unit_delay();
        // postcondition: SCL and SDA low
        Ok(())
    }

    pub fn restart(&mut self) -> Result<(), Error> {
        // precondition SCL and SDA low
        self.sda_oe(false);
        self.unit_delay();
        self.scl_oe(false);
        self.unit_delay();
        self.start()?;
        // postcondition: SCL and SDA low
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        // precondition: SCL and SDA low
        self.unit_delay();
        self.scl_oe(false);
        self.unit_delay();
        self.sda_oe(false);
        self.unit_delay();
        if !self.sda_i() {
            return Err(Error::ArbitrationLost);
        }
        // postcondition: SCL and SDA high
        Ok(())
    }

    pub fn write(&mut self, data: u8) -> Result<(), Error> {
        // precondition: SCL and SDA low
        // MSB first
        for bit in (0..8).rev() {
            self.sda_oe(data & (1 << bit) == 0);
            self.unit_delay();
            self.scl_oe(false);
            self.unit_delay();
            self.scl_oe(true);
            self.unit_delay();
        }
        self.sda_oe(false);
        self.unit_delay();
        self.scl_oe(false);
        self.unit_delay();
        // Read ack/nack
        let ack = !self.sda_i();
        self.scl_oe(true);
        self.unit_delay();
        self.sda_oe(true);
        // postcondition: SCL and SDA low

        if ack { Ok(()) } else { Err(Error::Nack) }
    }

    pub fn read(&mut self, ack: bool) -> Result<u8, Error> {
        // precondition: SCL and SDA low
        self.sda_oe(false);

        let mut data: u8 = 0;

        // MSB first
        for bit in (0..8).rev() {
            self.unit_delay();
            self.scl_oe(false);
            self.unit_delay();
            if self.sda_i() { data |= 1 << bit }
            self.scl_oe(true);
        }
        // Send ack/nack (true = nack, false = ack)
        self.sda_oe(ack);
        self.unit_delay();
        self.scl_oe(false);
        self.unit_delay();
        self.scl_oe(true);
        self.sda_oe(true);
        // postcondition: SCL and SDA low

        Ok(data)
    }

    pub fn pca954x_select(&mut self, address: u8, channel: Option<u8>) -> Result<(), Error> {
        self.start()?;
        // PCA9547 supports only one channel at a time
        // for compatibility, PCA9548 is treated as such too
        // channel - Some(x) - # of the channel [0,7], or None for all disabled
        let setting = match self.pca_type {
            I2cMultiplexer::PCA9548 => { 
                match channel {
                    Some(ch) => 1 << ch,
                    None => 0,
                }
            },
            #[cfg(feature = "target_kasli_soc")]
            I2cMultiplexer::PCA9547 => {
                match channel {
                    Some(ch) => ch | 0x08,
                    None => 0,
                }
            }
        };

        let write_res = self.write(address << 1).or_else( |err| {
                error!("PCA954X write address fail: {:?}", err);
                Err(err)
            }).and_then(|_| self.write(setting).or_else(|err| {
                error!("PCA954X control word fail: {:?}", err);
                Err(err)
            })
        );
        let stop_res = self.stop();

        write_res.and(stop_res)
    }
}
