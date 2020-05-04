use libregister::*;
use crate::slcr;
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

    pub fn is_done(&self) -> bool {
        // Note: contrary to what the TRM says, this appears to be simply
        // the state of the DONE signal.
        self.regs.int_sts.read().ixr_pcfg_done()
    }

    pub fn program(&mut self) {
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.init_preload_fpga();
        });

        while !self.is_done() {}

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.init_postload_fpga();
        });
    }
}
