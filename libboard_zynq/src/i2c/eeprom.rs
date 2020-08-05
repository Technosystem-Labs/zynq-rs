use super::I2C;
use crate::time::Microseconds;
use embedded_hal::timer::CountDown;

pub struct EEPROM<'a> {
    i2c: &'a mut I2C,
    port: u8,
    address: u8,
}

impl<'a> EEPROM<'a> {
    #[cfg(feature = "target_zc706")]
    pub fn new(i2c: &'a mut I2C) -> Self {
        EEPROM {
            i2c: i2c,
            port: 2,
            address: 0b1010100,
        }
    }

    #[cfg(feature = "target_zc706")]
    fn select(&mut self) -> Result<(), &'static str> {
        let mask: u16 = 1 << self.port;
        self.i2c.pca9548_select(0b1110100, mask as u8)?;
        Ok(())
    }

    /// Random read
    pub fn read<'r>(&mut self, addr: u8, buf: &'r mut [u8]) -> Result<(), &'static str> {
        self.select()?;

        self.i2c.start()?;
        self.i2c.write(self.address << 1)?;
        self.i2c.write(addr)?;

        self.i2c.restart()?;
        self.i2c.write((self.address << 1) | 1)?;
        let buf_len = buf.len();
        for (i, byte) in buf.iter_mut().enumerate() {
            *byte = self.i2c.read(i < buf_len - 1)?;
        }

        self.i2c.stop()?;

        Ok(())
    }

    /// Page write
    pub fn write(&mut self, addr: u8, buf: &[u8]) -> Result<(), &'static str> {
        self.select()?;

        self.i2c.start()?;
        self.i2c.write(self.address << 1)?;
        self.i2c.write(addr)?;
        for (i, byte) in buf.iter().enumerate() {
            self.i2c.write(*byte)?;
        }

        self.i2c.stop()?;
        self.poll(1_000_000_000)?;

        Ok(())
    }

    /// Poll
    pub fn poll(&mut self, timeout_us: u64) -> Result<(), &'static str> {
        self.select()?;

        self.i2c.count_down.start(Microseconds(timeout_us));
        while !self.i2c.write((self.address << 1) | 1)? {
            if !self.i2c.count_down.waiting() {
                return Err("I2C polling timeout")
            }
        }

        Ok(())
    }

    pub fn read_eui48<'r>(&mut self) -> Result<[u8; 6], &'static str> {
        let mut buffer = [0u8; 6];
        self.read(0xFA, &mut buffer)?;
        Ok(buffer)
    }
}