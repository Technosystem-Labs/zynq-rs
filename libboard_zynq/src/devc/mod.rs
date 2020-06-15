use crate::slcr;
use libregister::*;
use log::debug;
mod regs;

pub struct DevC {
    regs: &'static mut regs::RegisterBlock,
}

/// DMA transfer type for PCAP
/// All insecure, we do not implement encrypted transfer
#[derive(PartialEq)]
pub enum TransferType {
    PcapWrite,
    PcapReadback,
    ConcurrentReadWrite,
}

pub const INVALID_ADDR: u32 = 0xFFFFFFFF;

impl DevC {
    pub fn new() -> Self {
        DevC {
            regs: regs::RegisterBlock::devc(),
        }
    }

    pub fn enable(&mut self) {
        unsafe {
            // unlock register with magic pattern
            self.regs.unlock.write(0x757BDF0D);
        }
        self.regs
            .control
            .modify(|_, w| w.pcap_mode(true).pcap_pr(true));
        self.regs
            .int_mask
            .write(self::regs::int_mask::Write { inner: 0xFFFFFFFF });
        self.clear_interrupts();
    }
    pub fn disable(&mut self) {
        self.regs
            .control
            .modify(|_, w| w.pcap_mode(false).pcap_pr(false))
    }

    pub fn is_done(&self) -> bool {
        // Note: contrary to what the TRM says, this appears to be simply
        // the state of the DONE signal.
        self.regs.int_sts.read().ixr_pcfg_done()
    }

    pub fn program(&mut self, src_addr: u32, len: u32) {
        debug!("Init preload FPGA");
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.init_preload_fpga();
        });
        debug!("Toggling PROG_B");
        // set PCFG_PROG_B to high low high
        self.regs.control.modify(|_, w| w.pcfg_prog_b(true));
        self.regs.control.modify(|_, w| w.pcfg_prog_b(false));
        while self.regs.status.read().pcfg_init() {}
        self.regs.control.modify(|_, w| w.pcfg_prog_b(true));
        while !self.regs.status.read().pcfg_init() {}
        self.regs.int_sts.write(
            self::regs::IntSts::zeroed()
                .pss_cfg_reset_b_int(true)
                .ixr_pcfg_cfg_rst(true),
        );

        debug!("ADDR: {:0X}", src_addr);
        self.dma_transfer(
            src_addr | 0x1,
            INVALID_ADDR | 0x1,
            len,
            len,
            TransferType::PcapWrite,
        );

        self.wait_dma_transfer_complete();
        debug!("INT_STS: {:0X}", self.regs.int_sts.read().inner);
        debug!("STATUS: {:0X}", self.regs.status.read().inner);

        debug!("Waiting for done");
        while !self.is_done() {}

        debug!("INT_STS: {:0X}", self.regs.int_sts.read().inner);

        debug!("Init postload FPGA");
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.init_postload_fpga();
        });
    }

    /// Initiate DMA transaction, all lengths are in words.
    /// > This function is not meant to be used directly.
    fn initiate_dma(&mut self, src_addr: u32, dest_addr: u32, src_len: u32, dest_len: u32) {
        self.regs.dma_src_addr.modify(|_, w| w.src_addr(src_addr));
        self.regs
            .dma_dest_addr
            .modify(|_, w| w.dest_addr(dest_addr));
        self.regs.dma_src_len.modify(|_, w| w.dma_len(src_len));
        self.regs.dma_dest_len.modify(|_, w| w.dma_len(dest_len));
    }

    /// DMA transfer, all lengths are in words.
    pub fn dma_transfer(
        &mut self,
        src_addr: u32,
        dest_addr: u32,
        src_len: u32,
        dest_len: u32,
        transfer_type: TransferType,
    ) -> Result<(), &str> {
        if self.regs.status.read().dma_cmd_q_f() {
            return Err("DMA busy");
        }

        if transfer_type != TransferType::ConcurrentReadWrite
            && !self.regs.status.read().pcfg_init()
        {
            return Err("Fabric not initialized");
        }
        match &transfer_type {
            TransferType::PcapReadback => {
                // clear internal PCAP loopback
                self.regs.mctrl.modify(|_, w| w.pcap_lpbk(false));
                // send READ frame command
                self.initiate_dma(src_addr, INVALID_ADDR, src_len, 0);
                // wait until DMA done
                while !self.regs.int_sts.read().ixr_d_p_done() {}
                // initiate the DMA write
                self.initiate_dma(INVALID_ADDR, dest_addr, 0, dest_len);
            }
            TransferType::PcapWrite | TransferType::ConcurrentReadWrite => {
                self.regs
                    .mctrl
                    .modify(|_, w| w.pcap_lpbk(transfer_type == TransferType::ConcurrentReadWrite));
                // PCAP data transmitted every clock
                self.regs.control.modify(|_, w| w.pcap_rate_en(false));

                self.initiate_dma(src_addr, dest_addr, src_len, dest_len);
            }
        }
        Ok(())
    }

    pub fn wait_dma_transfer_complete(&mut self) {
        debug!("Wait for DMA done");
        while !self.regs.int_sts.read().ixr_dma_done() {}
        self.regs
            .int_sts
            .write(self::regs::IntSts::zeroed().ixr_dma_done(true));
    }

    pub fn dump_registers(&self) {
        debug!("Control: 0x{:0X}", self.regs.control.read().inner);
        debug!("Status: 0x{:0X}", self.regs.status.read().inner);
        debug!("INT STS: 0x{:0X}", self.regs.int_sts.read().inner);
    }

    pub fn clear_interrupts(&mut self) {
        self.regs.int_sts.modify(|_, w| {
            w.pss_gts_usr_b_int(true)
                .pss_fst_cfg_b_int(true)
                .pss_gpwrdwn_b_int(true)
                .pss_gts_cfg_b_int(true)
                .pss_cfg_reset_b_int(true)
                .ixr_axi_wto(true)
                .ixr_axi_werr(true)
                .ixr_axi_rto(true)
                .ixr_axi_rerr(true)
                .ixr_rx_fifo_ov(true)
                .ixr_wr_fifo_lvl(true)
                .ixr_rd_fifo_lvl(true)
                .ixr_dma_cmd_err(true)
                .ixr_dma_q_ov(true)
                .ixr_dma_done(true)
                .ixr_d_p_done(true)
                .ixr_p2d_len_err(true)
                .ixr_pcfg_hmac_err(true)
                .ixr_pcfg_seu_err(true)
                .ixr_pcfg_por_b(true)
                .ixr_pcfg_cfg_rst(true)
                .ixr_pcfg_done(true)
                .ixr_pcfg_init_pe(true)
                .ixr_pcfg_init_ne(true)
        })
    }
}
