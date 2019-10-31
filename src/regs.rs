//! Type-safe interface to peripheral registers akin to the code that
//! svd2rust generates.
#![allow(unused)]

use vcell::VolatileCell;
use volatile_register::{RO, WO, RW};
use bit_field::BitField;

/// A readable register
pub trait RegisterR {
    /// Type-safe reader for the register value
    type R;

    fn read(&self) -> Self::R;
}
/// A writable register
pub trait RegisterW {
    /// Type-safe writer to the register value
    type W;

    fn zeroed() -> Self::W;
    fn write(&mut self, w: Self::W);
}
/// A modifiable register
pub trait RegisterRW: RegisterR + RegisterW {
    fn modify<F: FnOnce(<Self as RegisterR>::R, <Self as RegisterW>::W) -> <Self as RegisterW>::W>(&mut self, f: F);
}

#[doc(hidden)]
#[macro_export]
macro_rules! register_common {
    ($mod_name: ident, $struct_name: ident, $access: ty, $inner: ty) => (
        #[repr(C)]
        pub struct $struct_name {
            inner: $access,
        }

        pub mod $mod_name {
            #[derive(Clone)]
            pub struct Read {
                pub inner: $inner,
            }
            #[derive(Clone)]
            pub struct Write {
                pub inner: $inner,
            }
        }
    );
}
#[doc(hidden)]
#[macro_export]
macro_rules! register_r {
    ($mod_name: ident, $struct_name: ident) => (
        impl crate::regs::RegisterR for $struct_name {
            type R = $mod_name::Read;

            fn read(&self) -> Self::R {
                let inner = self.inner.read();
                $mod_name::Read { inner }
            }
        }
    );
}
#[doc(hidden)]
#[macro_export]
macro_rules! register_w {
    ($mod_name: ident, $struct_name: ident) => (
        impl crate::regs::RegisterW for $struct_name {
            type W = $mod_name::Write;

            fn zeroed() -> $mod_name::Write {
                $mod_name::Write { inner: 0 }
            }

            fn write(&mut self, w: Self::W) {
                unsafe {
                    self.inner.write(w.inner);
                }
            }
        }
     );
}
#[doc(hidden)]
#[macro_export]
macro_rules! register_rw {
    ($mod_name: ident, $struct_name: ident) => (
        impl crate::regs::RegisterRW for $struct_name {
            fn modify<F: FnOnce(Self::R, Self::W) -> Self::W>(&mut self, f: F) {
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

#[doc(hidden)]
#[macro_export]
macro_rules! register_vcell {
    ($mod_name: ident, $struct_name: ident) => (
        impl crate::regs::RegisterR for $struct_name {
            type R = $mod_name::Read;

            fn read(&self) -> Self::R {
                let inner = self.inner.get();
                $mod_name::Read { inner }
            }
        }
        impl crate::regs::RegisterW for $struct_name {
            type W = $mod_name::Write;

            fn zeroed() -> $mod_name::Write {
                $mod_name::Write { inner: 0 }
            }

            fn write(&mut self, w: Self::W) {
                self.inner.set(w.inner);
            }
        }
        impl crate::regs::RegisterRW for $struct_name {
            fn modify<F: FnOnce(Self::R, Self::W) -> Self::W>(&mut self, f: F) {
                let r = self.read();
                let w = $mod_name::Write { inner: r.inner };
                let w = f(r, w);
                self.write(w);
            }
        }
    );
}

/// Main macro for register definition
#[macro_export]
macro_rules! register {
    // Define read-only register
    ($mod_name: ident, $struct_name: ident, RO, $inner: ty) => (
        crate::register_common!($mod_name, $struct_name, volatile_register::RO<$inner>, $inner);
        crate::register_r!($mod_name, $struct_name);
    );

    // Define write-only register
    ($mod_name: ident, $struct_name: ident, WO, $inner: ty) => (
        crate::register_common!($mod_name, $struct_name, volatile_register::WO<$inner>, $inner);
        crate::register_w!($mod_name, $struct_name);
    );

    // Define read-write register
    ($mod_name: ident, $struct_name: ident, RW, $inner: ty) => (
        crate::register_common!($mod_name, $struct_name, volatile_register::RW<$inner>, $inner);
        crate::register_r!($mod_name, $struct_name);
        crate::register_w!($mod_name, $struct_name);
        crate::register_rw!($mod_name, $struct_name);
    );

    // Define read-write register
    ($mod_name: ident, $struct_name: ident, VolatileCell, $inner: ty) => (
        crate::register_common!($mod_name, $struct_name, VolatileCell<$inner>, $inner);
        crate::register_vcell!($mod_name, $struct_name);
    );
}

/// Define a 1-bit field of a register
#[macro_export]
macro_rules! register_bit {
    ($mod_name: ident, $(#[$outer:meta])* $name: ident, $bit: expr) => (
        $(#[$outer])*
        impl $mod_name::Read {
            #[allow(unused)]
            pub fn $name(&self) -> bool {
                use bit_field::BitField;

                self.inner.get_bit($bit)
            }
        }

        $(#[$outer])*
        impl $mod_name::Write {
            #[allow(unused)]
            pub fn $name(mut self, value: bool) -> Self {
                use bit_field::BitField;

                self.inner.set_bit($bit, value);
                self
            }
        }
    );
}

/// Define a multi-bit field of a register
#[macro_export]
macro_rules! register_bits {
    ($mod_name: ident, $(#[$outer:meta])* $name: ident, $type: ty, $bit_begin: expr, $bit_end: expr) => (
        impl $mod_name::Read {
            #[allow(unused)]
            $(#[$outer])*
            pub fn $name(&self) -> $type {
                use bit_field::BitField;

                self.inner.get_bits($bit_begin..=$bit_end) as $type
            }
        }

        #[allow(unused)]
        $(#[$outer])*
        impl $mod_name::Write {
            #[allow(unused)]
            pub fn $name(mut self, value: $type) -> Self {
                use bit_field::BitField;

                self.inner.set_bits($bit_begin..=$bit_end, value.into());
                self
            }
        }
    );
}

/// Define a multi-bit field of a register, coerced to a certain type
///
/// Because read bits are just transmuted to the `$type`, its
/// definition must be annotated with `#[repr($bit_type)]`!
#[macro_export]
macro_rules! register_bits_typed {
    ($mod_name: ident, $(#[$outer:meta])* $name: ident, $bit_type: ty, $type: ty, $bit_begin: expr, $bit_end: expr) => (
        impl $mod_name::Read {
            #[allow(unused)]
            $(#[$outer])*
            pub fn $name(&self) -> $type {
                use bit_field::BitField;

                let bits = self.inner.get_bits($bit_begin..=$bit_end) as $bit_type;
                unsafe { core::mem::transmute(bits) }
            }
        }

        impl $mod_name::Write {
            #[allow(unused)]
            $(#[$outer])*
            pub fn $name(mut self, value: $type) -> Self {
                use bit_field::BitField;

                let bits = (value as $bit_type).into();
                self.inner.set_bits($bit_begin..=$bit_end, bits);
                self
            }
        }
    );
}

#[macro_export]
macro_rules! register_at {
    ($name: ident, $addr: expr, $ctor: ident) => (
        impl $name {
            #[allow(unused)]
            pub fn $ctor() -> &'static mut Self {
                let addr = $addr as *mut Self;
                unsafe { &mut *addr }
            }
        }
    )
}
