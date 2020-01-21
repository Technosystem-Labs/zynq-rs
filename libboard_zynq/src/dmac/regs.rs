use volatile_register::{RO, WO, RW};

use libregister::{
    register, register_at,
    register_bit, register_bits, register_bits_typed,
};

#[allow(unused)]
#[repr(C)]
pub struct RegisterBlock {
    pub ds: Ds,
    pub dpc: DPc,
    pub inten: Inten,
    pub es: Es,
    pub intstatus: IntStatus,
    pub intclr: IntClr,
    pub fsm: Fsm,
    pub fsc: Fsc,
    pub ftm: Ftm,
    pub ftc0: Ftc0,
    pub xdmaps_ftcn_offset_1: XDmaPsFtcnOffset1,
    pub xdmaps_ftcn_offset_2: XDmaPsFtcnOffset2,
    pub xdmaps_ftcn_offset_3: XDmaPsFtcnOffset3,
    pub xdmaps_ftcn_offset_4: XDmaPsFtcnOffset4,
    pub xdmaps_ftcn_offset_5: XDmaPsFtcnOffset5,
    pub xdmaps_ftcn_offset_6: XDmaPsFtcnOffset6,
    pub xdmaps_ftcn_offset_7: XDmaPsFtcnOffset7,
    pub cs0: Cs0,
    pub cpc0: Cpc0,
    pub xdmaps_csn_offset_1:  XDmaPsCSnOffset1,
    pub xdmaps_cpcn_offset_1: XDmaPsCPCnOffset1,
    pub xdmaps_csn_offset_2:  XDmaPsCSnOffset2,
    pub xdmaps_cpcn_offset_2: XDmaPsCPCnOffset2,
    pub xdmaps_csn_offset_3:  XDmaPsCSnOffset3,
    pub xdmaps_cpcn_offset_3: XDmaPsCPCnOffset3,
    pub xdmaps_csn_offset_4:  XDmaPsCSnOffset4,
    pub xdmaps_cpcn_offset_4: XDmaPsCPCnOffset4,
    pub xdmaps_csn_offset_5:  XDmaPsCSnOffset5,
    pub xdmaps_cpcn_offset_5: XDmaPsCPCnOffset5,
    pub xdmaps_csn_offset_6:  XDmaPsCSnOffset6,
    pub xdmaps_cpcn_offset_6: XDmaPsCPCnOffset6,
    pub xdmaps_csn_offset_7:  XDmaPsCSnOffset7,
    pub xdmaps_cpcn_offset_7: XDmaPsCPCnOffset7,
    pub sa_0: Sa0,
    pub da_0: Da0,
    pub cc_0: Cc0,
    pub lc0_0: Lc00,
    pub lc1_0: Lc10,
    pub xdmaps_sa_n_offset_1:  XDmaPsSaNOffset1,
    pub xdmaps_da_n_offset_1:  XDmaPsDaNOffset1,
    pub xdmaps_cc_n_offset_1:  XDmaPsCcNOffset1,
    pub xdmaps_lc0_n_offset_1: XDmaPsLc0NOffset1,
    pub xdmaps_lc1_n_offset_1: XDmaPsLc1NOffset1,
    pub xdmaps_sa_n_offset_2:  XDmaPsSaNOffset2,
    pub xdmaps_da_n_offset_2:  XDmaPsDaNOffset2,
    pub xdmaps_cc_n_offset_2:  XDmaPsCcNOffset2,
    pub xdmaps_lc0_n_offset_2: XDmaPsLc0NOffset2,
    pub xdmaps_lc1_n_offset_2: XDmaPsLc1NOffset2,
    pub xdmaps_sa_n_offset_3:  XDmaPsSaNOffset3,
    pub xdmaps_da_n_offset_3:  XDmaPsDaNOffset3,
    pub xdmaps_cc_n_offset_3:  XDmaPsCcNOffset3,
    pub xdmaps_lc0_n_offset_3: XDmaPsLc0NOffset3,
    pub xdmaps_lc1_n_offset_3: XDmaPsLc1NOffset3,
    pub xdmaps_sa_n_offset_4:  XDmaPsSaNOffset4,
    pub xdmaps_da_n_offset_4:  XDmaPsDaNOffset4,
    pub xdmaps_cc_n_offset_4:  XDmaPsCcNOffset4,
    pub xdmaps_lc0_n_offset_4: XDmaPsLc0NOffset4,
    pub xdmaps_lc1_n_offset_4: XDmaPsLc1NOffset4,
    pub xdmaps_sa_n_offset_5:  XDmaPsSaNOffset5,
    pub xdmaps_da_n_offset_5:  XDmaPsDaNOffset5,
    pub xdmaps_cc_n_offset_5:  XDmaPsCcNOffset5,
    pub xdmaps_lc0_n_offset_5: XDmaPsLc0NOffset5,
    pub xdmaps_lc1_n_offset_5: XDmaPsLc1NOffset5,
    pub xdmaps_sa_n_offset_6:  XDmaPsSaNOffset6,
    pub xdmaps_da_n_offset_6:  XDmaPsDaNOffset6,
    pub xdmaps_cc_n_offset_6:  XDmaPsCcNOffset6,
    pub xdmaps_lc0_n_offset_6: XDmaPsLc0NOffset6,
    pub xdmaps_lc1_n_offset_6: XDmaPsLc1NOffset6,
    pub xdmaps_sa_n_offset_7:  XDmaPsSaNOffset7,
    pub xdmaps_da_n_offset_7:  XDmaPsDaNOffset7,
    pub xdmaps_cc_n_offset_7:  XDmaPsCcNOffset7,
    pub xdmaps_lc0_n_offset_7: XDmaPsLc0NOffset7,
    pub xdmaps_lc1_n_offset_7: XDmaPsLc1NOffset7,
    pub dbgstatus: DbgStatus,
    pub dbgcmd: DbgCmd,
    pub dbginst0: DbgInst0,
    pub dbginst1: DbgInst1,
    pub cr0: Cr0,
    pub cr1: Cr1,
    pub cr2: Cr2,
    pub cr3: Cr3,
    pub cr4: Cr4,
    pub crdn: Crdn,
    pub wd: Wd,
    pub periph_id_0: PeriphId0,
    pub periph_id_1: PeriphId1,
    pub periph_id_2: PeriphId2,
    pub periph_id_3: PeriphId3,
    pub pcell_id_0: PCellId0,
    pub pcell_id_1: PCellId1,
    pub pcell_id_2: PCellId2,
    pub pcell_id_3: PCellId3,

}

