pub trait ReadableRegister<T> {
    fn get(&self) -> T;
}

macro_rules! def_reg_get {
    ($name:ty, $type:ty, $asm_instr:tt) => {
        impl ReadableRegister<$type> for $name {
            #[inline(always)]
            fn get(&self) -> $type {
                let mut value;
                unsafe { asm!($asm_instr : "=r" (value) ::: "volatile") }
                value
            }
        }
    }
}

pub trait WritableRegister<T> {
    fn set(&self, value: T);
}

macro_rules! def_reg_set {
    ($name:ty, $type:ty, $asm_instr:tt) => {
        impl WritableRegister<$type> for $name {
            #[inline(always)]
            fn set(&self, value: $type) {
                unsafe { asm!($asm_instr :: "r" (value) :: "volatile") }
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

/// Invalidate TLBs
pub fn tlbiall() {
    unsafe {
        asm!("mcr p15, 0, $0, c8, c7, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate I-Cache
pub fn iciallu() {
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 0" :: "r" (0) :: "volatile");
    }
}

/// Invalidate Branch Predictor Array
pub fn bpiall() {
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 6" :: "r" (0) :: "volatile");
    }
}

/// Invalidate D-Cache
pub fn dccisw() {
    // TODO: $0 is r11 at what value?
    unsafe {
        asm!("mcr p15, 0, $0, c7, c5, 6" :: "r" (0) :: "volatile");
    }
}

/// Enable I-Cache and D-Cache
pub fn sctlr() {
    unsafe {
        asm!("mcr p15, 0, $0, c1, c0, 0" :: "r" (0x00401004) :: "volatile");
    }
}
