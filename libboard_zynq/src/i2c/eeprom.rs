use super::I2C;
use crate::time::Milliseconds;
use embedded_hal::timer::CountDown;

pub struct EEPROM<'a> {
    i2c: &'a mut I2C,
    port: u8,
    address: u8,
    page_size: u8,
    count_down: crate::timer::global::CountDown<Milliseconds>
}

impl<'a> EEPROM<'a> {
    #[cfg(feature = "target_zc706")]
    pub fn new(i2c: &'a mut I2C, page_size: u8) -> Self {
        EEPROM {
            i2c: i2c,
            port: 2,
            address: 0b1010100,
            page_size: page_size,
            count_down: unsafe { crate::timer::GlobalTimer::get() }.countdown()
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

    /// Smart multi-page writing
    /// Using the "Page Write" function of an EEPROM, the memory region for each transaction
    /// (i.e. from byte `addr` to byte `addr+buf.len()`) should fit under each page 
    /// (i.e. `addr+buf.len()` < `addr/self.page_size+1`); otherwise, a roll-oever occurs, 
    /// where bytes beyond the page end. This smart function takes care of the scenario to avoid
    /// any roll-over when writing ambiguous memory regions.
    pub fn write(&mut self, addr: u8, buf: &[u8]) -> Result<(), &'static str> {
        self.select()?;

        let buf_len = buf.len();
        let mut pb: u8 = addr % self.page_size;
        for (i, byte) in buf.iter().enumerate() {
            if (i == 0) || (pb == 0) {
                self.i2c.start()?;
                self.i2c.write(self.address << 1)?;
                self.i2c.write(addr + (i as u8))?;
            }
            self.i2c.write(*byte)?;
            pb += 1;

            if (i == buf_len-1) || (pb == self.page_size) {
                self.i2c.stop()?;
                self.poll(1_000)?;
                pb = 0;
            }
        }

        Ok(())
    }

    /// Poll
    pub fn poll(&mut self, timeout_ms: u64) -> Result<(), &'static str> {
        self.select()?;

        self.count_down.start(Milliseconds(timeout_ms));
        loop {
            self.i2c.start()?;
            let ack = self.i2c.write(self.address << 1)?;
            self.i2c.stop()?;
            if ack {
                break
            };
            if !self.count_down.waiting() {
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