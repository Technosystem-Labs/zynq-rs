use bit_field::BitField;
use super::{PhyRegister, Link, LinkDuplex, LinkSpeed};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LedControl {
    OnLinkOffNoLink = 0b0000,
    OnLinkBlinkActivityOffNoLink = 0b0001,
    OnFullDuplexBlinkCollisionOffHalfDuplex = 0b0010,
    OnActivityOffNoActivity = 0b0011,
    BlinkActivityOffNoActivity = 0b0100,
    OnTransmitOffNoTransmit = 0b0101,
    On101000LinkOffElse = 0b0110,
    On10LinkOffElse = 0b0111,
    ForceOff = 0b1000,
    ForceOn = 0b1001,
    ForceHiZ = 0b1010,
    ForceBlink = 0b1011,
    //LED[0] only
    Mode1 = 0b1100,
    Mode2 = 0b1101,
    Mode3 = 0b1110,
    Mode4 = 0b1111
}

#[derive(Clone, Copy, Debug)]
/// LED Control Register
pub struct Leds(pub u16);

impl Leds {
    pub fn led(led_no: u8) -> LedControl {
        match self.0.get_bits((led_no*4)..=(led_no*4+3)) {
            0b0000 => LedControl::OnLinkOffNoLink,
            0b0001 => LedControl::OnLinkBlinkActivityOffNoLink,
            0b0010 => LedControl::OnFullDuplexBlinkCollisionOffHalfDuplex,
            0b0011 => LedControl::OnActivityOffNoActivity,
            0b0100 => LedControl::BlinkActivityOffNoActivity,
            0b0101 => LedControl::OnTransmitOffNoTransmit,
            0b0110 => LedControl::On101000LinkOffElse,
            0b0111 => LedControl::On10LinkOffElse,
            0b1000 => LedControl::ForceOff,
            0b1001 => LedControl::ForceOn,
            0b1010 => LedControl::ForceHiZ,
            0b1011 => LedControl::ForceBlink,
            0b1100 => LedControl::Mode1,
            0b1101 => LedControl::Mode2,
            0b1110 => LedControl::Mode3,
            0b1111 => LedControl::Mode4,
        }
    }

    pub fn set_led(led_no: u8, setting: LedControl) -> Self {
        self.0.set_bits((led_no*4)..=(led_no*4+3), setting as u8)
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