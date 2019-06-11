use crate::regs::{RegisterR, RegisterW};

macro_rules! def_reg_get {
    ($name:tt, $type: ty, $asm_instr:tt) => {
        impl RegisterR for $name {
            type R = $type;

            #[inline(always)]
            fn read(&self) -> Self::R {
                let mut value;
                unsafe { asm!($asm_instr : "=r" (value) ::: "volatile") }
                value
            }
        }
    }
}

macro_rules! def_reg_set {
    ($name:ty, $type:ty, $asm_instr:tt) => {
        impl RegisterW for $name {
            type W = $type;

            #[inline(always)]
            fn write(&mut self, value: Self::W) {
                unsafe { asm!($asm_instr :: "r" (value) :: "volatile") }
            }

            fn zeroed() -> Self::W {
                0
            }
        }
    }
}

/// Stack Pointer
pub struct SP;
def_reg_get!(SP, u32, "mov $0, sp");
def_reg_set!(SP, u32, "mov sp, $0");

/// Link register (function call return address)
pub struct LR;
def_reg_get!(LR, u32, "mov $0, lr");
def_reg_set!(LR, u32, "mov lr, $0");

pub struct MPIDR;
def_reg_get!(MPIDR, u32, "mrc p15, 0, $0, c0, c0, 5");

pub struct DFAR;
def_reg_get!(DFAR, u32, "mrc p15, 0, $0, c6, c0, 0");

pub struct DFSR;
def_reg_get!(DFSR, u32, "mrc p15, 0, $0, c5, c0, 0");

pub struct SCTLR;
def_reg_get!(SCTLR, u32, "mrc p15, 0, $0, c1, c0, 0");
def_reg_set!(SCTLR, u32, "mcr p15, 0, $0, c1, c0, 0");

/// Invalidate TLBs
#[inline(always)]
pub fn tlbiall() {
    unsafe {
        asm!("mcr p15, 0, $0, c8, c7, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate I-Cache
#[inline(always)]
pub fn iciallu() {
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate Branch Predictor Array
#[inline(always)]
pub fn bpiall() {
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 6" :: "r" (0) :: "volatile");
    }
}

/// Invalidate D-Cache
#[inline(always)]
pub fn dccisw() {
    // TODO: $0 is r11 at what value?
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 6" :: "r" (0) :: "volatile");
    }
}
