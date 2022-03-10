use bit_field::BitField;
use super::{PhyRegister, Led0Control, Led1Control};

#[derive(Clone, Copy, Debug)]
/// LED Control Register
pub struct Leds(pub u16);

impl Leds {
    pub fn led0(&self) -> Led0Control {
        match self.0.get_bits(0..=3) {
            0b0000 => Led0Control::OnLinkOffNoLink,
            0b0001 => Led0Control::OnLinkBlinkActivityOffNoLink,
            0b0010 => Led0Control::BlinkDependingOnLink,
            0b0011 => Led0Control::OnActivityOffNoActivity,
            0b0100 => Led0Control::BlinkActivityOffNoActivity,
            0b0101 => Led0Control::OnTransmitOffNoTransmit,
            0b0110 => Led0Control::OnCopperLinkOffElse,
            0b0111 => Led0Control::On1000LinkOffElse,
            0b1000 => Led0Control::ForceOff,
            0b1001 => Led0Control::ForceOn,
            0b1010 => Led0Control::ForceHiZ,
            0b1011 => Led0Control::ForceBlink,
            0b1100 => Led0Control::Mode1,
            0b1101 => Led0Control::Mode2,
            0b1110 => Led0Control::Mode3,
            0b1111 => Led0Control::Mode4,
            _ => unreachable!()
        }
    }
    pub fn led1(&self) -> Led1Control {
        match self.0.get_bits(4..=7) {
            0b0000 => Led1Control::OnReceiveOffNoReceive,
            0b0001 => Led1Control::OnLinkBlinkActivityOffNoLink,
            0b0010 => Led1Control::OnLinkBlinkReceiveOffNoLink,
            0b0011 => Led1Control::OnActivityOffNoActivity,
            0b0100 => Led1Control::BlinkActivityOffNoActivity,
            0b0101 => Led1Control::On100OrFiberOffElse,
            0b0110 => Led1Control::On1001000LinkOffElse,
            0b0111 => Led1Control::On100LinkOffElse,
            0b1000 => Led1Control::ForceOff,
            0b1001 => Led1Control::ForceOn,
            0b1010 => Led1Control::ForceHiZ,
            0b1011 => Led1Control::ForceBlink,
            _ => unreachable!()
        }
    }

    pub fn set_led0(mut self, setting: Led0Control) -> Self {
        self.0.set_bits(0..=3, setting as u16);
        self
    }

    pub fn set_led1(mut self, setting: Led1Control) -> Self {
        self.0.set_bits(4..=7, setting as u16);
        self
    }
}

impl PhyRegister for Leds {
    fn addr() -> u8 {
        0x10
    }
        
    fn page() -> u8 {
        3
    }
}

impl From<u16> for Leds {
    fn from(value: u16) -> Self {
        Leds(value)
    }
}

impl Into<u16> for Leds {
    fn into(self) -> u16 {
        self.0
    }
}
