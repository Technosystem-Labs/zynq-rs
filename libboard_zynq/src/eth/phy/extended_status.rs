use bit_field::BitField;
use super::{PhyRegister, Link, LinkDuplex, LinkSpeed};

#[derive(Clone, Copy, Debug)]
/// 1000Base-T Extended Status Register
pub struct ExtendedStatus(pub u16);

impl ExtendedStatus {
    pub fn cap_1000base_t_half(&self) -> bool {
        self.0.get_bit(12)
    }
    pub fn cap_1000base_t_full(&self) -> bool {
        self.0.get_bit(13)
    }
    pub fn cap_1000base_x_half(&self) -> bool {
        self.0.get_bit(14)
    }
    pub fn cap_1000base_x_full(&self) -> bool {
        self.0.get_bit(12)
    }

    pub fn get_link(&self) -> Option<Link> {
        if self.cap_1000base_t_half() {
            Some(Link {
                speed: LinkSpeed::S1000,
                duplex: LinkDuplex::Half,
            })
        } else if self.cap_1000base_t_full() {
            Some(Link {
                speed: LinkSpeed::S1000,
                duplex: LinkDuplex::Full,
            })
        } else if self.cap_1000base_x_half() {
            Some(Link {
                speed: LinkSpeed::S1000,
                duplex: LinkDuplex::Half,
            })
        } else if self.cap_1000base_x_full() {
            Some(Link {
                speed: LinkSpeed::S1000,
                duplex: LinkDuplex::Full,
            })
        } else {
            None
        }
    }
}

impl PhyRegister for ExtendedStatus {
    fn addr() -> u8 {
        0xF
    }
}

impl From<u16> for ExtendedStatus {
    fn from(value: u16) -> Self {
        ExtendedStatus(value)
    }
}
