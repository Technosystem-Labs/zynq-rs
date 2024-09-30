pub mod id;
use id::{identify_phy, PhyIdentifier};
mod status;
pub use status::Status;
mod control;
pub use control::Control;
mod pssr;
pub use pssr::PSSR;
mod leds;
pub use leds::Leds;

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Led0Control {
    OnLinkOffNoLink = 0b0000,
    OnLinkBlinkActivityOffNoLink = 0b0001,
    BlinkDependingOnLink = 0b0010,
    OnActivityOffNoActivity = 0b0011,
    BlinkActivityOffNoActivity = 0b0100,
    OnTransmitOffNoTransmit = 0b0101,
    OnCopperLinkOffElse = 0b0110,
    On1000LinkOffElse = 0b0111,
    ForceOff = 0b1000,
    ForceOn = 0b1001,
    ForceHiZ = 0b1010,
    ForceBlink = 0b1011,
    Mode1 = 0b1100,
    Mode2 = 0b1101,
    Mode3 = 0b1110,
    Mode4 = 0b1111
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Led1Control {
    OnReceiveOffNoReceive = 0b0000,
    OnLinkBlinkActivityOffNoLink = 0b0001,
    OnLinkBlinkReceiveOffNoLink = 0b0010,
    OnActivityOffNoActivity = 0b0011,
    BlinkActivityOffNoActivity = 0b0100,
    On100OrFiberOffElse = 0b0101,
    On1001000LinkOffElse = 0b0110,
    On100LinkOffElse = 0b0111,
    ForceOff = 0b1000,
    ForceOn = 0b1001,
    ForceHiZ = 0b1010,
    ForceBlink = 0b1011,
}

pub trait PhyAccess {
    fn read_phy(&mut self, addr: u8, reg: u8) -> u16;
    fn write_phy(&mut self, addr: u8, reg: u8, data: u16);
}

pub trait PhyRegister {
    fn addr() -> u8;
    fn page() -> u8;
}


#[derive(Clone)]
pub struct Phy {
    pub addr: u8,
}

const OUI_MARVELL: u32 = 0x005043;
const OUI_REALTEK: u32 = 0x000732;
const OUI_LANTIQ : u32 = 0x355969;
const OUI_ICPLUS : u32 = 0x0090c3;

//only change pages on Kasli-SoC's Marvel 88E11xx
#[cfg(feature="target_kasli_soc")]
const PAGE_REGISTER: u8 = 0x16;

impl Phy {
    /// Probe all addresses on MDIO for a known PHY
    pub fn find<PA: PhyAccess>(pa: &mut PA) -> Option<Phy> {
        (0..32).find(|addr| {
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
                Some(PhyIdentifier {
                    oui: OUI_ICPLUS,
                    // IP101G-DS-R01
                    model: 5,
                    rev: 4,
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
        #[cfg(feature="target_kasli_soc")]
        pa.write_phy(self.addr, PAGE_REGISTER, PR::page().into());

        pa.read_phy(self.addr, PR::addr()).into()
    }

    pub fn modify_reg<PA, PR, F>(&self, pa: &mut PA, mut f: F)
    where
        PA: PhyAccess,
        PR: PhyRegister + From<u16> + Into<u16>,
        F: FnMut(PR) -> PR,
    {
        #[cfg(feature="target_kasli_soc")]
        pa.write_phy(self.addr, PAGE_REGISTER, PR::page().into());
        
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

    pub fn modify_leds<PA, F>(&self, pa: &mut PA, f: F)
    where
        PA: PhyAccess,
        F: FnMut(Leds) -> Leds,
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

    #[cfg(feature="target_kasli_soc")]
    pub fn set_leds<PA: PhyAccess>(&self, pa: &mut PA) {
        self.modify_leds(pa, |leds|
            leds.set_led0(Led0Control::OnCopperLinkOffElse)
                .set_led1(Led1Control::BlinkActivityOffNoActivity)
        );
    }
}
