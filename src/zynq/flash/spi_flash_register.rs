use bit_field::BitField;

pub trait SpiFlashRegister {
    fn inst_code() -> u8;
    fn transfer_len() -> usize;
    fn new<I: Iterator<Item=u8>>(src: I) -> Self;
}

macro_rules! u8_register {
    ($name: ident, $inst_code: expr) => {
        #[derive(Clone)]
        pub struct $name {
            pub inner: u8,
        }

        impl SpiFlashRegister for $name {
            fn inst_code() -> u8 {
                $inst_code
            }

            fn transfer_len() -> usize {
                2
            }

            fn new<I: Iterator<Item=u8>>(mut src: I) -> Self {
                $name {
                    inner: src.next().unwrap(),
                }
            }
        }
    };
}

u8_register!(CR, 0x35);
u8_register!(SR1, 0x05);
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

u8_register!(SR2, 0x07);
