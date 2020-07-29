pub mod id;
use id::{identify_phy, PhyIdentifier};
mod status;
pub use status::Status;
mod control;
pub use control::Control;
mod pssr;
pub use pssr::PSSR;

#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub speed: LinkSpeed,
    pub duplex: LinkDuplex,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LinkSpeed {
    S10,
    S100,
    S1000,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LinkDuplex {
    Half,
    Full,
}

pub trait PhyAccess {
    fn read_phy(&mut self, addr: u8, reg: u8) -> u16;
    fn write_phy(&mut self, addr: u8, reg: u8, data: u16);
}

#[derive(Clone)]
pub struct Phy {
    pub addr: u8,
    device: PhyDevice,
}

#[derive(Clone, Copy)]
pub enum PhyDevice {
    Marvell88E1116R,
    Rtl8211E,
}

const OUI_MARVELL: u32 = 0x005043;
const OUI_REALTEK: u32 = 0x000732;

impl Phy {
    /// Probe all addresses on MDIO for a known PHY
    pub fn find<PA: PhyAccess>(pa: &mut PA) -> Option<Phy> {
        (1..32).filter_map(|addr| {
            match identify_phy(pa, addr) {
                Some(PhyIdentifier {
                    oui: OUI_MARVELL,
                    model: 36,
                    ..
                }) => Some(PhyDevice::Marvell88E1116R),
                Some(PhyIdentifier {
                    oui: OUI_REALTEK,
                    model: 0b010001,
                    rev: 0b0101,
                }) => Some(PhyDevice::Rtl8211E),
                _ => None,
            }.map(|device| Phy { addr, device })
        }).next()
    }

    pub fn name(&self) -> &'static str {
        match self.device {
            PhyDevice::Marvell88E1116R => &"Marvell 88E1116R",
            PhyDevice::Rtl8211E => &"RTL8211E",
        }
    }

    pub fn read_reg<PA, PR>(&self, pa: &mut PA) -> PR
    where
        PA: PhyAccess,
        PR: PhyRegister + From<u16>,
    {
        pa.read_phy(self.addr, PR::addr()).into()
    }

    pub fn modify_reg<PA, PR, F>(&self, pa: &mut PA, mut f: F)
    where
        PA: PhyAccess,
        PR: PhyRegister + From<u16> + Into<u16>,
        F: FnMut(PR) -> PR,
    {
        let reg = pa.read_phy(self.addr, PR::addr()).into();
        let reg = f(reg);
        pa.write_phy(self.addr, PR::addr(), reg.into())
    }

    pub fn modify_control<PA, F>(&self, pa: &mut PA, f: F)
    where
        PA: PhyAccess,
        F: FnMut(Control) -> Control,
    {
        self.modify_reg(pa, f)
    }

    pub fn get_control<PA: PhyAccess>(&self, pa: &mut PA) -> Control {
        self.read_reg(pa)
    }

    pub fn get_status<PA: PhyAccess>(&self, pa: &mut PA) -> Status {
        self.read_reg(pa)
    }

    pub fn get_link<PA: PhyAccess>(&self, pa: &mut PA) -> Option<Link> {
        let status = self.get_status(pa);
        if !status.link_status() {
            None
        } else if status.cap_1000base_t_extended_status() {
            let phy_status: PSSR = self.read_reg(pa);
            phy_status.get_link()
        } else {
            status.get_link()
        }
    }

    pub fn reset<PA: PhyAccess>(&self, pa: &mut PA) {
        self.modify_control(pa, |control|
            control.set_reset(true)
        );
        while self.get_control(pa).reset() {}
    }

    pub fn restart_autoneg<PA: PhyAccess>(&self, pa: &mut PA) {
        self.modify_control(pa, |control|
            control.set_autoneg_enable(true)
                .set_restart_autoneg(true)
        );
    }
}

pub trait PhyRegister {
    fn addr() -> u8;
}
