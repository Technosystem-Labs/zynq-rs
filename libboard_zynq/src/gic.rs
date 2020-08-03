//! ARM Generic Interrupt Controller

use bit_field::BitField;
use libregister::{RegisterW, RegisterRW, RegisterR};
use super::mpcore;

#[derive(Debug, Clone, Copy)]
pub struct InterruptId(pub u8);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CPUCore {
    Core0 = 0b01,
    Core1 = 0b10
}

#[derive(Debug, Clone, Copy)]
pub struct TargetCPU(u8);

impl TargetCPU {
    pub const fn none() -> TargetCPU {
        TargetCPU(0)
    }

    pub const fn and(self, other: TargetCPU) -> TargetCPU {
        TargetCPU(self.0 | other.0)
    }
}

impl From<CPUCore> for TargetCPU {
    fn from(core: CPUCore) -> Self {
        TargetCPU(core as u8)
    }
}

pub enum TargetList {
    CPUList(TargetCPU),
    Others,
    This
}

impl From<CPUCore> for TargetList {
    fn from(core: CPUCore) -> Self {
        TargetList::CPUList(TargetCPU(core as u8))
    }
}

impl From<TargetCPU> for TargetList {
    fn from(cpu: TargetCPU) -> Self {
        TargetList::CPUList(cpu)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InterruptSensitivity {
    Level,
    Edge,
}

pub struct InterruptController {
    mpcore: &'static mut mpcore::RegisterBlock,
}

impl InterruptController {
    pub fn new(mpcore: &'static mut mpcore::RegisterBlock) -> Self {
        InterruptController { mpcore }
    }

    pub fn disable_interrupts(&mut self) {
        self.mpcore.iccicr.modify(|_, w| w.enable_ns(false)
                                          .enable_s(false));
        // FIXME: Should we disable the distributor globally when we disable interrupt (for a single
        // core)?
        // self.mpcore.icddcr.modify(|_, w| w.enable_secure(false)
        //                                   .enable_non_secure(false));
    }

    /// enable interrupt signaling
    pub fn enable_interrupts(&mut self) {
        self.mpcore.iccicr.modify(|_, w| w.enable_ns(true)
                                          .enable_s(true));
        self.mpcore.icddcr.modify(|_, w| w.enable_secure(true));

        // Enable all interrupts except those of the lowest priority.
        self.mpcore.iccpmr.write(mpcore::ICCPMR::zeroed().priority(0xFF));
    }

    /// send software generated interrupt
    pub fn send_sgi(&mut self, id: InterruptId, targets: TargetList) {
        assert!(id.0 < 16);
        self.mpcore.icdsgir.modify(|_, w| match targets {
            TargetList::CPUList(list) => w.target_list_filter(0).cpu_target_list(list.0),
            TargetList::Others => w.target_list_filter(0b01),
            TargetList::This => w.target_list_filter(0b10)
        }.sgiintid(id.0).satt(false));
    }

    /// enable the interrupt *for this core*.
    /// Not needed for SGI.
    pub fn enable(&mut self, id: InterruptId, target_cpu: CPUCore, sensitivity: InterruptSensitivity, priority: u8) {
        // only 5 bits of the priority is useful
        assert!(priority < 32);

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
            self.mpcore.icdiptr[m].modify(|mut icdiptr| *icdiptr.set_bits(n..=n+1, target_cpu as u32 + 1));
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

        // priority
        let offset = (id.0 % 4) * 8;
        let priority: u32 = (priority as u32) << (offset + 3);
        let mask: u32 = 0xFFFFFFFF ^ (0xFF << offset);
        unsafe {
            self.mpcore.icdipr[id.0 as usize / 4].modify(|v| (v & mask) | priority);
        }

        self.enable_interrupts();
    }

    pub fn end_interrupt(&mut self, id: InterruptId) {
        self.mpcore.icceoir.modify(|_, w| w.eoiintid(id.0 as u32));
    }

    pub fn get_interrupt_id(&self) -> InterruptId {
        InterruptId(self.mpcore.icciar.read().ackintid() as u8)
    }

}
