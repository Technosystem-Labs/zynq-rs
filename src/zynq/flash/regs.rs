use volatile_register::{RO, WO, RW};

use crate::{register, register_bit, register_bits};

#[repr(C)]
pub struct RegisterBlock {
    pub config: Config,
    pub intr_status: RW<u32>,
    pub intr_en: RW<u32>,
    pub intr_dis: RW<u32>,
    pub intr_mask: RO<u32>,
    pub enable: RW<u32>,
    pub delay: RW<u32>,
    pub txd0: WO<u32>,
    pub rx_data: RO<u32>,
    pub slave_idle_count: RW<u32>,
    pub tx_thres: RW<u32>,
    pub rx_thes: RW<u32>,
    pub gpio: RW<u32>,
    pub _unused1: RO<u32>,
    pub lpbk_dly_adj: RW<u32>,
    pub _unused2: [RO<u32>; 17],
    pub txd1: WO<u32>,
    pub txd2: WO<u32>,
    pub txd3: WO<u32>,
    pub _unused3: [RO<u32>; 5],
    pub lqspi_cfg: LqspiCfg,
    pub lqspi_sts: RW<u32>,
    pub _unused4: [RO<u32>; 21],
    pub mod_id: RW<u32>,
}

impl RegisterBlock {
    const BASE_ADDRESS: *mut Self = 0xE000D000 as *mut _;

    pub fn qspi() -> &'static mut Self {
        unsafe { &mut *Self::BASE_ADDRESS }
    }
}

register!(config, Config, RW, u32);
register_bit!(config,
              /// Enables master mode
              mode_sel, 0);
register_bit!(config,
              /// Clock polarity low/high
              clk_pol, 1);
register_bit!(config,
              /// Clock phase
              clk_ph, 2);
register_bits!(config,
               /// divider = 2 ** (1 + baud_rate_div)
               baud_rate_div, u8, 3, 5);
register_bits!(config,
               /// Must be set to 0b11
               fifo_width, u8, 6, 7);
register_bit!(config,
              /// Must be 0
              ref_clk, 8);
register_bit!(config,
              /// Peripheral Chip Select Line
              pcs, 10);
register_bit!(config,
              /// false: auto mode, true: manual CS mode
              manual_cs, 14);
register_bit!(config,
              /// false: auto mode, true: enables manual start enable
              man_start_en, 15);
register_bit!(config,
              /// false: auto mode, true: enables manual start command
              man_start_com, 16);
register_bit!(config, holdb_dr, 19);
register_bit!(config,
              /// false: little, true: endian
              endian, 26);
register_bit!(config,
              /// false: legacy SPI mode, true: Flash memory interface mode
              leg_flsh, 31);

register!(lqspi_cfg, LqspiCfg, RW, u32);
register_bits!(lqspi_cfg, inst_code, u8, 0, 7);
register_bits!(lqspi_cfg, dummy_byte, u8, 8, 10);
register_bits!(lqspi_cfg, mode_bits, u8, 16, 23);
register_bit!(lqspi_cfg, mode_on, 24);
register_bit!(lqspi_cfg, mode_en, 25);
register_bit!(lqspi_cfg, u_page, 28);
register_bit!(lqspi_cfg, sep_bus, 29);
register_bit!(lqspi_cfg, two_mem, 30);
register_bit!(lqspi_cfg, lq_mode, 31);
