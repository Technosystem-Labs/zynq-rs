use bit_field::BitField;

use super::PhyRegister;

#[derive(Clone, Copy, Debug)]
/// Basic Mode Control Register
pub struct Control(pub u16);

impl Control {
    pub fn speed1(&self) -> bool {
        self.0.get_bit(6)
    }
    pub fn set_speed1(mut self, value: bool) -> Self {
        self.0.set_bit(6, value);
        self
    }
    pub fn collision_test(&self) -> bool {
        self.0.get_bit(7)
    }
    pub fn set_collision_test(mut self, value: bool) -> Self {
        self.0.set_bit(7, value);
        self
    }
    pub fn duplex(&self) -> bool {
        self.0.get_bit(8)
    }
    pub fn set_duplex(mut self, value: bool) -> Self {
        self.0.set_bit(8, value);
        self
    }
    pub fn restart_autoneg(&self) -> bool {
        self.0.get_bit(9)
    }
    pub fn set_restart_autoneg(mut self, value: bool) -> Self {
        self.0.set_bit(9, value);
        self
    }
    pub fn isolate(&self) -> bool {
        self.0.get_bit(10)
    }
    pub fn set_isolate(mut self, value: bool) -> Self {
        self.0.set_bit(10, value);
        self
    }
    pub fn power_down(&self) -> bool {
        self.0.get_bit(11)
    }
    pub fn set_power_down(mut self, value: bool) -> Self {
        self.0.set_bit(11, value);
        self
    }
    pub fn autoneg_enable(&self) -> bool {
        self.0.get_bit(12)
    }
    pub fn set_autoneg_enable(mut self, value: bool) -> Self {
        self.0.set_bit(12, value);
        self
    }
    pub fn speed0(&self) -> bool {
        self.0.get_bit(13)
    }
    pub fn set_speed0(mut self, value: bool) -> Self {
        self.0.set_bit(13, value);
        self
    }
    pub fn loopback(&self) -> bool {
        self.0.get_bit(14)
    }
    pub fn set_loopback(mut self, value: bool) -> Self {
        self.0.set_bit(14, value);
        self
    }
    pub fn reset(&self) -> bool {
        self.0.get_bit(15)
    }
    pub fn set_reset(mut self, value: bool) -> Self {
        self.0.set_bit(15, value);
        self
    }
}

impl PhyRegister for Control {
    fn addr() -> u8 {
        0
    }

    fn page() -> u8 {
        0
    }
}

impl From<u16> for Control {
    fn from(value: u16) -> Self {
        Control(value)
    }
}

impl Into<u16> for Control {
    fn into(self) -> u16 {
        self.0
    }
}
