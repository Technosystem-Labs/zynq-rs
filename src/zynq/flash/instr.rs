use super::Transfer;

pub trait Instruction {
    type Args: Iterator<Item = u32>;
    type Result: for<'r, 't> From<&'t mut Transfer<'r, Self::Args>>;
    fn inst_code() -> u8;
    fn args(&self) -> Self::Args;
}

pub struct ReadId;

impl Instruction for ReadId {
    type Result = u32; // TODO: u8;
    type Args = core::iter::Empty<u32>;
    
    fn inst_code() -> u8 {
        0x9f
    }
    fn args(&self) -> Self::Args {
        core::iter::empty()
    }
}

/// Read configuration register
pub struct RdCr;

impl Instruction for RdCr {
    type Result = u8;
    type Args = core::iter::Empty<u32>;
    
    fn inst_code() -> u8 {
        0x35
    }
    fn args(&self) -> Self::Args {
        core::iter::empty()
    }
}
