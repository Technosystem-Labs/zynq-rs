use bit_field::BitField;

pub trait SpiFlashRegister {
    fn inst_code() -> u8;
    fn new(src: u8) -> Self;
}

macro_rules! u8_register {
    ($name: ident, $doc: tt, $inst_code: expr) => {
        #[derive(Clone)]
        #[doc=$doc]
        pub struct $name {
            pub inner: u8,
        }

        impl SpiFlashRegister for $name {
            fn inst_code() -> u8 {
                $inst_code
            }

            fn new(src: u8) -> Self {
                $name {
                    inner: src,
                }
            }
        }

        impl $name {
            #[allow(unused)]
            pub fn is_zeroed(&self) -> bool {
                self.inner == 0
            }
        }
    };
}

u8_register!(CR, "Configuration Register", 0x35);
u8_register!(SR1, "Status Register-1", 0x05);
impl SR1 {
    /// Write In Progress
    pub fn wip(&self) -> bool {
        self.inner.get_bit(0)
    }

    /// Write Enable Latch
    pub fn wel(&self) -> bool {
        self.inner.get_bit(1)
    }

    /// Erase Error Occurred
    pub fn e_err(&self) -> bool {
        self.inner.get_bit(5)
    }

    /// Programming Error Occurred
    pub fn p_err(&self) -> bool {
        self.inner.get_bit(6)
    }
}

u8_register!(SR2, "Status Register-2", 0x07);
u8_register!(BA, "Bank Address Register", 0xB9);
