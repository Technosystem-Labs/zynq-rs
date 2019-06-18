use bit_field::BitField;

use super::PhyAccess;


#[derive(Clone, Debug)]
pub struct PhyIdentifier {
    pub oui: u32,
    pub model: u8,
    pub rev: u8,
}

pub fn identify_phy<PA: PhyAccess>(pa: &mut PA, addr: u8) -> Option<PhyIdentifier> {
    let id1 = pa.read_phy(addr, 2);
    let id2 = pa.read_phy(addr, 3);
    if id1 != 0xFFFF || id2 != 0xFFFF {
        let mut oui = 0;
        oui.set_bits(6..=21, id1.get_bits(0..=15).into());
        oui.set_bits(0..=5, id2.get_bits(10..=15).into());
        Some(PhyIdentifier {
            oui,
            model: id2.get_bits(4..=9) as u8,
            rev: id2.get_bits(0..=3) as u8,
        })
    } else {
        None
    }
}
