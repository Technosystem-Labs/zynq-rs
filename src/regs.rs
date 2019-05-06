//! Interface to peripheral registers akin to the code that svd2rust
//! generates.

use volatile_register::{RO, WO, RW};
use bit_field::BitField;

pub trait Register {
    type R;
    type W;

    fn read(&self) -> Self::R;
    fn write(&self, w: Self::W);
    fn modify<F: FnOnce(Self::R, Self::W) -> Self::W>(&self, f: F);
}

#[macro_export]
macro_rules! register {
    ($mod_name: ident, $struct_name: ident, $inner: ty) => (
        #[repr(packed)]
        pub struct $struct_name {
            inner: RW<$inner>,
        }

        pub mod $mod_name {
            pub struct Read {
                pub inner: $inner,
            }
            pub struct Write {
                pub inner: $inner,
            }
        }

        impl $struct_name {
            pub fn zeroed() -> $mod_name::Write {
                $mod_name::Write { inner: 0 }
            }
        }

        impl crate::regs::Register for $struct_name {
            type R = $mod_name::Read;
            type W = $mod_name::Write;

            fn read(&self) -> Self::R {
                let inner = self.inner.read();
                $mod_name::Read { inner }
            }

            fn write(&self, w: Self::W) {
                unsafe {
                    self.inner.write(w.inner);
                }
            }

            fn modify<F: FnOnce(Self::R, Self::W) -> Self::W>(&self, f: F) {
                unsafe {
                    self.inner.modify(|inner| {
                        f($mod_name::Read { inner }, $mod_name::Write { inner })
                            .inner
                    });
                }
            }
        }
    );
}

#[macro_export]
macro_rules! register_bit {
    ($mod_name: ident, $name: ident, $bit: expr) => (
        impl $mod_name::Read {
            fn $name(&self) -> bool {
                use bit_field::BitField;

                self.inner.get_bit($bit)
            }
        }

        impl $mod_name::Write {
            fn $name(mut self, value: bool) -> Self {
                use bit_field::BitField;

                self.inner.set_bit($bit, value);
                self
            }
        }
    );
}

#[macro_export]
macro_rules! register_bits {
    ($mod_name: ident, $name: ident, $type: ty, $bit_begin: expr, $bit_end: expr) => (
        impl $mod_name::Read {
            fn $name(&self) -> $type {
                use bit_field::BitField;

                self.inner.get_bits($bit_begin..=$bit_end) as $type
            }
        }

        impl $mod_name::Write {
            fn $name(mut self, value: $type) -> Self {
                use bit_field::BitField;

                self.inner.set_bits($bit_begin..=$bit_end, value.into());
                self
            }
        }
    );
}