register_at!(RegisterBlock, 0xF8004000, dmac0_ns);
register_at!(RegisterBlock, 0xF8003000, dmac0_s);

#[allow(unused)]
#[repr(u8)]
pub enum WakeUpEvent{
    // @missing: there's a binary prefix ahead of this as per TRM 1173 Wakeup_event
    Event0  = 0b0000,
    Event1  = 0b0001,
    Event2  = 0b0010,
    Event3  = 0b0011,
    Event4  = 0b0100,
    Event5  = 0b0101,
    Event6  = 0b0110,
    Event7  = 0b0111,
    Event8  = 0b1000,
    Event9  = 0b1001,
    Event10 = 0b1010,
    Event11 = 0b1011,
    Event12 = 0b1100,
    Event13 = 0b1101,
    Event14 = 0b1110,
    Event15 = 0b1111,
}
#[allow(unused)]
#[repr(u8)]
pub enum DMAStatus{
    Stopped         = 0b0000,
    Executing       = 0b0001,
    CacheMiss       = 0b0010,
    UpdatingPc      = 0b0011,
    WaitingForEvent = 0b0100,
    Reserved0       = 0b0101,
    Reserved1       = 0b0110,
    Reserved2       = 0b0111,
    Reserved3       = 0b1000,
    Reserved4       = 0b1001,
    Reserved5       = 0b1010,
    Reserved6       = 0b1011,
    Reserved7       = 0b1100,
    Reserved8       = 0b1101,
    Reserved9       = 0b1110,
    Faulting        = 0b1111,
}

register!(ds, Ds, RW, u32);
register_bit!(ds, dns, 9);
register_bits_typed!(ds, wakeup_event, u8, WakeUpEvent, 4, 8);
register_bits_typed!(ds, dma_status, u8, DMAStatus, 0, 3);

register!(dpc, DPc, RW, u32);
register_bits!(dpc, pc_mgr, u8, 0, 31);

register!(inten, Inten, RW, u32);
register_bits!(inten, event_irq_select, u8, 0, 31);

register!(es, Es, RW, u32);
register_bits!(es, dmasev_active, u8, 0, 31);

register!(intstatus, IntStatus, RW, u32);
register_bits!(intstatus, irq_status, u8, 0, 31);

register!(intclr, IntClr, RW, u32);
register_bits!(intstatus, irq_clr, u8, 0, 31);

register!(fsm, Fsm, RW, u32);
register_bit!(fsm, fs_mgr, 0);

register!(fsc, Fsc, RW, u32);
register_bits!(fsc, fault_status, u8, 0, 7);

register!(ftm, Ftm, RW, u32);
register_bit!(ftm, dbg_instr, 30);
register_bit!(ftm, instr_fetch_err, 16);
register_bit!(ftm, mgr_evnt_err, 5);
register_bit!(ftm, dmago_err, 4);
register_bit!(ftm, operand_invalid, 1);
register_bit!(ftm, undef_instr, 0);

register!(ftc0, Ftc0, RW, u32);
register_bit!(ftc0, lockup_err, 31);
register_bit!(ftc0, dbg_instr, 30);
register_bit!(ftc0, data_read_err, 18);
register_bit!(ftc0, data_write_err, 17);
register_bit!(ftc0, instr_fetch_err, 16);
register_bit!(ftc0, st_data_unavailable, 13);
register_bit!(ftc0, mfifo_err, 12);
register_bit!(ftc0, ch_rdwr_err, 7);
register_bit!(ftc0, ch_periph_err, 6);
register_bit!(ftc0, ch_evnt_err, 5);
register_bit!(ftc0, operand_invalid, 1);
register_bit!(ftc0, undef_instr, 0);

