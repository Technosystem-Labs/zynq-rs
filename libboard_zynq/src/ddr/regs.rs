use volatile_register::{RO, RW};

use libregister::{register, register_bit, register_bits_typed};

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
    pub hpr_reg: RW<u32>,
    pub lpr_reg: RW<u32>,
    pub wr_reg: RW<u32>,
    pub dram_param_reg0: RW<u32>,
    pub dram_param_reg1: RW<u32>,
    pub dram_param_reg2: RW<u32>,
    pub dram_param_reg3: RW<u32>,
    pub dram_param_reg4: RW<u32>,
    pub dram_init_param: RW<u32>,
    pub dram_emr_reg: RW<u32>,
    pub dram_emr_mr_reg: RW<u32>,
    pub dram_burst8_rdwr: RW<u32>,
    pub dram_disable_dq: RW<u32>,
    pub dram_addr_map_bank: RW<u32>,
    pub dram_addr_map_col: RW<u32>,
    pub dram_addr_map_row: RW<u32>,
    pub dram_odt_reg: RW<u32>,
    pub phy_dbg_reg: RW<u32>,
    pub phy_cmd_timeout_rddata_cpt: RW<u32>,
    pub mode_sts_reg: ModeStsReg,
    pub dll_calib: RW<u32>,
    pub odt_delay_hold: RW<u32>,
    pub ctrl_reg1: RW<u32>,
    pub ctrl_reg2: RW<u32>,
    pub ctrl_reg3: RW<u32>,
    pub ctrl_reg4: RW<u32>,
    _unused0: [RO<u32>; 2],
    pub ctrl_reg5: RW<u32>,
    pub ctrl_reg6: RW<u32>,
    _unused1: [RO<u32>; 8],
    pub che_refresh_timer01: RW<u32>,
    pub che_t_zq: RW<u32>,
    pub che_t_zq_short_interval_reg: RW<u32>,
    pub deep_pwrdwn_reg: RW<u32>,
    pub reg_2c: RW<u32>,
    pub reg_2d: RW<u32>,
    pub dfi_timing: RW<u32>,
    _unused2: [RO<u32>; 2],
    pub che_ecc_control_reg_offset: RW<u32>,
    pub che_corr_ecc_log_reg_offset: RW<u32>,
    pub che_corr_ecc_addr_reg_offset: RW<u32>,
    pub che_corr_ecc_data_31_0_reg_offset: RW<u32>,
    pub che_corr_ecc_data_63_32_reg_offset: RW<u32>,
    pub che_corr_ecc_data_71_64_reg_offset: RW<u32>,
    pub che_uncorr_ecc_log_reg_offset: RW<u32>,
    pub che_uncorr_ecc_addr_reg_offset: RW<u32>,
    pub che_uncorr_ecc_data_31_0_reg_offset: RW<u32>,
    pub che_uncorr_ecc_data_63_32_reg_offset: RW<u32>,
    pub che_uncorr_ecc_data_71_64_reg_offset: RW<u32>,
    pub che_ecc_stats_reg_offset: RW<u32>,
    pub ecc_scrub: RW<u32>,
    pub che_ecc_corr_bit_mask_31_0_reg_offset: RW<u32>,
    pub che_ecc_corr_bit_mask_63_32_reg_offset: RW<u32>,
    _unused3: [RO<u32>; 5],
    pub phy_rcvr_enable: RW<u32>,
    pub phy_config0: RW<u32>,
    pub phy_config1: RW<u32>,
    pub phy_config2: RW<u32>,
    pub phy_config3: RW<u32>,
    _unused4: RO<u32>,
    pub phy_init_ratio0: RW<u32>,
    pub phy_init_ratio1: RW<u32>,
    pub phy_init_ratio2: RW<u32>,
    pub phy_init_ratio3: RW<u32>,
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
    pub reg_65: RW<u32>,
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
    pub phy_ctrl_sts_reg2: RW<u32>,
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

// Controller operation mode status
register!(mode_sts_reg,
          ModeStsReg, RO, u32);
register_bits_typed!(mode_sts_reg, operating_mode, u8, ControllerStatus, 0, 2);
// (mode_sts_reg) ...
