use core::fmt;

use libregister::*;
mod regs;

pub struct DevC {
    regs: &'static mut regs::RegisterBlock,
}

impl DevC {
    pub fn new() -> Self {
        DevC {
            regs: regs::RegisterBlock::devc(),
        }
    }

    pub fn enable(&mut self) {
        self.regs.control.modify(|_, w| {
            w.pcap_mode(true)
            .pcap_pr(true)
        })
    }
    pub fn disable(&mut self) {
        self.regs.control.modify(|_, w| {
            w.pcap_mode(false)
            .pcap_pr(false)
        })
    }
}
