use bit_field::BitField;
use super::{PhyRegister, Link, LinkDuplex, LinkSpeed};

#[derive(Clone, Copy, Debug)]
/// PHY-Specific Status Register
pub struct PSSR(pub u16);

impl PSSR {
    pub fn link(&self) -> bool {
        self.0.get_bit(10)
    }

    pub fn duplex(&self) -> LinkDuplex {
        if self.0.get_bit(13) {
            LinkDuplex::Full
        } else {
            LinkDuplex::Half
        }
    }

    pub fn speed(&self) -> Option<LinkSpeed> {
        match self.0.get_bits(14..=15) {
            0b00 => Some(LinkSpeed::S10),
            0b01 => Some(LinkSpeed::S100),
            0b10 => Some(LinkSpeed::S1000),
            _ => None,
        }
    }

    pub fn get_link(&self) -> Option<Link> {
        if self.link() {
            Some(Link {
                speed: self.speed()?,
                duplex: self.duplex(),
            })
        } else {
            None
        }
    }
}

impl PhyRegister for PSSR {
    fn addr() -> u8 {
        0x11
    }
        
    fn page() -> u8 {
        0
    }
}

impl From<u16> for PSSR {
    fn from(value: u16) -> Self {
        PSSR(value)
    }
}