register!(xdmaps_ftcn_offset_1, XDmaPsFtcnOffset1, RW, u32);
register_bit!(xdmaps_ftcn_offset_1, lockup_err, 31);
register_bit!(xdmaps_ftcn_offset_1, dbg_instr, 30);
register_bit!(xdmaps_ftcn_offset_1, data_read_err, 18);
register_bit!(xdmaps_ftcn_offset_1, data_write_err, 17);
register_bit!(xdmaps_ftcn_offset_1, instr_fetch_err, 16);
register_bit!(xdmaps_ftcn_offset_1, st_data_unavailable, 13);
register_bit!(xdmaps_ftcn_offset_1, mfifo_err, 12);
register_bit!(xdmaps_ftcn_offset_1, ch_rdwr_err, 7);
register_bit!(xdmaps_ftcn_offset_1, ch_periph_err, 6);
register_bit!(xdmaps_ftcn_offset_1, ch_evnt_err, 5);
register_bit!(xdmaps_ftcn_offset_1, operand_invalid, 1);
register_bit!(xdmaps_ftcn_offset_1, undef_instr, 0);

register!(xdmaps_ftcn_offset_2, XDmaPsFtcnOffset2, RW, u32);
register_bit!(xdmaps_ftcn_offset_2, lockup_err, 31);
register_bit!(xdmaps_ftcn_offset_2, dbg_instr, 30);
register_bit!(xdmaps_ftcn_offset_2, data_read_err, 18);
register_bit!(xdmaps_ftcn_offset_2, data_write_err, 17);
register_bit!(xdmaps_ftcn_offset_2, instr_fetch_err, 16);
register_bit!(xdmaps_ftcn_offset_2, st_data_unavailable, 13);
register_bit!(xdmaps_ftcn_offset_2, mfifo_err, 12);
register_bit!(xdmaps_ftcn_offset_2, ch_rdwr_err, 7);
register_bit!(xdmaps_ftcn_offset_2, ch_periph_err, 6);
register_bit!(xdmaps_ftcn_offset_2, ch_evnt_err, 5);
register_bit!(xdmaps_ftcn_offset_2, operand_invalid, 1);
register_bit!(xdmaps_ftcn_offset_2, undef_instr, 0);

register!(xdmaps_ftcn_offset_3, XDmaPsFtcnOffset3, RW, u32);
register_bit!(xdmaps_ftcn_offset_3, lockup_err, 31);
register_bit!(xdmaps_ftcn_offset_3, dbg_instr, 30);
register_bit!(xdmaps_ftcn_offset_3, data_read_err, 18);
register_bit!(xdmaps_ftcn_offset_3, data_write_err, 17);
register_bit!(xdmaps_ftcn_offset_3, instr_fetch_err, 16);
register_bit!(xdmaps_ftcn_offset_3, st_data_unavailable, 13);
register_bit!(xdmaps_ftcn_offset_3, mfifo_err, 12);
register_bit!(xdmaps_ftcn_offset_3, ch_rdwr_err, 7);
register_bit!(xdmaps_ftcn_offset_3, ch_periph_err, 6);
register_bit!(xdmaps_ftcn_offset_3, ch_evnt_err, 5);
register_bit!(xdmaps_ftcn_offset_3, operand_invalid, 1);
register_bit!(xdmaps_ftcn_offset_3, undef_instr, 0);

register!(xdmaps_ftcn_offset_4, XDmaPsFtcnOffset4, RW, u32);
register_bit!(xdmaps_ftcn_offset_4, lockup_err, 31);
register_bit!(xdmaps_ftcn_offset_4, dbg_instr, 30);
register_bit!(xdmaps_ftcn_offset_4, data_read_err, 18);
register_bit!(xdmaps_ftcn_offset_4, data_write_err, 17);
register_bit!(xdmaps_ftcn_offset_4, instr_fetch_err, 16);
register_bit!(xdmaps_ftcn_offset_4, st_data_unavailable, 13);
register_bit!(xdmaps_ftcn_offset_4, mfifo_err, 12);
register_bit!(xdmaps_ftcn_offset_4, ch_rdwr_err, 7);
register_bit!(xdmaps_ftcn_offset_4, ch_periph_err, 6);
register_bit!(xdmaps_ftcn_offset_4, ch_evnt_err, 5);
register_bit!(xdmaps_ftcn_offset_4, operand_invalid, 1);
register_bit!(xdmaps_ftcn_offset_4, undef_instr, 0);

register!(xdmaps_ftcn_offset_5, XDmaPsFtcnOffset5, RW, u32);
register_bit!(xdmaps_ftcn_offset_5, lockup_err, 31);
register_bit!(xdmaps_ftcn_offset_5, dbg_instr, 30);
register_bit!(xdmaps_ftcn_offset_5, data_read_err, 18);
register_bit!(xdmaps_ftcn_offset_5, data_write_err, 17);
register_bit!(xdmaps_ftcn_offset_5, instr_fetch_err, 16);
register_bit!(xdmaps_ftcn_offset_5, st_data_unavailable, 13);
register_bit!(xdmaps_ftcn_offset_5, mfifo_err, 12);
register_bit!(xdmaps_ftcn_offset_5, ch_rdwr_err, 7);
register_bit!(xdmaps_ftcn_offset_5, ch_periph_err, 6);
register_bit!(xdmaps_ftcn_offset_5, ch_evnt_err, 5);
register_bit!(xdmaps_ftcn_offset_5, operand_invalid, 1);
register_bit!(xdmaps_ftcn_offset_5, undef_instr, 0);

