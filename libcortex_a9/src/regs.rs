use core::arch::asm;

use libregister::{RegisterR, RegisterRW, RegisterW, register_bit, register_bits};

macro_rules! def_reg_r {
    ($name:tt, $type: ty, $asm_instr:tt) => {
        impl RegisterR for $name {
            type R = $type;

            #[inline]
            fn read(&self) -> Self::R {
                let mut value: u32;
                unsafe { asm!($asm_instr, lateout(reg) value) }
                value.into()
            }
        }
    }
}

macro_rules! def_reg_w {
    ($name:ty, $type:ty, $asm_instr:tt) => {
        impl RegisterW for $name {
            type W = $type;

            #[inline]
            fn write(&mut self, value: Self::W) {
                let value: u32 = value.into();
                unsafe { asm!($asm_instr, in(reg) value) }
            }

            #[inline]
            fn zeroed() -> Self::W {
                0u32.into()
            }
        }
    }
}

macro_rules! wrap_reg {
    ($mod_name:ident) => {
        pub mod $mod_name {
            pub struct Read {
                pub inner: u32,
            }
            impl From<u32> for Read {
                #[inline]
                fn from(value: u32) -> Self {
                    Read { inner: value }
                }
            }

            pub struct Write {
                pub inner: u32,
            }
            impl From<u32> for Write {
                #[inline]
                fn from(value: u32) -> Self {
                    Write { inner: value }
                }
            }
            impl Into<u32> for Write {
                #[inline]
                fn into(self) -> u32 {
                    self.inner
                }
            }
        }
    };
}

/// Stack Pointer
pub struct SP;
def_reg_r!(SP, u32, "mov {}, sp");
def_reg_w!(SP, u32, "mov sp, {}");

/// Link register (function call return address)
pub struct LR;
def_reg_r!(LR, u32, "mov {}, lr");
def_reg_w!(LR, u32, "mov lr, {}");

pub struct VBAR;
def_reg_r!(VBAR, u32, "mrc p15, 0, {}, c12, c0, 0");
def_reg_w!(VBAR, u32, "mcr p15, 0, {}, c12, c0, 0");

pub struct MVBAR;
def_reg_r!(MVBAR, u32, "mrc p15, 0, {}, c12, c0, 1");
def_reg_w!(MVBAR, u32, "mcr p15, 0, {}, c12, c0, 1");

pub struct HVBAR;
def_reg_r!(HVBAR, u32, "mrc p15, 4, {}, c12, c0, 0");
def_reg_w!(HVBAR, u32, "mcr p15, 4, {}, c12, c0, 0");

/// Multiprocess Affinity Register
pub struct MPIDR;
def_reg_r!(MPIDR, mpidr::Read, "mrc p15, 0, {}, c0, c0, 5");
wrap_reg!(mpidr);
register_bits!(mpidr,
               /// CPU core index
               cpu_id, u8, 0, 1);
register_bits!(mpidr,
               /// Processor index in "multi-socket" systems
               cluster_id, u8, 8, 11);
register_bit!(mpidr,
              /// true if part of uniprocessor system
              u, 30);

pub struct DFAR;
def_reg_r!(DFAR, u32, "mrc p15, 0, {}, c6, c0, 0");

pub struct DFSR;
def_reg_r!(DFSR, u32, "mrc p15, 0, {}, c5, c0, 0");

pub struct SCTLR;
wrap_reg!(sctlr);
def_reg_r!(SCTLR, sctlr::Read, "mrc p15, 0, {}, c1, c0, 0");
def_reg_w!(SCTLR, sctlr::Write, "mcr p15, 0, {}, c1, c0, 0");
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

/// Auxiliary Control Register
pub struct ACTLR;
wrap_reg!(actlr);
def_reg_r!(ACTLR, actlr::Read, "mrc p15, 0, {}, c1, c0, 1");
def_reg_w!(ACTLR, actlr::Write, "mcr p15, 0, {}, c1, c0, 1");
// SMP bit
register_bit!(actlr, parity_on, 9);
register_bit!(actlr, alloc_one_way, 8);
register_bit!(actlr, excl, 7);
register_bit!(actlr, smp, 6);
register_bit!(actlr, write_full_line_of_zeros, 3);
register_bit!(actlr, l1_prefetch_enable, 2);
// L2 cache prefetch hint, in UG585 section 3.4.8
register_bit!(actlr, l2_prefetch_enable, 1);
// Cache/TLB maintenance broadcast
register_bit!(actlr, fw, 0);

impl RegisterRW for ACTLR {
    #[inline]
    fn modify<F: FnOnce(Self::R, Self::W) -> Self::W>(&mut self, f: F) {
        let r = self.read();
        let w = actlr::Write { inner: r.inner };
        let w = f(r, w);
        self.write(w);
    }
}

impl ACTLR {
    pub fn enable_smp(&mut self) {
        self.modify(|_, w| w.smp(true).fw(true).alloc_one_way(true));
    }

    pub fn enable_prefetch(&mut self) {
        self.modify(|_, w| w.l1_prefetch_enable(true).l2_prefetch_enable(true))
    }
}

/// Domain Access Control Register
pub struct DACR;
def_reg_r!(DACR, u32, "mrc p15, 0, {}, c3, c0, 0");
def_reg_w!(DACR, u32, "mcr p15, 0, {}, c3, c0, 0");

/// Translation Table Base Register 0
pub struct TTBR0;
/// Translation Table Base Register 1
pub struct TTBR1;
def_reg_r!(TTBR0, ttbr::Read, "mrc p15, 0, {}, c2, c0, 0");
def_reg_w!(TTBR0, ttbr::Write, "mcr p15, 0, {}, c2, c0, 0");
def_reg_r!(TTBR1, ttbr::Read, "mrc p15, 0, {}, c2, c0, 1");
def_reg_w!(TTBR1, ttbr::Write, "mcr p15, 0, {}, c2, c0, 1");
wrap_reg!(ttbr);
register_bits!(ttbr, table_base, u32, 14, 31);
register_bit!(ttbr, irgn0, 6);
register_bits!(ttbr, rgn, u8, 3, 4);
register_bit!(ttbr,
              /// Translation table walk to shared memory?
              s, 1);
register_bit!(ttbr, irgn1, 0);
