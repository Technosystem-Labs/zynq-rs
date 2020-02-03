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
    pub ftc: [Ftc; 8],
    pub cs0: Cs,
    pub cpc0: Cpc,
    pub cs1: Cs,
    pub cpc1: Cpc,
    pub cs2: Cs,
    pub cpc2: Cpc,
    pub cs3: Cs,
    pub cpc3: Cpc,
    pub cs4: Cs,
    pub cpc4: Cpc,
    pub cs5: Cs,
    pub cpc5: Cpc,
    pub cs6: Cs,
    pub cpc6: Cpc,
    pub cs7: Cs,
    pub cpc7: Cpc,
    pub sa0: Sa,
    pub da0: Da,
    pub cc0: Cc,
    pub lc0_0: Lc,
    pub lc0_1: Lc,
    pub sa1: Sa,
    pub da1: Da,
    pub cc1: Cc,
    pub lc1_0: Lc,
    pub lc1_1: Lc,
    pub sa2: Sa,
    pub da2: Da,
    pub cc2: Cc,
    pub lc2_0: Lc,
    pub lc2_1: Lc,
    pub sa3: Sa,
    pub da3: Da,
    pub cc3: Cc,
    pub lc3_0: Lc,
    pub lc3_1: Lc,
    pub sa4: Sa,
    pub da4: Da,
    pub cc4: Cc,
    pub lc4_0: Lc,
    pub lc4_1: Lc,
    pub sa5: Sa,
    pub da5: Da,
    pub cc5: Cc,
    pub lc5_0: Lc,
    pub lc5_1: Lc,
    pub sa6: Sa,
    pub da6: Da,
    pub cc6: Cc,
    pub lc6_0: Lc,
    pub lc6_1: Lc,
    pub sa7: Sa,
    pub da7: Da,
    pub cc7: Cc,
    pub lc7_0: Lc,
    pub lc7_1: Lc,
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

register!(ftc, Ftc, RW, u32);
register_bit!(ftc, lockup_err, 31);
register_bit!(ftc, dbg_instr, 30);
register_bit!(ftc, data_read_err, 18);
register_bit!(ftc, data_write_err, 17);
register_bit!(ftc, instr_fetch_err, 16);
register_bit!(ftc, st_data_unavailable, 13);
register_bit!(ftc, mfifo_err, 12);
register_bit!(ftc, ch_rdwr_err, 7);
register_bit!(ftc, ch_periph_err, 6);
register_bit!(ftc, ch_evnt_err, 5);
register_bit!(ftc, operand_invalid, 1);
register_bit!(ftc, undef_instr, 0);

register!(cs, Cs, RW, u32);
register_bit!(cs, cns, 21);
register_bit!(cs, dmawfp_periph, 15);
register_bit!(cs, dmawfp_b_ns, 14);
register_bits!(cs, wakeup_num, u8, 4, 8);
register_bits!(cs, channel_status, u8, 0, 3);

register!(cpc, Cpc, RW, u32);
register_bits!(cpc, pc_chnl, u8, 0, 31);

register!(sa, Sa, RW, u32);
register_bits!(sa, src_addr, u8, 0, 31);

register!(da, Da, RW, u32);
register_bits!(da, dest_addr, u8, 0, 31);

register!(cc, Cc, RW, u32);
register_bits!(cc, endian_swap_size, u8, 28, 30);
register_bits!(cc, dst_cache_ctrl, u8, 25, 27);
register_bits!(cc, dst_prot_ctrl, u8, 22, 24);
register_bits!(cc, dst_burst_len, u8, 18, 21);
register_bits!(cc, dst_burst_size, u8, 15, 17);
register_bit!(cc, dst_inc, 14);
register_bits!(cc, src_cache_ctrl, u8, 11, 13);
register_bits!(cc, src_prot_ctrl, u8, 8, 10);
register_bits!(cc, src_burst_len, u8, 4, 7);
register_bits!(cc, src_burst_size, u8, 1, 3);
register_bit!(cc, src_inc, 0);

register!(lc0, Lc, RW, u32);
register_bits!(lc0, loop_counter_iteration, u8, 0, 7);

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