register!(xdmaps_ftcn_offset_6, XDmaPsFtcnOffset6, RW, u32);
register_bit!(xdmaps_ftcn_offset_6, lockup_err, 31);
register_bit!(xdmaps_ftcn_offset_6, dbg_instr, 30);
register_bit!(xdmaps_ftcn_offset_6, data_read_err, 18);
register_bit!(xdmaps_ftcn_offset_6, data_write_err, 17);
register_bit!(xdmaps_ftcn_offset_6, instr_fetch_err, 16);
register_bit!(xdmaps_ftcn_offset_6, st_data_unavailable, 13);
register_bit!(xdmaps_ftcn_offset_6, mfifo_err, 12);
register_bit!(xdmaps_ftcn_offset_6, ch_rdwr_err, 7);
register_bit!(xdmaps_ftcn_offset_6, ch_periph_err, 6);
register_bit!(xdmaps_ftcn_offset_6, ch_evnt_err, 5);
register_bit!(xdmaps_ftcn_offset_6, operand_invalid, 1);
register_bit!(xdmaps_ftcn_offset_6, undef_instr, 0);

register!(xdmaps_ftcn_offset_7, XDmaPsFtcnOffset7, RW, u32);
register_bit!(xdmaps_ftcn_offset_7, lockup_err, 31);
register_bit!(xdmaps_ftcn_offset_7, dbg_instr, 30);
register_bit!(xdmaps_ftcn_offset_7, data_read_err, 18);
register_bit!(xdmaps_ftcn_offset_7, data_write_err, 17);
register_bit!(xdmaps_ftcn_offset_7, instr_fetch_err, 16);
register_bit!(xdmaps_ftcn_offset_7, st_data_unavailable, 13);
register_bit!(xdmaps_ftcn_offset_7, mfifo_err, 12);
register_bit!(xdmaps_ftcn_offset_7, ch_rdwr_err, 7);
register_bit!(xdmaps_ftcn_offset_7, ch_periph_err, 6);
register_bit!(xdmaps_ftcn_offset_7, ch_evnt_err, 5);
register_bit!(xdmaps_ftcn_offset_7, operand_invalid, 1);
register_bit!(xdmaps_ftcn_offset_7, undef_instr, 0);

register!(cs0, Cs0, RW, u32);
register_bit!(cs0, cns, 21);
register_bit!(cs0, dmawfp_periph, 15);
register_bit!(cs0, dmawfp_b_ns, 14);
register_bits!(cs0, wakeup_num, u8, 4, 8);
register_bits!(cs0, channel_status, u8, 0, 3);

register!(cpc0, Cpc0, RW, u32);
register_bits!(cpc0, pc_chnl, u8, 0, 31);

register!(xdmaps_csn_offset_1, XDmaPsCSnOffset1, RW, u32);
register_bit!(xdmaps_csn_offset_1, cns, 21);
register_bit!(xdmaps_csn_offset_1, dmawfp_periph, 15);
register_bit!(xdmaps_csn_offset_1, dmawfp_b_ns, 14);
register_bits!(xdmaps_csn_offset_1, wakeup_num, u8, 4, 8);
register_bits!(xdmaps_csn_offset_1, channel_status, u8, 0, 3);

register!(xdmaps_cpcn_offset_1, XDmaPsCPCnOffset1, RW, u32);
register_bits!(xdmaps_cpcn_offset_1, pc_chnl, u8, 0, 31);

register!(xdmaps_csn_offset_2, XDmaPsCSnOffset2, RW, u32);
register_bit!(xdmaps_csn_offset_2, cns, 21);
register_bit!(xdmaps_csn_offset_2, dmawfp_periph, 15);
register_bit!(xdmaps_csn_offset_2, dmawfp_b_ns, 14);
register_bits!(xdmaps_csn_offset_2, wakeup_num, u8, 4, 8);
register_bits!(xdmaps_csn_offset_2, channel_status, u8, 0, 3);

register!(xdmaps_cpcn_offset_2, XDmaPsCPCnOffset2, RW, u32);
register_bits!(xdmaps_cpcn_offset_2, pc_chnl, u8, 0, 31);

register!(xdmaps_csn_offset_3, XDmaPsCSnOffset3, RW, u32);
register_bit!(xdmaps_csn_offset_3, cns, 21);
register_bit!(xdmaps_csn_offset_3, dmawfp_periph, 15);
register_bit!(xdmaps_csn_offset_3, dmawfp_b_ns, 14);
register_bits!(xdmaps_csn_offset_3, wakeup_num, u8, 4, 8);
register_bits!(xdmaps_csn_offset_3, channel_status, u8, 0, 3);

register!(xdmaps_cpcn_offset_3, XDmaPsCPCnOffset3, RW, u32);
register_bits!(xdmaps_cpcn_offset_3, pc_chnl, u8, 0, 31);

