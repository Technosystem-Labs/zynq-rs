use volatile_register::{RO, WO, RW};

use crate::{register, register_bit, register_bits, register_bits_typed, regs::*};

#[repr(C)]
pub struct RegisterBlock {
    pub net_ctrl: NetCtrl,
    pub net_cfg: NetCfg,
    pub net_status: NetStatus,
    pub unused0: RO<u32>,
    pub dma_cfg: RW<u32>,
    pub tx_status: TxStatus,
    pub rx_qbar: RxQbar,
    pub tx_qbar: TxQbar,
    pub rx_status: RxStatus,
    pub intr_status: RW<u32>,
    pub intr_en: WO<u32>,
    pub intr_dis: IntrDis,
    pub intr_mask: RW<u32>,
    pub phy_maint: PhyMaint,
    pub rx_pauseq: RO<u32>,
    pub tx_pauseq: RW<u32>,
    pub unused1: [RO<u32>; 16],
    pub hash_bot: RW<u32>,
    pub hash_top: RW<u32>,
    pub spec_addr1_bot: RW<u32>,
    pub spec_addr1_top: RW<u32>,
    pub spec_addr2_bot: RW<u32>,
    pub spec_addr2_top: RW<u32>,
    pub spec_addr3_bot: RW<u32>,
    pub spec_addr3_top: RW<u32>,
    pub spec_addr4_bot: RW<u32>,
    pub spec_addr4_top: RW<u32>,
    pub type_id_match1: RW<u32>,
    pub type_id_match2: RW<u32>,
    pub type_id_match3: RW<u32>,
    pub type_id_match4: RW<u32>,
    pub wake_on_lan: RW<u32>,
    pub ipg_stretch: RW<u32>,
    pub stacked_vlan: RW<u32>,
    pub tx_pfc_pause: RW<u32>,
    pub spec_addr1_mask_bot: RW<u32>,
    pub spec_addr1_mask_top: RW<u32>,
    pub unused2: [RO<u32>; 11],
    pub module_id: RO<u32>,
    pub octets_tx_bot: RO<u32>,
    pub octets_tx_top: RO<u32>,
    pub frames_tx: RO<u32>,
    pub broadcast_frames_tx: RO<u32>,
    pub multi_frames_tx: RO<u32>,
    pub pause_frames_tx: RO<u32>,
    pub frames_64b_tx: RO<u32>,
    pub frames_65to127b_tx: RO<u32>,
    pub frames_128to255b_tx: RO<u32>,
    pub frames_256to511b_tx: RO<u32>,
    pub frames_512to1023b_tx: RO<u32>,
    pub frames_1024to1518b_tx: RO<u32>,
    pub tx_under_runs: RO<u32>,
    pub unused3: RO<u32>,
    pub single_collisn_frames: RO<u32>,
    pub multi_collisn_frames: RO<u32>,
    pub excessive_collisns: RO<u32>,
    pub late_collisns: RO<u32>,
    pub deferred_tx_frames: RO<u32>,
    pub carrier_sense_errs: RO<u32>,
    pub octets_rx_bot: RO<u32>,
    pub octets_rx_top: RO<u32>,
    pub frames_rx: RO<u32>,
    pub bdcast_fames_rx: RO<u32>,
    pub multi_frames_rx: RO<u32>,
    pub pause_rx: RO<u32>,
    pub frames_64b_rx: RO<u32>,
    pub frames_65to127b_rx: RO<u32>,
    pub frames_128to255b_rx: RO<u32>,
    pub frames_256to511b_rx: RO<u32>,
    pub frames_512to1023b_rx: RO<u32>,
    pub frames_1024to1518b_rx: RO<u32>,
    pub unused4: RO<u32>,
    pub undersz_rx: RO<u32>,
    pub oversz_rx: RO<u32>,
    pub jab_rx: RO<u32>,
    pub fcs_errors: RO<u32>,
    pub length_field_errors: RO<u32>,
    pub rx_symbol_errors: RO<u32>,
    pub align_errors: RO<u32>,
    pub rx_resource_errors: RO<u32>,
    pub rx_overrun_errors: RO<u32>,
    pub ip_hdr_csum_errors: RO<u32>,
    pub tcp_csum_errors: RO<u32>,
    pub udp_csum_errors: RO<u32>,
    pub unused5: [RO<u32>; 5],
    pub timer_strobe_s: RW<u32>,
    pub timer_strobe_ns: RW<u32>,
    pub timer_s: RW<u32>,
    pub timer_ns: RW<u32>,
    pub timer_adjust: RW<u32>,
    pub timer_incr: RW<u32>,
    pub ptp_tx_s: RO<u32>,
    pub ptp_tx_ns: RO<u32>,
    pub ptp_rx_s: RO<u32>,
    pub ptp_rx_ns: RO<u32>,
    pub ptp_peer_tx_s: RO<u32>,
    pub ptp_peer_tx_ns: RO<u32>,
    pub ptp_peer_rx_s: RO<u32>,
    pub ptp_peer_rx_ns: RO<u32>,
    pub unused6: [RO<u32>; 33],
    pub design_cfg2: RO<u32>,
    pub design_cfg3: RO<u32>,
    pub design_cfg4: RO<u32>,
    pub design_cfg5: RO<u32>,
}

