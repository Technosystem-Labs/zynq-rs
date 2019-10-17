use crate::{register_bit, register_bits};
use crate::regs::{RegisterR, RegisterW};

macro_rules! def_reg_r {
    ($name:tt, $type: ty, $asm_instr:tt) => {
        impl RegisterR for $name {
            type R = $type;

            #[inline(always)]
            fn read(&self) -> Self::R {
                let mut value: u32;
                unsafe { asm!($asm_instr : "=r" (value) ::: "volatile") }
                value.into()
            }
        }
    }
}

macro_rules! def_reg_w {
    ($name:ty, $type:ty, $asm_instr:tt) => {
        impl RegisterW for $name {
            type W = $type;

            #[inline(always)]
            fn write(&mut self, value: Self::W) {
                let value: u32 = value.into();
                unsafe { asm!($asm_instr :: "r" (value) :: "volatile") }
            }

            fn zeroed() -> Self::W {
                0u32.into()
            }
        }
    }
}

macro_rules! wrap_reg {
    ($mod_name: ident) => {
        pub mod $mod_name {
            pub struct Read {
                pub inner: u32,
            }
            impl From<u32> for Read {
                fn from(value: u32) -> Self {
                    Read { inner: value }
                }
            }

            pub struct Write {
                pub inner: u32,
            }
            impl From<u32> for Write {
                fn from(value: u32) -> Self {
                    Write { inner: value }
                }
            }
            impl Into<u32> for Write {
                fn into(self) -> u32 {
                    self.inner
                }
            }
        }
    }
}

/// Stack Pointer
pub struct SP;
def_reg_r!(SP, u32, "mov $0, sp");
def_reg_w!(SP, u32, "mov sp, $0");

/// Link register (function call return address)
pub struct LR;
def_reg_r!(LR, u32, "mov $0, lr");
def_reg_w!(LR, u32, "mov lr, $0");

pub struct MPIDR;
def_reg_r!(MPIDR, u32, "mrc p15, 0, $0, c0, c0, 5");

pub struct DFAR;
def_reg_r!(DFAR, u32, "mrc p15, 0, $0, c6, c0, 0");

pub struct DFSR;
def_reg_r!(DFSR, u32, "mrc p15, 0, $0, c5, c0, 0");

pub struct SCTLR;
wrap_reg!(sctlr);
def_reg_r!(SCTLR, sctlr::Read, "mrc p15, 0, $0, c1, c0, 0");
def_reg_w!(SCTLR, sctlr::Write, "mcr p15, 0, $0, c1, c0, 0");
register_bit!(sctlr,
              /// Enables MMU
              m, 0);
register_bit!(sctlr,
              /// Strict Alignment Checking
              a, 1);
register_bit!(sctlr,
              /// Data Caching
              c, 2);
register_bit!(sctlr, sw, 10);
register_bit!(sctlr, z, 11);
register_bit!(sctlr, i, 12);
register_bit!(sctlr, v, 13);
register_bit!(sctlr, ha, 17);
register_bit!(sctlr, ee, 25);
register_bit!(sctlr,
              /// (read-only)
              nmfi, 27);
register_bit!(sctlr, unaligned, 22);
register_bit!(sctlr,
              /// TEX Remap Enable
              tre, 28);
register_bit!(sctlr,
              /// Access Flag Enable
              afe, 29);
register_bit!(sctlr,
              /// Thumb Exception Enable
              te, 30);

/// Domain Access Control Register
pub struct DACR;
def_reg_r!(DACR, u32, "mrc p15, 0, $0, c3, c0, 0");
def_reg_w!(DACR, u32, "mcr p15, 0, $0, c3, c0, 0");

/// Translation Table Base Register 0
pub struct TTBR0;
/// Translation Table Base Register 1
pub struct TTBR1;
def_reg_r!(TTBR0, ttbr::Read, "mrc p15, 0, $0, c2, c0, 0");
def_reg_w!(TTBR0, ttbr::Write, "mcr p15, 0, $0, c2, c0, 0");
def_reg_r!(TTBR1, ttbr::Read, "mrc p15, 0, $0, c2, c0, 1");
def_reg_w!(TTBR1, ttbr::Write, "mcr p15, 0, $0, c2, c0, 1");
wrap_reg!(ttbr);
register_bits!(ttbr, table_base, u32, 14, 31);
register_bit!(ttbr, irgn0, 6);
register_bits!(ttbr, rgn, u8, 3, 4);
register_bit!(ttbr,
              /// Translation table walk to shared memory?
              s, 1);
register_bit!(ttbr, irgn1, 0);