register!(xdmaps_csn_offset_4, XDmaPsCSnOffset4, RW, u32);
register_bit!(xdmaps_csn_offset_4, cns, 21);
register_bit!(xdmaps_csn_offset_4, dmawfp_periph, 15);
register_bit!(xdmaps_csn_offset_4, dmawfp_b_ns, 14);
register_bits!(xdmaps_csn_offset_4, wakeup_num, u8, 4, 8);
register_bits!(xdmaps_csn_offset_4, channel_status, u8, 0, 3);

register!(xdmaps_cpcn_offset_4, XDmaPsCPCnOffset4, RW, u32);
register_bits!(xdmaps_cpcn_offset_4, pc_chnl, u8, 0, 31);

register!(xdmaps_csn_offset_5, XDmaPsCSnOffset5, RW, u32);
register_bit!(xdmaps_csn_offset_5, cns, 21);
register_bit!(xdmaps_csn_offset_5, dmawfp_periph, 15);
register_bit!(xdmaps_csn_offset_5, dmawfp_b_ns, 14);
register_bits!(xdmaps_csn_offset_5, wakeup_num, u8, 4, 8);
register_bits!(xdmaps_csn_offset_5, channel_status, u8, 0, 3);

register!(xdmaps_cpcn_offset_5, XDmaPsCPCnOffset5, RW, u32);
register_bits!(xdmaps_cpcn_offset_5, pc_chnl, u8, 0, 31);

register!(xdmaps_csn_offset_6, XDmaPsCSnOffset6, RW, u32);
register_bit!(xdmaps_csn_offset_6, cns, 21);
register_bit!(xdmaps_csn_offset_6, dmawfp_periph, 15);
register_bit!(xdmaps_csn_offset_6, dmawfp_b_ns, 14);
register_bits!(xdmaps_csn_offset_6, wakeup_num, u8, 4, 8);
register_bits!(xdmaps_csn_offset_6, channel_status, u8, 0, 3);

register!(xdmaps_cpcn_offset_6, XDmaPsCPCnOffset6, RW, u32);
register_bits!(xdmaps_cpcn_offset_6, pc_chnl, u8, 0, 31);

register!(xdmaps_csn_offset_7, XDmaPsCSnOffset7, RW, u32);
register_bit!(xdmaps_csn_offset_7, cns, 21);
register_bit!(xdmaps_csn_offset_7, dmawfp_periph, 15);
register_bit!(xdmaps_csn_offset_7, dmawfp_b_ns, 14);
register_bits!(xdmaps_csn_offset_7, wakeup_num, u8, 4, 8);
register_bits!(xdmaps_csn_offset_7, channel_status, u8, 0, 3);

register!(xdmaps_cpcn_offset_7, XDmaPsCPCnOffset7, RW, u32);
register_bits!(xdmaps_cpcn_offset_7, pc_chnl, u8, 0, 31);

register!(sa_0, Sa0, RW, u32);
register_bits!(sa_0, src_addr, u8, 0, 31);

register!(da_0, Da0, RW, u32);
register_bits!(da_0, dest_addr, u8, 0, 31);

register!(cc_0, Cc0, RW, u32);
register_bits!(cc_0, endian_swap_size, u8, 28, 30);
register_bits!(cc_0, dst_cache_ctrl, u8, 25, 27);
register_bits!(cc_0, dst_prot_ctrl, u8, 22, 24);
register_bits!(cc_0, dst_burst_len, u8, 18, 21);
register_bits!(cc_0, dst_burst_size, u8, 15, 17);
register_bit!(cc_0, dst_inc, 14);
register_bits!(cc_0, src_cache_ctrl, u8, 11, 13);
register_bits!(cc_0, src_prot_ctrl, u8, 8, 10);
register_bits!(cc_0, src_burst_len, u8, 4, 7);
register_bits!(cc_0, src_burst_size, u8, 1, 3);
register_bit!(cc_0, src_inc, 0);

register!(lc0_0, Lc00, RW, u32);
register_bits!(lc0_0, loop_counter_iteration, u8, 0, 7);

register!(lc1_0, Lc10, RW, u32);
register_bits!(lc1_0, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_sa_n_offset_1, XDmaPsSaNOffset1, RW, u32);
register_bits!(xdmaps_sa_n_offset_1, src_addr, u8, 0, 31);

register!(xdmaps_da_n_offset_1, XDmaPsDaNOffset1, RW, u32);
register_bits!(xdmaps_da_n_offset_1, dest_addr, u8, 0, 31);

register!(xdmaps_cc_n_offset_1, XDmaPsCcNOffset1, RW, u32);
register_bits!(xdmaps_cc_n_offset_1, endian_swap_size, u8, 28, 30);
register_bits!(xdmaps_cc_n_offset_1, dst_cache_ctrl, u8, 25, 27);
register_bits!(xdmaps_cc_n_offset_1, dst_prot_ctrl, u8, 22, 24);
register_bits!(xdmaps_cc_n_offset_1, dst_burst_len, u8, 18, 21);
register_bits!(xdmaps_cc_n_offset_1, dst_burst_size, u8, 15, 17);
register_bit!(xdmaps_cc_n_offset_1,  dst_inc, 14);
register_bits!(xdmaps_cc_n_offset_1, src_cache_ctrl, u8, 11, 13);
register_bits!(xdmaps_cc_n_offset_1, src_prot_ctrl, u8, 8, 10);
register_bits!(xdmaps_cc_n_offset_1, src_burst_len, u8, 4, 7);
register_bits!(xdmaps_cc_n_offset_1, src_burst_size, u8, 1, 3);
register_bit!(xdmaps_cc_n_offset_1,  src_inc, 0);

