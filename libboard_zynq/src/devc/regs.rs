use libregister::{
    register, register_at,
    register_bit, register_bits, register_bits_typed,
};
use volatile_register::WO;

#[repr(C)]
pub struct RegisterBlock {
    pub control: Control,
    pub lock: Lock,
    pub cfg: Cfg,
    pub int_sts: IntSts,
    pub int_mask: IntMask,
    pub status: Status,
    pub dma_src_addr: DmaSrcAddr,
    pub dma_dest_addr: DmaDestAddr,
    pub dma_src_len: DmaSrcLen,
    pub dma_dest_len: DmaDestLen,
    unused0: u32,
    pub multiboot_addr: MultibootAddr,
    unused1: u32,
    pub unlock: WO<u32>,
    unused2: [u32; 18],
    pub mctrl: MCtrl,
    unused3: [u32; 31],
    pub xadcif_cfg: XADCIfCfg,
    pub xadcif_int_sts: XADCIfIntSts,
    pub xadcif_int_mask: XADCIfIntMask,
    pub xadcif_msts: XADCIf_Msts,
    pub xadcif_cmdfifo: XADCIf_CmdFIFO,
    pub xadcif_rdfifo: XADCIf_RdFIFO,
    pub xadcif_mctl: XADCIf_MCtl,
}
register_at!(RegisterBlock, 0xF8007000, devc);
register!(control, Control, RW, u32);
register_bit!(control,  force_rst, 31);
register_bit!(control,  pcfg_prog_b, 30);
register_bit!(control,  pcfg_pro_cnt_4k, 29);
register_bit!(control,  pcap_pr, 27);
register_bit!(control,  pcap_mode, 26);
register_bit!(control,  pcap_rate_en, 25);
register_bit!(control,  multiboot_en, 24);
register_bit!(control,  jtag_chain_dis, 23);
register_bit!(control,  pcfg_aes_fuse, 12);
register_bits!(control, pcfg_aes_en, u8, 9, 11);
register_bit!(control,  seu_en, 8);
register_bit!(control,  sec_en, 7);
register_bit!(control,  spniden, 6);
register_bit!(control,  spiden, 5);
register_bit!(control,  niden, 4);
register_bit!(control,  dbgen, 3);
register_bits!(control, dap_en, u8, 0, 2);

register!(lock, Lock, RW, u32);
register_bit!(lock, aes_fuse_lock, 4);
register_bit!(lock, aes_en, 3);
register_bit!(lock, seu, 2);
register_bit!(lock, sec, 1);
register_bit!(lock, dbg, 0);

register!(cfg, Cfg, RW, u32);
#[allow(unused)]
#[repr(u8)]
pub enum RFifoTh {
    OneFourthFull    = 0b00, // One fourth full for read
    HalfFull         = 0b01, // Half full for read
    ThreeFourthFull  = 0b10, // Three fourth full for read
    Full             = 0b11, // Full for read
}
register_bits_typed!(cfg, rfifo_th, u8, RFifoTh, 10, 11);
#[allow(unused)]
#[repr(u8)]
pub enum WFifoTh {
    OneFourthEmpty   = 0b00, // One fourth empty for write
    HalfEmpty        = 0b01, // Half empty for write
    ThreeFourthEmpty = 0b10, // Three fourth empty for write
    Empty            = 0b11, // Empty for write
}
register_bits_typed!(cfg, wfifo_th, u8, WFifoTh, 8, 9);
register_bit!(cfg, rclk_edge, 7);
register_bit!(cfg, wclk_edge, 6);
register_bit!(cfg, disable_src_inc, 5);
register_bit!(cfg, disable_dst_inc, 4);

register!(int_sts, IntSts, RW, u32);
register_bit!(int_sts, pss_gts_usr_b_int, 31);
register_bit!(int_sts, pss_fst_cfg_b_int, 30);
register_bit!(int_sts, pss_gpwrdwn_b_int, 29);
register_bit!(int_sts, pss_gts_cfg_b_int, 28);
register_bit!(int_sts, pss_cfg_reset_b_int, 27);
register_bit!(int_sts, ixr_axi_wto, 23);
register_bit!(int_sts, ixr_axi_werr, 22);
register_bit!(int_sts, ixr_axi_rto, 21);
register_bit!(int_sts, ixr_axi_rerr, 20);
register_bit!(int_sts, ixr_rx_fifo_ov, 18);
register_bit!(int_sts, ixr_wr_fifo_lvl, 17);
register_bit!(int_sts, ixr_rd_fifo_lvl, 16);
register_bit!(int_sts, ixr_dma_cmd_err, 15);
register_bit!(int_sts, ixr_dma_q_ov, 14);
register_bit!(int_sts, ixr_dma_done, 13);
register_bit!(int_sts, ixr_d_p_done, 12);
register_bit!(int_sts, ixr_p2d_len_err, 11);
register_bit!(int_sts, ixr_pcfg_hmac_err, 6);
register_bit!(int_sts, ixr_pcfg_seu_err, 5);
register_bit!(int_sts, ixr_pcfg_por_b, 4);
register_bit!(int_sts, ixr_pcfg_cfg_rst, 3);
register_bit!(int_sts, ixr_pcfg_done, 2);
register_bit!(int_sts, ixr_pcfg_init_pe, 1);
register_bit!(int_sts, ixr_pcfg_init_ne, 0);

