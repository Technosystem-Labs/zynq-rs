use volatile_register::{RO, RW};

use libregister::{register, register_bit, register_bits, register_bits_typed};

#[allow(unused)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DataBusWidth {
    Width32bit = 0b00,
    Width16bit = 0b01,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum ControllerStatus {
    Init = 0,
    Normal = 1,
    Powerdown = 2,
    SelfRefresh = 3,
    Powerdown1 = 4,
    Powerdown2 = 5,
    Powerdown3 = 6,
    Powerdown4 = 7,
}



#[repr(C)]
pub struct RegisterBlock {
    pub ddrc_ctrl: DdrcCtrl,
    pub two_rank_cfg: RW<u32>,
    pub hpr: RW<u32>,
    pub lpr: RW<u32>,
    pub wr: RW<u32>,
    pub dram_param0: DramParam0,
    pub dram_param1: RW<u32>,
    pub dram_param2: DramParam2,
    pub dram_param3: RW<u32>,
    pub dram_param4: RW<u32>,
    pub dram_init_param: RW<u32>,
    pub dram_emr: RW<u32>,
    pub dram_emr_mr: DramEmrMr,
    pub dram_burst8_rdwr: RW<u32>,
    pub dram_disable_dq: RW<u32>,
    pub dram_addr_map_bank: RW<u32>,
    pub dram_addr_map_col: RW<u32>,
    pub dram_addr_map_row: RW<u32>,
    pub dram_odt: RW<u32>,
    pub phy_dbg: RW<u32>,
    pub phy_cmd_timeout_rddata_cpt: PhyCmdTimeoutRddataCpt,
    pub mode_sts: ModeStsReg,
    pub dll_calib: RW<u32>,
    pub odt_delay_hold: RW<u32>,
    pub ctrl1: RW<u32>,
    pub ctrl2: RW<u32>,
    pub ctrl3: RW<u32>,
    pub ctrl4: RW<u32>,
    _unused0: [RO<u32>; 2],
    pub ctrl5: RW<u32>,
    pub ctrl6: RW<u32>,
    _unused1: [RO<u32>; 8],
    pub che_refresh_timer01: RW<u32>,
    pub che_t_zq: RW<u32>,
    pub che_t_zq_short_interval: RW<u32>,
    pub deep_pwrdwn: RW<u32>,
    pub reg_2c: Reg2C,
    pub reg_2d: RW<u32>,
    pub dfi_timing: DfiTiming,
    _unused2: [RO<u32>; 2],
    pub che_ecc_control_offset: RW<u32>,
    pub che_corr_ecc_log_offset: RW<u32>,
    pub che_corr_ecc_addr_offset: RW<u32>,
    pub che_corr_ecc_data_31_0_offset: RW<u32>,
    pub che_corr_ecc_data_63_32_offset: RW<u32>,
    pub che_corr_ecc_data_71_64_offset: RW<u32>,
    pub che_uncorr_ecc_log_offset: RW<u32>,
    pub che_uncorr_ecc_addr_offset: RW<u32>,
    pub che_uncorr_ecc_data_31_0_offset: RW<u32>,
    pub che_uncorr_ecc_data_63_32_offset: RW<u32>,
    pub che_uncorr_ecc_data_71_64_offset: RW<u32>,
    pub che_ecc_stats_offset: RW<u32>,
    pub ecc_scrub: RW<u32>,
    pub che_ecc_corr_bit_mask_31_0_offset: RW<u32>,
    pub che_ecc_corr_bit_mask_63_32_offset: RW<u32>,
    _unused3: [RO<u32>; 5],
    pub phy_rcvr_enable: RW<u32>,
    pub phy_config0: RW<u32>,
    pub phy_config1: RW<u32>,
    pub phy_config2: RW<u32>,
    pub phy_config3: RW<u32>,
    _unused4: RO<u32>,
    pub phy_init_ratio0: PhyInitRatio,
    pub phy_init_ratio1: PhyInitRatio,
    pub phy_init_ratio2: PhyInitRatio,
    pub phy_init_ratio3: PhyInitRatio,
    _unused5: RO<u32>,
    pub phy_rd_dqs_cfg0: RW<u32>,
    pub phy_rd_dqs_cfg1: RW<u32>,
    pub phy_rd_dqs_cfg2: RW<u32>,
    pub phy_rd_dqs_cfg3: RW<u32>,
    _unused6: RO<u32>,
    pub phy_wr_dqs_cfg0: RW<u32>,
    pub phy_wr_dqs_cfg1: RW<u32>,
    pub phy_wr_dqs_cfg2: RW<u32>,
    pub phy_wr_dqs_cfg3: RW<u32>,
    _unused7: RO<u32>,
    pub phy_we_cfg0: RW<u32>,
    pub phy_we_cfg1: RW<u32>,
    pub phy_we_cfg2: RW<u32>,
    pub phy_we_cfg3: RW<u32>,
    _unused8: RO<u32>,
    pub wr_data_slv0: RW<u32>,
    pub wr_data_slv1: RW<u32>,
    pub wr_data_slv2: RW<u32>,
    pub wr_data_slv3: RW<u32>,
    _unused9: RO<u32>,
    pub reg_64: RW<u32>,
    pub reg_65: Reg65,
    _unused10: [RO<u32>; 3],
    pub reg69_6a0: RW<u32>,
    pub reg69_6a1: RW<u32>,
    _unused11: RO<u32>,
    pub reg6c_6d2: RW<u32>,
    pub reg6c_6d3: RW<u32>,
    pub reg6e_710: RW<u32>,
    pub reg6e_711: RW<u32>,
    pub reg6e_712: RW<u32>,
    pub reg6e_713: RW<u32>,
    pub phy_dll_sts0: RW<u32>,
    _unused12: RO<u32>,
    pub phy_dll_sts1: RW<u32>,
    pub phy_dll_sts2: RW<u32>,
    pub phy_dll_sts3: RW<u32>,
    _unused13: RO<u32>,
    pub dll_lock_sts: RW<u32>,
    pub phy_ctrl_sts: RW<u32>,
    pub phy_ctrl_sts2: RW<u32>,
    _unused14: [RO<u32>; 5],
    pub axi_id: RW<u32>,
    pub page_mask: RW<u32>,
    pub axi_priority_wr_port0: RW<u32>,
    pub axi_priority_wr_port1: RW<u32>,
    pub axi_priority_wr_port2: RW<u32>,
    pub axi_priority_wr_port3: RW<u32>,
    pub axi_priority_rd_port0: RW<u32>,
    pub axi_priority_rd_port1: RW<u32>,
    pub axi_priority_rd_port2: RW<u32>,
    pub axi_priority_rd_port3: RW<u32>,
    _unused15: [RO<u32>; 27],
    pub excl_access_cfg0: RW<u32>,
    pub excl_access_cfg1: RW<u32>,
    pub excl_access_cfg2: RW<u32>,
    pub excl_access_cfg3: RW<u32>,
    pub mode_reg_read: RW<u32>,
    pub lpddr_ctrl0: RW<u32>,
    pub lpddr_ctrl1: RW<u32>,
    pub lpddr_ctrl2: RW<u32>,
    pub lpddr_ctrl3: RW<u32>,
}

