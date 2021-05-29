pub mod id;
mod status;
pub use status::Status;
mod control;
pub use control::Control;
mod pssr;
pub use pssr::PSSR;

#[derive(Copy, Clone, Debug, PartialEq)]
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

pub trait PhyRegister {
    fn addr() -> u8;
}

#[cfg(not(feature = "target_kasli_soc"))]
mod phy_impl {
    use super::*;
    use id::{identify_phy, PhyIdentifier};

    #[derive(Clone)]
    pub struct Phy {
        pub addr: u8,
    }

    const OUI_MARVELL: u32 = 0x005043;
    const OUI_REALTEK: u32 = 0x000732;
    const OUI_LANTIQ : u32 = 0x355969;

    impl Phy {
        /// Probe all addresses on MDIO for a known PHY
        pub fn find<PA: PhyAccess>(pa: &mut PA) -> Option<Phy> {
            (1..32).find(|addr| {
                match identify_phy(pa, *addr) {
                    Some(PhyIdentifier {
                        oui: OUI_MARVELL,
                        // Marvell 88E1116R
                        model: 36,
                        ..
                    }) => true,
                    Some(PhyIdentifier {
                        oui: OUI_MARVELL,
                        // Marvell 88E1512
                        model: 29,
                        ..
                    }) => true,
                    Some(PhyIdentifier {
                        oui: OUI_REALTEK,
                        // RTL 8211E
                        model: 0b010001,
                        rev: 0b0101,
                    }) => true,
                    Some(PhyIdentifier {
                        oui: OUI_LANTIQ,
                        // Intel XWAY PHY11G (PEF 7071/PEF 7072) v1.5 / v1.6
                        model: 0,
                        ..
                    }) => true,
                    _ => false,
                }
            }).map(|addr| Phy { addr })
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
}

#[cfg(feature = "target_kasli_soc")]
mod phy_impl {
    use super::*;

    #[derive(Clone)]
    pub struct Phy {
    }

    impl Phy {
        pub fn find<PA: PhyAccess>(_pa: &mut PA) -> Option<Phy> {
            Some(Phy {})
        }

        pub fn get_link<PA: PhyAccess>(&self, _pa: &mut PA) -> Option<Link> {
            Some(Link {
                speed: LinkSpeed::S1000,
                duplex: LinkDuplex::Full,
            })
        }

        pub fn modify_control<PA, F>(&self, _pa: &mut PA, _f: F)
        where
            PA: PhyAccess,
            F: FnMut(Control) -> Control,
        {
        }

        pub fn reset<PA: PhyAccess>(&self, _pa: &mut PA) {
        }

        pub fn restart_autoneg<PA: PhyAccess>(&self, _pa: &mut PA) {
        }
    }
}

pub use phy_impl::*;