register!(int_mask, IntMask, RW, u32);
register_bit!(int_mask, m_pss_gts_usr_b_int, 31);
register_bit!(int_mask, m_pss_fst_cfg_b_int, 30);
register_bit!(int_mask, m_pss_gpwrdwn_b_int, 29);
register_bit!(int_mask, m_pss_gts_cfg_b_int, 28);
register_bit!(int_mask, m_pss_cfg_reset_b_int, 27);
register_bit!(int_mask, ixr_axi_wto, 23);
register_bit!(int_mask, ixr_axi_werr, 22);
register_bit!(int_mask, ixr_axi_rto, 21);
register_bit!(int_mask, ixr_axi_rerr, 20);
register_bit!(int_mask, ixr_rx_fifo_ov, 18);
register_bit!(int_mask, ixr_wr_fifo_lvl, 17);
register_bit!(int_mask, ixr_rd_fifo_lvl, 16);
register_bit!(int_mask, ixr_dma_cmd_err, 15);
register_bit!(int_mask, ixr_dma_q_ov, 14);
register_bit!(int_mask, ixr_dma_done, 13);
register_bit!(int_mask, ixr_d_p_done, 12);
register_bit!(int_mask, ixr_p2d_len_err, 11);
register_bit!(int_mask, ixr_pcfg_hmac_err, 6);
register_bit!(int_mask, ixr_pcfg_seu_err, 5);
register_bit!(int_mask, ixr_pcfg_por_b, 4);
register_bit!(int_mask, ixr_pcfg_cfg_rst, 3);
register_bit!(int_mask, ixr_pcfg_done, 2);
register_bit!(int_mask, ixr_pcfg_init_pe, 1);
register_bit!(int_mask, ixr_pcfg_init_ne, 0);

register!(status, Status, RO, u32);
register_bit!(status, dma_cmd_q_f, 31);
register_bit!(status, dma_cmd_q_e, 30);
register_bits!(status, dma_done_cnt, u8, 28, 29);
register_bits!(status, rx_fifo_lvl, u8, 20, 24);
register_bits!(status, tx_fifo_lvl, u8, 12, 18);
register_bit!(status, pss_gts_usr_b, 11);
register_bit!(status, pss_fst_cfg_b, 10);
register_bit!(status, pss_gpwrdwn_b, 9);
register_bit!(status, pss_gts_cfg_b, 8);
register_bit!(status, secure_rst, 7);
register_bit!(status, illegal_apb_access , 6);
register_bit!(status, pss_cfg_reset_b, 5);
register_bit!(status, pcfg_init, 4);
register_bit!(status, efuse_sw_reserve, 3);
register_bit!(status, efuse_sec_en, 2);
register_bit!(status, efuse_jtag_dis, 1);

register!(dma_src_addr, DmaSrcAddr, RW, u32);
register_bits!(dma_src_addr, src_addr, u32, 0, 31);

register!(dma_dest_addr, DmaDestAddr, RW, u32);
register_bits!(dma_dest_addr, dest_addr, u32, 0, 31);

register!(dma_src_len, DmaSrcLen, RW, u32);
register_bits!(dma_src_len, dma_len, u32, 0, 26);

register!(dma_dest_len, DmaDestLen, RW, u32);
register_bits!(dma_dest_len, dma_len, u32, 0, 26);

register!(multiboot_addr, MultibootAddr, RW, u32);
register_bits!(multiboot_addr, multiboot_addr, u8, 0, 12);

register!(mctrl, MCtrl, RW, u32);
register_bits!(mctrl, ps_version, u8, 28, 31);
register_bit!(mctrl, pcfg_por_b, 8);
register_bit!(mctrl, pcap_lpbk, 4);

register!(xadcif_cfg, XADCIfCfg, RW, u32);
register_bit!(xadcif_cfg, enable, 31);
register_bits!(xadcif_cfg, cfifoth, u8, 20, 23);
register_bits!(xadcif_cfg, dfifoth, u8, 16, 19);
register_bit!(xadcif_cfg, wedge, 13);
register_bit!(xadcif_cfg, redge, 13);
register_bits!(xadcif_cfg, tckrate, u8, 8, 9);
register_bits!(xadcif_cfg, igap, u8, 0, 4);

register!(xadcif_int_sts, XADCIfIntSts, RW, u32);
register_bit!(xadcif_int_sts, cfifo_lth, 9);
register_bit!(xadcif_int_sts, dfifo_gth, 8);
register_bit!(xadcif_int_sts, ot, 7);
register_bits!(xadcif_int_sts, alm, u8, 0, 6);

register!(xadcif_int_mask, XADCIfIntMask, RW, u32);
register_bit!(xadcif_int_mask, m_cfifo_lth, 9);
register_bit!(xadcif_int_mask, m_dfifo_gth, 8);
register_bit!(xadcif_int_mask, m_ot, 7);
register_bits!(xadcif_int_mask, m_alm, u8, 0, 6);

register!(xadcif_msts, XADCIf_Msts, RO, u32);
register_bits!(xadcif_msts, cfifo_lvl, u8, 16, 19);
register_bits!(xadcif_msts, dfifo_lvl, u8, 12, 15);
register_bit!(xadcif_msts, cfifof, 11);
register_bit!(xadcif_msts, cfifoe, 10);
register_bit!(xadcif_msts, dfifof, 9);
register_bit!(xadcif_msts, dfifoe, 8);
register_bit!(xadcif_msts, ot, 7);
register_bits!(xadcif_msts, alm, u8, 0, 6);

register!(xadcif_cmdfifo, XADCIf_CmdFIFO, WO, u32);
register_bits!(xadcif_cmdfifo, cmd, u8, 0, 31);

register!(xadcif_rdfifo, XADCIf_RdFIFO, RO, u32);
register_bits!(xadcif_rdfifo, rddata, u8, 0, 31);

register!(xadcif_mctl, XADCIf_MCtl, RW, u32);
register_bit!(xadcif_mctl, reset, 4);