impl RegisterBlock {
    const GEM0: *mut Self = 0xE000B000 as *mut _;
    const GEM1: *mut Self = 0xE000C000 as *mut _;

    pub fn gem0() -> &'static mut Self {
        unsafe { &mut *Self::GEM0 }
    }

    pub fn gem1() -> &'static mut Self {
        unsafe { &mut *Self::GEM1 }
    }
}

register!(net_ctrl, NetCtrl, RW, u32);
register_bit!(net_ctrl, clear_stat_regs, 5);

register!(net_cfg, NetCfg, RW, u32);
/// false for 10Mbps, true for 100Mbps
register_bit!(net_cfg, speed, 0);
register_bit!(net_cfg, full_duplex, 1);
/// Discard non-VLAN frames
register_bit!(net_cfg, disc_non_vlan, 2);
/// Accept all valid frames?
register_bit!(net_cfg, copy_all, 4);
/// Don't accept broadcast destination address
register_bit!(net_cfg, no_broadcast, 5);
/// Multicast hash enable
register_bit!(net_cfg, multi_hash_en, 6);
/// Unicast hash enable
register_bit!(net_cfg, uni_hash_en, 7);
/// Accept frames up to 1536 bytes (instead of up to 1518 bytes)
register_bit!(net_cfg, rx_1536_byte_frames, 8);
/// External address match enable - when set the external address
/// match interface can be used to copy frames to memory.
register_bit!(net_cfg, ext_addr_match_en, 9);
/// Gigabit mode enable
register_bit!(net_cfg, gige_en, 10);
/// Enable TBI instead of GMII/MII interface?
register_bit!(net_cfg, pcs_sel, 11);
/// Retry test (reduces backoff between collisions to one slot)
register_bit!(net_cfg, retry_test, 12);
/// Pause frame enable
register_bit!(net_cfg, pause_en, 13);
/// Receive buffer offset
register_bits!(net_cfg, rx_buf_offset, u8, 14, 15);
/// Length field error frame discard
register_bit!(net_cfg, len_err_frame_disc, 16);
/// Write received frames to memory with Frame Check Sequence removed
register_bit!(net_cfg, fcs_remove, 17);
/// MDC clock divison
register_bits!(net_cfg, mdc_clk_div, u8, 18, 20);
/// Data bus width
register_bits!(net_cfg, dbus_width, u8, 21, 22);
/// Disable copy of pause frames
register_bit!(net_cfg, dis_cp_pause_frame, 23);
/// Receive checksum offload enable
register_bit!(net_cfg, rx_chksum_offld_en, 24);
/// Enable frames to be received in half-duplex mode while
/// transmitting
register_bit!(net_cfg, rx_hd_while_tx, 25);
/// Ignore Rx Framce Check Sequence (errors will not be rejected)
register_bit!(net_cfg, ignore_rx_fcs, 26);
/// SGMII mode enable
register_bit!(net_cfg, sgmii_en, 27);
/// IPG stretch enable
register_bit!(net_cfg, ipg_stretch_en, 28);
/// Receive bad preamble
register_bit!(net_cfg, rx_bad_preamble, 29);
/// Ignore IPG rx_er
register_bit!(net_cfg, ignore_ipg_rx_er, 30);
/// NA
register_bit!(net_cfg, unidir_en, 31);