register!(xdmaps_lc0_n_offset_1, XDmaPsLc0NOffset1, RW, u32);
register_bits!(xdmaps_lc0_n_offset_1, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_lc1_n_offset_1, XDmaPsLc1NOffset1, RW, u32);
register_bits!(xdmaps_lc1_n_offset_1, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_sa_n_offset_2, XDmaPsSaNOffset2, RW, u32);
register_bits!(xdmaps_sa_n_offset_2, src_addr, u8, 0, 31);

register!(xdmaps_da_n_offset_2, XDmaPsDaNOffset2, RW, u32);
register_bits!(xdmaps_da_n_offset_2, dest_addr, u8, 0, 31);

register!(xdmaps_cc_n_offset_2, XDmaPsCcNOffset2, RW, u32);
register_bits!(xdmaps_cc_n_offset_2, endian_swap_size, u8, 28, 30);
register_bits!(xdmaps_cc_n_offset_2, dst_cache_ctrl, u8, 25, 27);
register_bits!(xdmaps_cc_n_offset_2, dst_prot_ctrl, u8, 22, 24);
register_bits!(xdmaps_cc_n_offset_2, dst_burst_len, u8, 18, 21);
register_bits!(xdmaps_cc_n_offset_2, dst_burst_size, u8, 15, 17);
register_bit!(xdmaps_cc_n_offset_2,  dst_inc, 14);
register_bits!(xdmaps_cc_n_offset_2, src_cache_ctrl, u8, 11, 13);
register_bits!(xdmaps_cc_n_offset_2, src_prot_ctrl, u8, 8, 10);
register_bits!(xdmaps_cc_n_offset_2, src_burst_len, u8, 4, 7);
register_bits!(xdmaps_cc_n_offset_2, src_burst_size, u8, 1, 3);
register_bit!(xdmaps_cc_n_offset_2,  src_inc, 0);

register!(xdmaps_lc0_n_offset_2, XDmaPsLc0NOffset2, RW, u32);
register_bits!(xdmaps_lc0_n_offset_2, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_lc1_n_offset_2, XDmaPsLc1NOffset2, RW, u32);
register_bits!(xdmaps_lc1_n_offset_2, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_sa_n_offset_3, XDmaPsSaNOffset3, RW, u32);
register_bits!(xdmaps_sa_n_offset_3, src_addr, u8, 0, 31);

register!(xdmaps_da_n_offset_3, XDmaPsDaNOffset3, RW, u32);
register_bits!(xdmaps_da_n_offset_3, dest_addr, u8, 0, 31);

register!(xdmaps_cc_n_offset_3, XDmaPsCcNOffset3, RW, u32);
register_bits!(xdmaps_cc_n_offset_3, endian_swap_size, u8, 28, 30);
register_bits!(xdmaps_cc_n_offset_3, dst_cache_ctrl, u8, 25, 27);
register_bits!(xdmaps_cc_n_offset_3, dst_prot_ctrl, u8, 22, 24);
register_bits!(xdmaps_cc_n_offset_3, dst_burst_len, u8, 18, 21);
register_bits!(xdmaps_cc_n_offset_3, dst_burst_size, u8, 15, 17);
register_bit!(xdmaps_cc_n_offset_3,  dst_inc, 14);
register_bits!(xdmaps_cc_n_offset_3, src_cache_ctrl, u8, 11, 13);
register_bits!(xdmaps_cc_n_offset_3, src_prot_ctrl, u8, 8, 10);
register_bits!(xdmaps_cc_n_offset_3, src_burst_len, u8, 4, 7);
register_bits!(xdmaps_cc_n_offset_3, src_burst_size, u8, 1, 3);
register_bit!(xdmaps_cc_n_offset_3,  src_inc, 0);

register!(xdmaps_lc0_n_offset_3, XDmaPsLc0NOffset3, RW, u32);
register_bits!(xdmaps_lc0_n_offset_3, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_lc1_n_offset_3, XDmaPsLc1NOffset3, RW, u32);
register_bits!(xdmaps_lc1_n_offset_3, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_sa_n_offset_4, XDmaPsSaNOffset4, RW, u32);
register_bits!(xdmaps_sa_n_offset_4, src_addr, u8, 0, 31);

register!(xdmaps_da_n_offset_4, XDmaPsDaNOffset4, RW, u32);
register_bits!(xdmaps_da_n_offset_4, dest_addr, u8, 0, 31);

