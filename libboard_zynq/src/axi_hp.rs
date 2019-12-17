//! AXI_HP Interface (AFI)

use volatile_register::RW;

use libregister::{register, register_bit, register_bits};

pub unsafe fn axi_hp0() -> &'static RegisterBlock {
    &*(0xF8008000 as *const _)
}

pub unsafe fn axi_hp1() -> &'static RegisterBlock {
    &*(0xF8009000 as *const _)
}

pub unsafe fn axi_hp2() -> &'static RegisterBlock {
    &*(0xF800A000 as *const _)
}

pub unsafe fn axi_hp3() -> &'static RegisterBlock {
    &*(0xF800B000 as *const _)
}

#[repr(C)]
pub struct RegisterBlock {
    /// Read Channel Control Register
    pub rdchan_ctrl: RdchanCtrl,
    /// Read Issuing Capability Register
    pub rdchan_issuingcap: RW<u32>,
    /// QOS Read Channel Register
    pub rdqos: RW<u32>,
    /// Read Data FIFO Level Register
    pub rddatafifo_level: RW<u32>,
    /// Read Channel Debug Register
    pub rddebug: RW<u32>,
    /// Write Channel Control Register
    pub wrchan_ctrl: WrchanCtrl,
    /// Write Issuing Capability Register
    pub wrchan_issuingcap: RW<u32>,
    /// QOS Write Channel Register
    pub wrqos: RW<u32>,
    /// Write Data FIFO Level Register
    pub wrdatafifo_level: RW<u32>,
    /// Write Channel Debug Register
    pub wrdebug: RW<u32>,
}

register!(rdchan_ctrl, RdchanCtrl, RW, u32);
register_bit!(rdchan_ctrl, en_32bit, 0);
register_bit!(rdchan_ctrl, fabric_qos_en, 1);
register_bit!(rdchan_ctrl, fabric_out_cmd_en, 2);
register_bit!(rdchan_ctrl, qos_head_of_cmd_q_en, 3);

register!(wrchan_ctrl, WrchanCtrl, RW, u32);
register_bit!(wrchan_ctrl, en_32bit, 0);
register_bit!(wrchan_ctrl, fabric_qos_en, 1);
register_bit!(wrchan_ctrl, fabric_out_cmd_en, 2);
register_bit!(wrchan_ctrl, qos_head_of_cmd_q_en, 3);
register_bits!(wrchan_ctrl, wr_cmd_release_mode, u8, 4, 5);
register_bits!(wrchan_ctrl, wr_data_threshold, u8, 8, 11);
