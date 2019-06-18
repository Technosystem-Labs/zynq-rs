pub mod id;
use id::{identify_phy, PhyIdentifier};

pub trait PhyAccess {
    fn read_phy(&mut self, addr: u8, reg: u8) -> u16;
    fn write_phy(&mut self, addr: u8, reg: u8, data: u16);
}

pub enum Phy {
    Marvel88E1116R,
    Rtl8211E,
}

const OUI_MARVEL: u32 = 0x005043;
const OUI_REALTEK: u32 = 0x000732;

impl Phy {
    /// Probe all addresses on MDIO for a known PHY
    pub fn find<PA: PhyAccess>(pa: &mut PA) -> Option<(u8, Phy)> {
        for addr in 1..32 {
            match identify_phy(pa, addr) {
                Some(PhyIdentifier {
                    oui: OUI_MARVEL,
                    model: 36,
                    ..
                }) => return Some((addr, Phy::Marvel88E1116R)),
                Some(PhyIdentifier {
                    oui: OUI_REALTEK,
                    model: 0b010001,
                    rev: 0b0101,
                }) => return Some((addr, Phy::Rtl8211E)),
                _ => {}
            }
        }

        None
    }

    pub fn name(&self) -> &'static str {
        match self {
            Phy::Marvel88E1116R => &"Marvel 88E1116R",
            Phy::Rtl8211E => &"RTL8211E",
        }
    }
}