register!(xdmaps_cc_n_offset_4, XDmaPsCcNOffset4, RW, u32);
register_bits!(xdmaps_cc_n_offset_4, endian_swap_size, u8, 28, 30);
register_bits!(xdmaps_cc_n_offset_4, dst_cache_ctrl, u8, 25, 27);
register_bits!(xdmaps_cc_n_offset_4, dst_prot_ctrl, u8, 22, 24);
register_bits!(xdmaps_cc_n_offset_4, dst_burst_len, u8, 18, 21);
register_bits!(xdmaps_cc_n_offset_4, dst_burst_size, u8, 15, 17);
register_bit!(xdmaps_cc_n_offset_4,  dst_inc, 14);
register_bits!(xdmaps_cc_n_offset_4, src_cache_ctrl, u8, 11, 13);
register_bits!(xdmaps_cc_n_offset_4, src_prot_ctrl, u8, 8, 10);
register_bits!(xdmaps_cc_n_offset_4, src_burst_len, u8, 4, 7);
register_bits!(xdmaps_cc_n_offset_4, src_burst_size, u8, 1, 3);
register_bit!(xdmaps_cc_n_offset_4,  src_inc, 0);

register!(xdmaps_lc0_n_offset_4, XDmaPsLc0NOffset4, RW, u32);
register_bits!(xdmaps_lc0_n_offset_4, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_lc1_n_offset_4, XDmaPsLc1NOffset4, RW, u32);
register_bits!(xdmaps_lc1_n_offset_4, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_sa_n_offset_5, XDmaPsSaNOffset5, RW, u32);
register_bits!(xdmaps_sa_n_offset_5, src_addr, u8, 0, 31);

register!(xdmaps_da_n_offset_5, XDmaPsDaNOffset5, RW, u32);
register_bits!(xdmaps_da_n_offset_5, dest_addr, u8, 0, 31);

register!(xdmaps_cc_n_offset_5, XDmaPsCcNOffset5, RW, u32);
register_bits!(xdmaps_cc_n_offset_5, endian_swap_size, u8, 28, 30);
register_bits!(xdmaps_cc_n_offset_5, dst_cache_ctrl, u8, 25, 27);
register_bits!(xdmaps_cc_n_offset_5, dst_prot_ctrl, u8, 22, 24);
register_bits!(xdmaps_cc_n_offset_5, dst_burst_len, u8, 18, 21);
register_bits!(xdmaps_cc_n_offset_5, dst_burst_size, u8, 15, 17);
register_bit!(xdmaps_cc_n_offset_5,  dst_inc, 14);
register_bits!(xdmaps_cc_n_offset_5, src_cache_ctrl, u8, 11, 13);
register_bits!(xdmaps_cc_n_offset_5, src_prot_ctrl, u8, 8, 10);
register_bits!(xdmaps_cc_n_offset_5, src_burst_len, u8, 4, 7);
register_bits!(xdmaps_cc_n_offset_5, src_burst_size, u8, 1, 3);
register_bit!(xdmaps_cc_n_offset_5,  src_inc, 0);

register!(xdmaps_lc0_n_offset_5, XDmaPsLc0NOffset5, RW, u32);
register_bits!(xdmaps_lc0_n_offset_5, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_lc1_n_offset_5, XDmaPsLc1NOffset5, RW, u32);
register_bits!(xdmaps_lc1_n_offset_5, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_sa_n_offset_6, XDmaPsSaNOffset6, RW, u32);
register_bits!(xdmaps_sa_n_offset_6, src_addr, u8, 0, 31);

register!(xdmaps_da_n_offset_6, XDmaPsDaNOffset6, RW, u32);
register_bits!(xdmaps_da_n_offset_6, dest_addr, u8, 0, 31);

register!(xdmaps_cc_n_offset_6, XDmaPsCcNOffset6, RW, u32);
register_bits!(xdmaps_cc_n_offset_6, endian_swap_size, u8, 28, 30);
register_bits!(xdmaps_cc_n_offset_6, dst_cache_ctrl, u8, 25, 27);
register_bits!(xdmaps_cc_n_offset_6, dst_prot_ctrl, u8, 22, 24);
register_bits!(xdmaps_cc_n_offset_6, dst_burst_len, u8, 18, 21);
register_bits!(xdmaps_cc_n_offset_6, dst_burst_size, u8, 15, 17);
register_bit!(xdmaps_cc_n_offset_6,  dst_inc, 14);
register_bits!(xdmaps_cc_n_offset_6, src_cache_ctrl, u8, 11, 13);
register_bits!(xdmaps_cc_n_offset_6, src_prot_ctrl, u8, 8, 10);
register_bits!(xdmaps_cc_n_offset_6, src_burst_len, u8, 4, 7);
register_bits!(xdmaps_cc_n_offset_6, src_burst_size, u8, 1, 3);
register_bit!(xdmaps_cc_n_offset_6,  src_inc, 0);

register!(xdmaps_lc0_n_offset_6, XDmaPsLc0NOffset6, RW, u32);
register_bits!(xdmaps_lc0_n_offset_6, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_lc1_n_offset_6, XDmaPsLc1NOffset6, RW, u32);
register_bits!(xdmaps_lc1_n_offset_6, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_sa_n_offset_7, XDmaPsSaNOffset7, RW, u32);
register_bits!(xdmaps_sa_n_offset_7, src_addr, u8, 0, 31);

register!(xdmaps_da_n_offset_7, XDmaPsDaNOffset7, RW, u32);
register_bits!(xdmaps_da_n_offset_7, dest_addr, u8, 0, 31);

