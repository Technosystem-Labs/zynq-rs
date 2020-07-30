//! ARM Generic Interrupt Controller

use bit_field::BitField;
use libregister::{RegisterW, RegisterRW};
use super::mpcore;

#[derive(Debug, Clone, Copy)]
pub struct InterruptId(u8);

#[derive(Debug, Clone, Copy)]
pub enum InterruptSensitivity {
    Level,
    Edge,
}

pub struct InterruptController {
    mpcore: mpcore::RegisterBlock,
}

impl InterruptController {
    pub fn new(mpcore: mpcore::RegisterBlock) -> Self {
        InterruptController { mpcore }
    }

    pub fn disable_interrupts(&mut self) {
        self.mpcore.iccicr.modify(|_, w| w.enable_ns(false)
                                          .enable_s(false));
        self.mpcore.icddcr.modify(|_, w| w.enable_secure(false)
                                          .enable_non_secure(false));
    }

    /// enable interrupt signaling
    pub fn enable_interrupts(&mut self) {
        self.mpcore.iccicr.modify(|_, w| w.enable_ns(true)
                                          .enable_s(true));
        self.mpcore.icddcr.modify(|_, w| w.enable_secure(true));
    }

    pub fn enable(&mut self, id: InterruptId, target_cpu: u32, sensitivity: InterruptSensitivity) {
        assert!(target_cpu < 2);

        self.disable_interrupts();

        // enable
        let m = (id.0 >> 5) as usize;
        let n = (id.0 & 0x1F) as usize;
        assert!(m < 3);
        unsafe {
            self.mpcore.icdiser[m].modify(|mut icdiser| *icdiser.set_bit(n, true));
        }

        // target cpu
        let m = (id.0 >> 2) as usize;
        let n = (8 * (id.0 & 3)) as usize;
        unsafe {
            self.mpcore.icdiptr[m].modify(|mut icdiptr| *icdiptr.set_bits(n..=n+1, target_cpu + 1));
        }

        // sensitivity
        let m = (id.0 >> 4) as usize;
        let n = (2 * (id.0 & 0xF)) as usize;
        unsafe {
            self.mpcore.icdicfr[m].modify(|mut icdicfr| *icdicfr.set_bits(n..=n+1, match sensitivity {
                InterruptSensitivity::Level => 0b00,
                InterruptSensitivity::Edge  => 0b10,
            }));
        }

        // filter no interrupts (lowest priority)
        self.mpcore.iccpmr.write(mpcore::ICCPMR::zeroed().priority(0xFF));

        self.enable_interrupts();
    }
}
