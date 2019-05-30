pub trait PhyAccess {
    fn read_phy(&mut self, addr: u8, reg: u8) -> u16;
    fn write_phy(&mut self, addr: u8, reg: u8, data: u16);
}

pub trait PhyDevice {
    fn init() -> Self;
}