impl RegisterBlock {
    pub unsafe fn new() -> &'static mut Self {
        &mut *(0xF8006000 as *mut _)
    }
}

register!(ddrc_ctrl, DdrcCtrl, RW, u32);
register_bit!(ddrc_ctrl,
              /// `false` resets controller, `true` continues
              soft_rstb, 0);
register_bit!(ddrc_ctrl, powerdown_en, 1);
register_bits_typed!(ddrc_ctrl, data_bus_width, u8, DataBusWidth, 2, 3);
// (ddrc_ctrl) ...

register!(dram_param0, DramParam0, RW, u32);
register_bits!(dram_param0, t_rc, u8, 0, 5);
register_bits!(dram_param0, t_rfc_min, u8, 6, 13);
register_bits!(dram_param0, post_selfref_gap_x32, u8, 14, 20);

register!(dram_param2, DramParam2, RW, u32);
register_bits!(dram_param2, write_latency, u8, 0, 4);
register_bits!(dram_param2, rd2wr, u8, 5, 9);
register_bits!(dram_param2, wr2rd, u8, 10, 14);
register_bits!(dram_param2, t_xp, u8, 15, 19);
register_bits!(dram_param2, pad_pd, u8, 20, 22);
register_bits!(dram_param2, rd2pre, u8, 23, 27);
register_bits!(dram_param2, t_rcd, u8, 28, 31);

