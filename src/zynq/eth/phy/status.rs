use bit_field::BitField;
use super::{PhyRegister, Link, LinkDuplex, LinkSpeed};

#[derive(Clone, Copy, Debug)]
/// Basic Mode Status Register
pub struct Status(pub u16);

impl Status {
    pub fn extended_capability(&self) -> bool {
        self.0.get_bit(0)
    }
    pub fn jabber_detect(&self) -> bool {
        self.0.get_bit(1)
    }
    pub fn link_status(&self) -> bool {
        self.0.get_bit(2)
    }
    pub fn autoneg_ability(&self) -> bool {
        self.0.get_bit(3)
    }
    pub fn remote_fault(&self) -> bool {
        self.0.get_bit(4)
    }
    pub fn autoneg_complete(&self) -> bool {
        self.0.get_bit(5)
    }
    pub fn preamble_suppression(&self) -> bool {
        self.0.get_bit(6)
    }
    pub fn cap_1000base_t_extended_status(&self) -> bool {
        self.0.get_bit(8)
    }
    pub fn cap_10base_t2_half(&self) -> bool {
        self.0.get_bit(9)
    }
    pub fn cap_10base_t2_full(&self) -> bool {
        self.0.get_bit(10)
    }
    pub fn cap_10base_t_half(&self) -> bool {
        self.0.get_bit(11)
    }
    pub fn cap_10base_t_full(&self) -> bool {
        self.0.get_bit(12)
    }
    pub fn cap_100base_tx_half(&self) -> bool {
        self.0.get_bit(13)
    }
    pub fn cap_100base_tx_full(&self) -> bool {
        self.0.get_bit(14)
    }
    pub fn cap_100base_t4(&self) -> bool {
        self.0.get_bit(15)
    }

    pub fn get_link(&self) -> Option<Link> {
        if ! self.link_status() {
            None
        } else if self.cap_10base_t_half() {
            Some(Link {
                speed: LinkSpeed::S10,
                duplex: LinkDuplex::Half,
            })
        } else if self.cap_10base_t_full() {
            Some(Link {
                speed: LinkSpeed::S10,
                duplex: LinkDuplex::Full,
            })
        } else if self.cap_10base_t2_half() {
            Some(Link {
                speed: LinkSpeed::S10,
                duplex: LinkDuplex::Half,
            })
        } else if self.cap_10base_t2_full() {
            Some(Link {
                speed: LinkSpeed::S10,
                duplex: LinkDuplex::Full,
            })
        } else if self.cap_100base_t4() {
            Some(Link {
                speed: LinkSpeed::S100,
                duplex: LinkDuplex::Half,
            })
        } else if self.cap_100base_tx_half() {
            Some(Link {
                speed: LinkSpeed::S100,
                duplex: LinkDuplex::Half,
            })
        } else if self.cap_100base_tx_full() {
            Some(Link {
                speed: LinkSpeed::S100,
                duplex: LinkDuplex::Full,
            })
        } else {
            None
        }
    }
}

impl PhyRegister for Status {
    fn addr() -> u8 {
        1
    }
}

impl From<u16> for Status {
    fn from(value: u16) -> Self {
        Status(value)
    }
}
