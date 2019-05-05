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

pub struct SP;
def_reg_get!(SP, u32, "mov $0, sp");
def_reg_set!(SP, u32, "mov sp, $0");

pub struct MPIDR;
def_reg_get!(MPIDR, u32, "mrc p15, 0, $0, c0, c0, 5");