register!(dram_emr_mr, DramEmrMr, RW, u32);
register_bits!(dram_emr_mr, mr, u16, 0, 15);
register_bits!(dram_emr_mr, emr, u16, 16, 31);

register!(phy_cmd_timeout_rddata_cpt, PhyCmdTimeoutRddataCpt, RW, u32);
register_bits!(phy_cmd_timeout_rddata_cpt, rd_cmd_to_data, u8, 0, 3);
register_bits!(phy_cmd_timeout_rddata_cpt, wr_cmd_to_data, u8, 4, 7);
register_bits!(phy_cmd_timeout_rddata_cpt, we_to_re_delay, u8, 8, 11);
register_bit!(phy_cmd_timeout_rddata_cpt, rdc_fifo_rst_disable, 15);
register_bit!(phy_cmd_timeout_rddata_cpt, use_fixed_re, 16);
register_bit!(phy_cmd_timeout_rddata_cpt, rdc_fifo_rst_err_cnt_clr, 17);
register_bit!(phy_cmd_timeout_rddata_cpt, dis_phy_ctrl_rstn, 18);
register_bit!(phy_cmd_timeout_rddata_cpt, clk_stall_level, 19);
register_bits!(phy_cmd_timeout_rddata_cpt, gatelvl_num_of_dq0, u8, 24, 27);
register_bits!(phy_cmd_timeout_rddata_cpt, wrlvl_num_of_dq0, u8, 28, 31);

register!(reg_2c, Reg2C, RW, u32);
register_bits!(reg_2c, wrlvl_max_x1024, u16, 0, 11);
register_bits!(reg_2c, rdlvl_max_x1024, u16, 12, 23);
register_bit!(reg_2c, twrlvl_max_error, 24);
register_bit!(reg_2c, trdlvl_max_error, 25);
register_bit!(reg_2c, dfi_wr_level_en, 26);
register_bit!(reg_2c, dfi_rd_dqs_gate_level, 27);
register_bit!(reg_2c, dfi_rd_data_eye_train, 28);

register!(dfi_timing, DfiTiming, RW, u32);
register_bits!(dfi_timing, rddata_en, u8, 0, 4);
register_bits!(dfi_timing, ctrlup_min, u16, 5, 14);
register_bits!(dfi_timing, ctrlup_max, u16, 15, 24);

register!(phy_init_ratio, PhyInitRatio, RW, u32);
register_bits!(phy_init_ratio, wrlvl_init_ratio, u16, 0, 9);
register_bits!(phy_init_ratio, gatelvl_init_ratio, u16, 10, 19);

register!(reg_65, Reg65, RW, u32);
register_bits!(reg_65, wr_rl_delay, u8, 0, 4);
register_bits!(reg_65, rd_rl_delay, u8, 5, 9);
register_bits!(reg_65, dll_lock_diff, u8, 10, 13);
register_bit!(reg_65, use_wr_level, 14);
register_bit!(reg_65, use_rd_dqs_gate_level, 15);
register_bit!(reg_65, use_rd_data_eye_level, 16);
register_bit!(reg_65, dis_calib_rst, 17);
register_bits!(reg_65, ctrl_slave_delay, u8, 18, 19);

// Controller operation mode status
register!(mode_sts_reg,
          ModeStsReg, RO, u32);
register_bits_typed!(mode_sts_reg, operating_mode, u8, ControllerStatus, 0, 2);
// (mode_sts_reg) ...
