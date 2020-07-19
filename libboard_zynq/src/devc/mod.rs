use super::time::Milliseconds;
use crate::slcr;
use embedded_hal::timer::CountDown;
use libcortex_a9::cache;
use libregister::*;
use log::{debug, trace};

mod regs;

pub struct DevC {
    regs: &'static mut regs::RegisterBlock,
    enabled: bool,
    count_down: super::timer::global::CountDown,
    timeout_ms: Milliseconds,
}

/// DMA transfer type for PCAP
/// All insecure, we do not implement encrypted transfer
#[derive(PartialEq, Clone, Copy)]
pub enum TransferType {
    PcapWrite,
    PcapReadback,
    ConcurrentReadWrite,
}

pub enum TransferTarget<'a> {
    /// From/To PL, with length in bytes.
    PL(u32),
    /// Source target, immutable.
    SliceSrc(&'a [u8]),
    /// Last source target, immutable.
    SliceSrcLast(&'a [u8]),
    /// Destination target, mutable.
    SliceDest(&'a mut [u8]),
    /// Last destination target, mutable.
    SliceDestLast(&'a mut [u8]),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DevcError {
    NotInitialized,
    ResetTimeout,
    DmaBusy,
    DmaTimeout,
    DoneTimeout,
    Unknown(u32),
}

impl core::fmt::Display for DevcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use DevcError::*;
        match self {
            NotInitialized => write!(f, "DevC driver not initialized properly."),
            ResetTimeout => write!(f, "DevC driver reset timeout."),
            DmaBusy => write!(f, "DevC driver DMA busy."),
            DmaTimeout => write!(f, "DevC driver DMA timeout."),
            DoneTimeout => write!(
                f,
                "FPGA DONE signal timeout. Check if the bitstream is correct."
            ),
            Unknown(reg) => write!(f, "Unknown error, interrupt status register = 0x{:0X}", reg),
        }
    }
}

impl DevC {
    /// Create a new DevC peripheral handle with default timeout = 500ms.
    pub fn new() -> Self {
        Self::new_timeout(Milliseconds(500))
    }

    /// Create a new DevC peripheral handle.
    /// `timeout_ms`: timeout for operations like initialize and DMA transfer.
    pub fn new_timeout(timeout_ms: Milliseconds) -> Self {
        DevC {
            regs: regs::RegisterBlock::devc(),
            enabled: false,
            count_down: unsafe { super::timer::GlobalTimer::get() }.countdown(),
            timeout_ms,
        }
    }

    /// Enable the devc driver, must be called before `program` or
    /// `start_dma_transaction`.
    pub fn enable(&mut self) {
        const UNLOCK_PATTERN: u32 = 0x757BDF0D;
        unsafe {
            // unlock register with magic pattern
            self.regs.unlock.write(UNLOCK_PATTERN);
        }
        self.regs
            .control
            .modify(|_, w| w.pcap_mode(true).pcap_pr(true));
        self.regs
            .int_mask
            .write(self::regs::int_mask::Write { inner: 0xFFFFFFFF });
        self.clear_interrupts();
        self.enabled = true;
    }

    /// Disable the devc driver.
    /// `enable` has to be called before further `program` or
    /// `start_dma_transaction`.
    pub fn disable(&mut self) {
        self.regs
            .control
            .modify(|_, w| w.pcap_mode(false).pcap_pr(false));
        self.enabled = false;
    }

    /// Check if the FPGA programming is done.
    pub fn is_done(&self) -> bool {
        // Note: contrary to what the TRM says, this appears to be simply the
        // state of the DONE signal.
        self.regs.int_sts.read().ixr_pcfg_done()
    }

    /// Wait on a certain condition with hardcoded timeout.
    fn wait_condition<F: Fn(&mut Self) -> bool>(
        &mut self,
        fun: F,
        err: DevcError,
    ) -> Result<(), DevcError> {
        self.count_down.start(self.timeout_ms);
        while let Err(nb::Error::WouldBlock) = self.count_down.wait() {
            if fun(self) {
                return Ok(());
            } else if self.has_error() {
                return Err(DevcError::Unknown(self.regs.int_sts.read().inner));
            }
        }
        Err(err)
    }

    /// Program the FPGA.
    /// Note that the user should make sure that the bitstream loaded is
    /// correct.
    pub fn program(&mut self, src: &[u8]) -> Result<(), DevcError> {
        if !self.enabled {
            panic!("Attempting to use devc when it is not enabled");
        }
        self.clear_interrupts();

        debug!("Invalidate DCache for bitstream buffer");
        cache::dcci_slice(src);

        debug!("Init preload FPGA");
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.init_preload_fpga();
        });
        debug!("Toggling PROG_B");
        // set PCFG_PROG_B to high low high
        self.regs.control.modify(|_, w| w.pcfg_prog_b(true));
        self.regs.control.modify(|_, w| w.pcfg_prog_b(false));

        // wait until init is false
        self.wait_condition(
            |s| !s.regs.status.read().pcfg_init(),
            DevcError::ResetTimeout,
        )?;

        self.regs.control.modify(|_, w| w.pcfg_prog_b(true));
        // wait until init is true
        self.wait_condition(
            |s| s.regs.status.read().pcfg_init(),
            DevcError::ResetTimeout,
        )?;

        self.regs.int_sts.write(
            self::regs::IntSts::zeroed()
                .pss_cfg_reset_b_int(true)
                .ixr_pcfg_cfg_rst(true),
        );

        self.dma_transfer(
            TransferTarget::SliceSrcLast(src),
            TransferTarget::PL(src.len() as u32),
            TransferType::PcapWrite,
        )?;

        debug!("Waiting for done");
        self.wait_condition(|s| s.is_done(), DevcError::DoneTimeout)?;

        debug!("Init postload FPGA");
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.init_postload_fpga();
        });

        Ok(())
    }

    /// Initiate DMA transaction
    /// This function only sets the src and dest registers, and should not be used directly.
    fn initiate_dma(&mut self, src: TransferTarget, dest: TransferTarget) {
        use TransferTarget::*;
        const INVALID_ADDR: u32 = 0xFFFFFFFF;

        if let (PL(_), PL(_)) = (&src, &dest) {
            panic!("Only one of src/dest can be PL");
        }

        let (src_addr, src_len): (u32, u32) = match src {
            PL(l) => (INVALID_ADDR, l / 4),
            SliceSrc(s) => (s.as_ptr() as u32, s.len() as u32 / 4),
            SliceDest(s) => (s.as_ptr() as u32, s.len() as u32 / 4),
            SliceSrcLast(s) => ((s.as_ptr() as u32) | 0x01, s.len() as u32 / 4),
            SliceDestLast(s) => ((s.as_ptr() as u32) | 0x01, s.len() as u32 / 4),
        };

        let (dest_addr, dest_len): (u32, u32) = match dest {
            PL(l) => (INVALID_ADDR, l / 4),
            SliceDest(s) => (s.as_ptr() as u32, s.len() as u32 / 4),
            SliceDestLast(s) => ((s.as_ptr() as u32) | 0x01, s.len() as u32 / 4),
            SliceSrc(_) | SliceSrcLast(_) => {
                panic!("Destination cannot be SliceSrc/SliceSrcLast, it must be mutable.")
            }
        };

        self.regs.dma_src_addr.modify(|_, w| w.src_addr(src_addr));
        self.regs
            .dma_dest_addr
            .modify(|_, w| w.dest_addr(dest_addr));
        self.regs.dma_src_len.modify(|_, w| w.dma_len(src_len));
        self.regs.dma_dest_len.modify(|_, w| w.dma_len(dest_len));
    }

    /// Blocking DMA transfer
    /// ## Note
    /// This is blocking because there seems to be no other way to guarantee
    /// safety, and I don't think requiring static is a solution here due to the
    /// large buffer size.
    /// See https://docs.rust-embedded.org/embedonomicon/dma.html for details.
    ///
    /// The following checks are implemented in runtime (panic).
    /// * Dest would *NOT* accept src type, as the slices are immutable.
    /// * At most one of src and dest can be PL type.
    pub fn dma_transfer(
        &mut self,
        src: TransferTarget,
        dest: TransferTarget,
        transfer_type: TransferType,
    ) -> Result<(), DevcError> {
        if !self.enabled {
            panic!("Attempting to use devc when it is not enabled");
        }
        if self.regs.status.read().dma_cmd_q_f() {
            return Err(DevcError::DmaBusy);
        }

        if transfer_type != TransferType::ConcurrentReadWrite
            && !self.regs.status.read().pcfg_init()
        {
            return Err(DevcError::NotInitialized);
        }
        match &transfer_type {
            TransferType::PcapReadback => {
                // clear internal PCAP loopback
                self.regs.mctrl.modify(|_, w| w.pcap_lpbk(false));
                // send READ frame command
                self.initiate_dma(src, TransferTarget::PL(0));
                // wait until DMA done
                self.wait_dma_transfer_complete()?;
                // initiate the DMA write
                self.initiate_dma(TransferTarget::PL(0), dest);
            }
            TransferType::PcapWrite | TransferType::ConcurrentReadWrite => {
                self.regs
                    .mctrl
                    .modify(|_, w| w.pcap_lpbk(transfer_type == TransferType::ConcurrentReadWrite));
                // PCAP data transmitted every clock
                self.regs.control.modify(|_, w| w.pcap_rate_en(false));

                self.initiate_dma(src, dest);
            }
        }
        self.wait_dma_transfer_complete()?;
        Ok(())
    }

    fn wait_dma_transfer_complete(&mut self) -> Result<(), DevcError> {
        trace!("Wait for DMA done");
        self.wait_condition(
            |s| s.regs.int_sts.read().ixr_dma_done(),
            DevcError::DmaTimeout,
        )?;
        self.regs
            .int_sts
            .write(self::regs::IntSts::zeroed().ixr_dma_done(true));
        Ok(())
    }

    /// Dump useful registers for devc block.
    pub fn dump_registers(&self) {
        debug!("Mctrl: 0x{:0X}", self.regs.mctrl.read().inner);
        debug!("Control: 0x{:0X}", self.regs.control.read().inner);
        debug!("Status: 0x{:0X}", self.regs.status.read().inner);
        debug!("INT STS: 0x{:0X}", self.regs.int_sts.read().inner);
    }

    /// Clear interrupt status for devc.
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

    fn has_error(&self) -> bool {
        let status = self.regs.int_sts.read();
        status.ixr_axi_wto()
            || status.ixr_axi_werr()
            || status.ixr_axi_rto()
            || status.ixr_axi_rerr()
            || status.ixr_rx_fifo_ov()
            || status.ixr_dma_cmd_err()
            || status.ixr_dma_q_ov()
            || status.ixr_p2d_len_err()
    }
}