register!(net_status, NetStatus, RW, u32);
register_bit!(net_status, pcs_link_state, 0);
register_bit!(net_status, mdio_in_pin_status, 1);
register_bit!(net_status, phy_mgmt_idle, 2);
register_bit!(net_status, pcs_autoneg_dup_res, 3);
register_bit!(net_status, pcs_autoneg_pause_rx_res, 4);
register_bit!(net_status, pcs_autoneg_pause_tx_res, 5);
register_bit!(net_status, pfc_pri_pause_neg, 6);

register!(tx_status, TxStatus, RW, u32);
register_bit!(tx_status, used_bit_read, 0);
register_bit!(tx_status, collision, 1);
register_bit!(tx_status, retry_limit_exceeded, 2);
register_bit!(tx_status, tx_go, 3);
register_bit!(tx_status, tx_corr_ahb_err, 4);
register_bit!(tx_status, tx_complete, 5);
register_bit!(tx_status, tx_under_run, 6);
register_bit!(tx_status, late_collision, 7);
register_bit!(tx_status, hresp_not_ok, 8);

register!(rx_status, RxStatus, RW, u32);
register_bit!(rx_status, buffer_not_avail, 0);
register_bit!(rx_status, frame_recd, 1);
register_bit!(rx_status, rx_overrun, 2);
register_bit!(rx_status, hresp_not_ok, 3);

register!(rx_qbar, RxQbar, RW, u32);
register_bits!(rx_qbar, rx_q_baseaddr, u32, 2, 31);

register!(tx_qbar, TxQbar, RW, u32);
register_bits!(tx_qbar, tx_q_baseaddr, u32, 2, 31);

register!(intr_dis, IntrDis, WO, u32);
register_bit!(intr_dis, mgmt_done, 0);
register_bit!(intr_dis, rx_complete, 1);
register_bit!(intr_dis, rx_used_read, 2);
register_bit!(intr_dis, tx_used_read, 3);
register_bit!(intr_dis, tx_underrun, 4);
register_bit!(intr_dis, retry_ex_late_collisn, 5);
register_bit!(intr_dis, tx_corrupt_ahb_err, 6);
register_bit!(intr_dis, tx_complete, 7);
register_bit!(intr_dis, link_chng, 9);
register_bit!(intr_dis, rx_overrun, 10);
register_bit!(intr_dis, hresp_not_ok, 11);
register_bit!(intr_dis, pause_nonzeroq, 12);
register_bit!(intr_dis, pause_zero, 13);
register_bit!(intr_dis, pause_tx, 14);
register_bit!(intr_dis, ex_intr, 15);
register_bit!(intr_dis, autoneg_complete, 16);
register_bit!(intr_dis, partner_pg_rx, 17);
register_bit!(intr_dis, delay_req_rx, 18);
register_bit!(intr_dis, sync_rx, 19);
register_bit!(intr_dis, delay_req_tx, 20);
register_bit!(intr_dis, sync_tx, 21);
register_bit!(intr_dis, pdelay_req_rx, 22);
register_bit!(intr_dis, pdelay_resp_rx, 23);
register_bit!(intr_dis, pdelay_req_tx, 24);
register_bit!(intr_dis, pdelay_resp_tx, 25);
register_bit!(intr_dis, tsu_sec_incr, 26);

#[repr(u8)]
pub enum PhyOperation {
    Write = 0b01,
    Read  = 0b10,
}

register!(phy_maint, PhyMaint, RW, u32);
/// Read from/write to the PHY
register_bits!(phy_maint, data, u16, 0, 15);
// Write `0b10`
register_bits!(phy_maint, must_10, u8, 16, 17);
/// Register address
register_bits!(phy_maint, reg_addr, u8, 18, 22);
/// PHY address
register_bits!(phy_maint, phy_addr, u8, 23, 27);
register_bits_typed!(phy_maint, operation, u8, PhyOperation, 28, 29);
// PHY clause 22 compliant (not clause 45)?
register_bit!(phy_maint, clause_22, 30);