register!(xdmaps_cc_n_offset_7, XDmaPsCcNOffset7, RW, u32);
register_bits!(xdmaps_cc_n_offset_7, endian_swap_size, u8, 28, 30);
register_bits!(xdmaps_cc_n_offset_7, dst_cache_ctrl, u8, 25, 27);
register_bits!(xdmaps_cc_n_offset_7, dst_prot_ctrl, u8, 22, 24);
register_bits!(xdmaps_cc_n_offset_7, dst_burst_len, u8, 18, 21);
register_bits!(xdmaps_cc_n_offset_7, dst_burst_size, u8, 15, 17);
register_bit!(xdmaps_cc_n_offset_7,  dst_inc, 14);
register_bits!(xdmaps_cc_n_offset_7, src_cache_ctrl, u8, 11, 13);
register_bits!(xdmaps_cc_n_offset_7, src_prot_ctrl, u8, 8, 10);
register_bits!(xdmaps_cc_n_offset_7, src_burst_len, u8, 4, 7);
register_bits!(xdmaps_cc_n_offset_7, src_burst_size, u8, 1, 3);
register_bit!(xdmaps_cc_n_offset_7,  src_inc, 0);

register!(xdmaps_lc0_n_offset_7, XDmaPsLc0NOffset7, RW, u32);
register_bits!(xdmaps_lc0_n_offset_7, loop_counter_iteration, u8, 0, 7);

register!(xdmaps_lc1_n_offset_7, XDmaPsLc1NOffset7, RW, u32);
register_bits!(xdmaps_lc1_n_offset_7, loop_counter_iteration, u8, 0, 7);

register!(dbgstatus, DbgStatus, RW, u32);
register_bit!(dbgstatus, dbgstatus, 0);

register!(dbgcmd, DbgCmd, RW, u32);
register_bits!(dbgcmd, dbgcmd, u8, 0, 1);

register!(dbginst0, DbgInst0, RW, u32);
register_bits!(dbginst0, instruction_byte1, u8, 24, 31);
register_bits!(dbginst0, instruction_byte0, u8, 16, 23);
register_bits!(dbginst0, channel_num, u8, 8, 10);
register_bit!(dbginst0, debug_thread, 0);

register!(dbginst1, DbgInst1, RW, u32);
register_bits!(dbginst1, instruction_byte5, u8, 24, 31);
register_bits!(dbginst1, instruction_byte4, u8, 16, 23);
register_bits!(dbginst1, instruction_byte3, u8, 8, 10);
register_bits!(dbginst1, instruction_byte2, u8, 0, 7);

register!(cr0, Cr0, RW, u32);
register_bits!(cr0, num_events, u8, 17, 21);
register_bits!(cr0, num_periph_req, u8, 12, 16);
register_bits!(cr0, num_chnls, u8, 4, 6);
register_bit!(cr0,  mgr_ns_at_rst, 2);
register_bit!(cr0,  boot_en, 1);
register_bit!(cr0,  periph_req, 0);

register!(cr1, Cr1, RW, u32);
register_bits!(cr1, num_icache_lines, u8, 4, 7);
register_bits!(cr1, icache_len, u8, 0, 2);

register!(cr2, Cr2, RW, u32);
register_bits!(cr2, boot_addr, u8, 0, 31);

register!(cr3, Cr3, RW, u32);
register_bits!(cr3, ins, u8, 0, 31);

register!(cr4, Cr4, RW, u32);
register_bits!(cr4, ins, u8, 0, 31);

register!(crdn, Crdn, RW, u32);
register_bits!(crdn, data_buffer_dep, u8, 20, 29);
register_bits!(crdn, rd_q_dep, u8, 16, 19);
register_bits!(crdn, rd_cap, u8, 12, 14);
register_bits!(crdn, wr_q_dep, u8, 8, 11);
register_bits!(crdn, wr_cap, u8, 4, 6);
register_bits!(crdn, data_width, u8, 0, 2);

register!(wd, Wd, RW, u32);
register_bit!(wd, wd_irq_only, 0);

register!(periph_id_0, PeriphId0, RW, u32);
register_bits!(periph_id_0, part_number_0, u8, 0, 7);

register!(periph_id_1, PeriphId1, RW, u32);
register_bits!(periph_id_1, designer_0, u8, 4, 7);
register_bits!(periph_id_1, part_number_1, u8, 0, 3);

register!(periph_id_2, PeriphId2, RW, u32);
register_bits!(periph_id_2, revision, u8, 4, 7);
register_bits!(periph_id_2, designer_1, u8, 0, 3);

register!(periph_id_3, PeriphId3, RW, u32);
register_bit!(periph_id_3, integration_cfg, 0);

register!(pcell_id_0, PCellId0, RW, u32);
register_bits!(pcell_id_0, pcell_id_0, u8, 0, 7);

register!(pcell_id_1, PCellId1, RW, u32);
register_bits!(pcell_id_1, pcell_id_1, u8, 0, 7);

register!(pcell_id_2, PCellId2, RW, u32);
register_bits!(pcell_id_2, pcell_id_2, u8, 0, 7);

register!(pcell_id_3, PCellId3, RW, u32);
register_bits!(pcell_id_3, pcell_id_3, u8, 0, 7);
