use libregister::{register, register_at, register_bit, register_bits, RegisterRW, RegisterR, RegisterW};
use super::asm::dmb;
use volatile_register::RW;

/// enable L2 cache with specific prefetch offset
/// prefetch offset requires manual tuning, it seems that 8 is good for ZC706 current settings
pub fn enable_l2_cache(offset: u8) {
    dmb();
    let regs = RegisterBlock::new();
    // disable L2 cache
    regs.reg1_control.modify(|_, w| w.l2_enable(false));

    regs.reg15_prefetch_ctrl.modify(|_, w|
        w.instr_prefetch_en(true)
            .data_prefetch_en(true)
            .double_linefill_en(true)
            .incr_double_linefill_en(true)
            .pref_drop_en(true)
            .prefetch_offset(offset)
    );
    regs.reg1_aux_control.modify(|_, w| {
        w.early_bresp_en(true)
            .instr_prefetch_en(true)
            .data_prefetch_en(true)
            .cache_replace_policy(true)
            .way_size(3)
    });
    regs.reg1_tag_ram_control.modify(|_, w| w.ram_wr_access_lat(1).ram_rd_access_lat(1).ram_setup_lat(1));
    regs.reg1_data_ram_control.modify(|_, w| w.ram_wr_access_lat(1).ram_rd_access_lat(2).ram_setup_lat(1));
    // invalidate L2 ways
    unsafe {
        regs.reg7_inv_way.write(0xFFFF);
    }
    // poll for completion
    while regs.reg7_cache_sync.read().c() {}
    // write to a magic memory location with a magic sequence
    // required in UG585 Section 3.4.10 Initialization Sequence
    unsafe {
        core::ptr::write_volatile(0xF8000008usize as *mut u32, 0xDF0D);
        core::ptr::write_volatile(0xF8000A1Cusize as *mut u32, 0x020202);
        core::ptr::write_volatile(0xF8000004usize as *mut u32, 0x767B);
    }
    regs.reg1_control.modify(|_, w| w.l2_enable(true));
    dmb();
}

#[inline(always)]
pub fn l2_cache_invalidate_all()  {
    let regs = RegisterBlock::new();
    unsafe {
        regs.reg7_inv_way.write(0xFFFF);
    }
    // poll for completion
    while regs.reg7_cache_sync.read().c() {}
}

#[inline(always)]
pub fn l2_cache_clean_all()  {
    let regs = RegisterBlock::new();
    unsafe  {
        regs.reg7_clean_way.write(0xFFFF);
    }
    // poll for completion
    while regs.reg7_cache_sync.read().c() {}
}

#[inline(always)]
pub fn l2_cache_clean_invalidate_all()  {
    let regs = RegisterBlock::new();
    unsafe {
        regs.reg7_clean_inv_way.write(0xFFFF);
    }
    // poll for completion
    while regs.reg7_cache_sync.read().c() {}
}

/// L2 cache sync, similar to dsb for L1 cache
#[inline(always)]
pub fn l2_cache_sync() {
    let regs = RegisterBlock::new();
    regs.reg7_cache_sync.write(Reg7CacheSync::zeroed().c(false));
}

#[inline(always)]
pub fn l2_cache_clean(addr: usize) {
    let regs = RegisterBlock::new();
    unsafe {
        regs.reg7_clean_pa.write(addr as u32);
    }
}

#[inline(always)]
pub fn l2_cache_invalidate(addr: usize) {
    let regs = RegisterBlock::new();
    unsafe {
        regs.reg7_inv_pa.write(addr as u32);
    }
}

#[inline(always)]
pub fn l2_cache_clean_invalidate(addr: usize) {
    let regs = RegisterBlock::new();
    unsafe {
        regs.reg7_clean_inv_pa.write(addr as u32);
    }
}

#[repr(C)]
struct RegisterBlock {
    /// cache ID register, Returns the 32-bit device ID code it reads off the CACHEID input bus.
    /// The value is specified by the system integrator. Reset value: 0x410000c8
    pub reg0_cache_id: Reg0CacheId,
    /// cache type register, Returns the 32-bit cache type. Reset value: 0x1c100100
    pub reg0_cache_type: Reg0CacheType,
    unused0: [u32; 62],
    /// control register, reset value: 0x0
    pub reg1_control: Reg1Control,
    /// auxilary control register, reset value: 0x02020000
    pub reg1_aux_control: Reg1AuxControl,
    /// Configures Tag RAM latencies
    pub reg1_tag_ram_control: Reg1TagRamControl,
    /// configures data RAM latencies
    pub reg1_data_ram_control: Reg1DataRamControl,
    unused1: [u32; 60],
    /// Permits the event counters to be enabled and reset.
    pub reg2_ev_counter_ctrl: Reg2EvCounterCtrl,
    /// Enables event counter 1 to be driven by a specific event. Counter 1 increments when the
    /// event occurs.
    pub reg2_ev_counter1_cfg: Reg2EvCounter1Cfg,
    /// Enables event counter 0 to be driven by a specific event. Counter 0 increments when the
    /// event occurs.
    pub reg2_ev_counter0_cfg: Reg2EvCounter0Cfg,
    /// Enable the programmer to read off the counter value. The counter counts an event as
    /// specified by the Counter Configuration Registers. The counter can be preloaded if counting
    /// is disabled and reset by the Event Counter Control Register.
    pub reg2_ev_counter1: RW<u32>,
    /// Enable the programmer to read off the counter value. The counter counts an event as
    /// specified by the Counter Configuration Registers. The counter can be preloaded if counting
    /// is disabled and reset by the Event Counter Control Register.
    pub reg2_ev_counter0: RW<u32>,
    /// This register enables or masks interrupts from being triggered on the external pins of the
    /// cache controller. Figure 3-8 on page 3-17 shows the register bit assignments. The bit
    /// assignments enables the masking of the interrupts on both their individual outputs and the
    /// combined L2CCINTR line. Clearing a bit by writing a 0, disables the interrupt triggering on
    /// that pin. All bits are cleared by a reset. You must write to the register bits with a 1 to
    /// enable the generation of interrupts. 1 = Enabled. 0 = Masked. This is the default.
    pub reg2_int_mask: Reg2IntMask,
    /// This register is a read-only.It returns the masked interrupt status. This register can be
    /// accessed by secure and non-secure operations. The register gives an AND function of the raw
    /// interrupt status with the values of the interrupt mask register. All the bits are cleared
    /// by a reset. A write to this register is ignored. Bits read can be HIGH or LOW: HIGH If the
    /// bits read HIGH, they reflect the status of the input lines triggering an interrupt. LOW If
    /// the bits read LOW, either no interrupt has been generated, or the interrupt is masked.
    pub reg2_int_mask_status: Reg2IntMaskStatus,
    /// The Raw Interrupt Status Register enables the interrupt status that excludes the masking
    /// logic. Bits read can be HIGH or LOW: HIGH If the bits read HIGH, they reflect the status of
    /// the input lines triggering an interrupt. LOW If the bits read LOW, no interrupt has been
    /// generated.
    pub reg2_int_raw_status: Reg2IntRawStatus,
    /// Clears the Raw Interrupt Status Register bits. When a bit is written as 1, it clears the
    /// corresponding bit in the Raw Interrupt Status Register. When a bit is written as 0, it has
    /// no effect
    pub reg2_int_clear: Reg2IntClear,
    unused2: [u32; 323],
    /// Drain the STB. Operation complete when all buffers, LRB, LFB, STB, and EB, are empty
    pub reg7_cache_sync: Reg7CacheSync,
    unused3: [u32; 15],
    /// Invalidate Line by PA: Specific L2 cache line is marked as not valid
    pub reg7_inv_pa: RW<u32>,
    unused4: [u32; 2],
    /// Invalidate by Way Invalidate all data in specified ways, including dirty data. An
    /// Invalidate by way while selecting all cache ways is equivalent to invalidating all cache
    /// entries. Completes as a background task with the way, or ways, locked, preventing
    /// allocation.
    pub reg7_inv_way: RW<u32>,
    unused5: [u32; 12],
    /// Clean Line by PA Write the specific L2 cache line to L3 main memory if the line is marked
    /// as valid and dirty. The line is marked as not dirty. The valid bit is unchanged
    pub reg7_clean_pa: RW<u32>,
    unused6: [u32; 1],
    /// Clean Line by Set/Way Write the specific L2 cache line within the specified way to L3 main
    /// memory if the line is marked as valid and dirty. The line is marked as not dirty. The valid
    /// bit is unchanged
    pub reg7_clean_index: Reg7CleanIndex,
    /// Clean by Way Writes each line of the specified L2 cache ways to L3 main memory if the line
    /// is marked as valid and dirty. The lines are marked as not dirty. The valid bits are
    /// unchanged. Completes as a background task with the way, or ways, locked, preventing
    /// allocation.
    pub reg7_clean_way: RW<u32>,
    unused7: [u32; 12],
    /// Clean and Invalidate Line by PA Write the specific L2 cache line to L3 main memory if the
    /// line is marked as valid and dirty. The line is marked as not valid
    pub reg7_clean_inv_pa: RW<u32>,
    unused8: [u32; 1],
    /// Clean and Invalidate Line by Set/Way Write the specific L2 cache line within the specified
    /// way to L3 main memory if the line is marked as valid and dirty. The line is marked as not
    /// valid
    pub reg7_clean_inv_index: Reg7CleanInvIndex,
    /// Clean and Invalidate by Way Writes each line of the specified L2 cache ways to L3 main
    /// memory if the line is marked as valid and dirty. The lines are marked as not valid.
    /// Completes as a background task with the way, or ways, locked, preventing allocation.
    pub reg7_clean_inv_way: RW<u32>,
    unused9: [u32; 0x1D8],
    pub reg15_prefetch_ctrl: Reg15PrefetechCtrl,
}

register_at!(RegisterBlock, 0xF8F02000, new);

register!(reg0_cache_id, Reg0CacheId, RW, u32);
register_bits!(reg0_cache_id, implementer, u8, 24, 31);
register_bits!(reg0_cache_id, cache_id, u8, 10, 15);
register_bits!(reg0_cache_id, part_num, u8, 6, 9);
register_bits!(reg0_cache_id, rtl_release, u8, 0, 5);

register!(reg0_cache_type, Reg0CacheType, RW, u32);
register_bit!(reg0_cache_type, data_banking, 31);
register_bits!(reg0_cache_type, ctype, u8, 25, 28);
register_bit!(reg0_cache_type, h, 24);
register_bits!(reg0_cache_type, dsize_middsize_19, u8, 20, 22);
register_bit!(reg0_cache_type, l2_assoc_d, 18);
register_bits!(reg0_cache_type, l2cache_line_len_disize_11, u8, 12, 13);
register_bits!(reg0_cache_type, isize_midisize_7, u8, 8, 10);
register_bit!(reg0_cache_type, l2_assoc_i, 6);
register_bits!(reg0_cache_type, l2cache_line_len_i, u8, 0, 1);

register!(reg1_control, Reg1Control, RW, u32);
register_bit!(reg1_control, l2_enable, 0);

register!(reg1_aux_control, Reg1AuxControl, RW, u32);
register_bit!(reg1_aux_control, early_bresp_en, 30);
register_bit!(reg1_aux_control, instr_prefetch_en, 29);
register_bit!(reg1_aux_control, data_prefetch_en, 28);
register_bit!(reg1_aux_control, nonsec_inte_access_ctrl, 27);
register_bit!(reg1_aux_control, nonsec_lockdown_en, 26);
register_bit!(reg1_aux_control, cache_replace_policy, 25);
register_bits!(reg1_aux_control, force_write_alloc, u8, 23, 24);
register_bit!(reg1_aux_control, shared_attr_override_en, 22);
register_bit!(reg1_aux_control, parity_en, 21);
register_bit!(reg1_aux_control, event_mon_bus_en, 20);
register_bits!(reg1_aux_control, way_size, u8, 17, 19);
register_bit!(reg1_aux_control, associativity, 16);
register_bit!(reg1_aux_control, shared_attr_inva_en, 13);
register_bit!(reg1_aux_control, ex_cache_config, 12);
register_bit!(reg1_aux_control, store_buff_dev_lim_en, 11);
register_bit!(reg1_aux_control, high_pr_so_dev_rd_en, 10);
register_bit!(reg1_aux_control, full_line_zero_enable, 0);

register!(reg1_tag_ram_control, Reg1TagRamControl, RW, u32);
register_bits!(reg1_tag_ram_control, ram_wr_access_lat, u8, 8, 10);
register_bits!(reg1_tag_ram_control, ram_rd_access_lat, u8, 4, 6);
register_bits!(reg1_tag_ram_control, ram_setup_lat, u8, 0, 2);

register!(reg1_data_ram_control, Reg1DataRamControl, RW, u32);
register_bits!(reg1_data_ram_control, ram_wr_access_lat, u8, 8, 10);
register_bits!(reg1_data_ram_control, ram_rd_access_lat, u8, 4, 6);
register_bits!(reg1_data_ram_control, ram_setup_lat, u8, 0, 2);

register!(reg2_ev_counter_ctrl, Reg2EvCounterCtrl, RW, u32);
register_bit!(reg2_ev_counter_ctrl, ev_ctr_en, 0);

register!(reg2_ev_counter1_cfg, Reg2EvCounter1Cfg, RW, u32);
register_bits!(reg2_ev_counter1_cfg, ctr_ev_src, u8, 2, 5);
register_bits!(reg2_ev_counter1_cfg, ev_ctr_intr_gen, u8, 0, 1);

register!(reg2_ev_counter0_cfg, Reg2EvCounter0Cfg, RW, u32);
register_bits!(reg2_ev_counter0_cfg, ctr_ev_src, u8, 2, 5);
register_bits!(reg2_ev_counter0_cfg, ev_ctr_intr_gen, u8, 0, 1);

register!(reg2_int_mask, Reg2IntMask, RW, u32);
register_bit!(reg2_int_mask, decerr, 8);
register_bit!(reg2_int_mask, slverr, 7);
register_bit!(reg2_int_mask, errrd, 6);
register_bit!(reg2_int_mask, errrt, 5);
register_bit!(reg2_int_mask, errwd, 4);
register_bit!(reg2_int_mask, errwt, 3);
register_bit!(reg2_int_mask, parrd, 2);
register_bit!(reg2_int_mask, parrt, 1);
register_bit!(reg2_int_mask, ecntr, 0);

register!(reg2_int_mask_status, Reg2IntMaskStatus, RW, u32);
register_bit!(reg2_int_mask_status, decerr, 8);
register_bit!(reg2_int_mask_status, slverr, 7);
register_bit!(reg2_int_mask_status, errrd, 6);
register_bit!(reg2_int_mask_status, errrt, 5);
register_bit!(reg2_int_mask_status, errwd, 4);
register_bit!(reg2_int_mask_status, errwt, 3);
register_bit!(reg2_int_mask_status, parrd, 2);
register_bit!(reg2_int_mask_status, parrt, 1);
register_bit!(reg2_int_mask_status, ecntr, 0);

register!(reg2_int_raw_status, Reg2IntRawStatus, RW, u32);
register_bit!(reg2_int_raw_status, decerr, 8);
register_bit!(reg2_int_raw_status, slverr, 7);
register_bit!(reg2_int_raw_status, errrd, 6);
register_bit!(reg2_int_raw_status, errrt, 5);
register_bit!(reg2_int_raw_status, errwd, 4);
register_bit!(reg2_int_raw_status, errwt, 3);
register_bit!(reg2_int_raw_status, parrd, 2);
register_bit!(reg2_int_raw_status, parrt, 1);
register_bit!(reg2_int_raw_status, ecntr, 0);

register!(reg2_int_clear, Reg2IntClear, RW, u32, 0);
register_bit!(reg2_int_clear, decerr, 8, WTC);
register_bit!(reg2_int_clear, slverr, 7, WTC);
register_bit!(reg2_int_clear, errrd, 6, WTC);
register_bit!(reg2_int_clear, errrt, 5, WTC);
register_bit!(reg2_int_clear, errwd, 4, WTC);
register_bit!(reg2_int_clear, errwt, 3, WTC);
register_bit!(reg2_int_clear, parrd, 2, WTC);
register_bit!(reg2_int_clear, parrt, 1, WTC);
register_bit!(reg2_int_clear, ecntr, 0, WTC);

register!(reg7_cache_sync, Reg7CacheSync, RW, u32);
register_bit!(reg7_cache_sync, c, 0);

register!(reg7_clean_index, Reg7CleanIndex, RW, u32);
register_bits!(reg7_clean_index, way, u8, 28, 30);
register_bits!(reg7_clean_index, index, u8, 5, 11);
register_bit!(reg7_clean_index, c, 0);

register!(reg7_clean_inv_index, Reg7CleanInvIndex, RW, u32);
register_bits!(reg7_clean_inv_index, way, u8, 28, 30);
register_bits!(reg7_clean_inv_index, index, u8, 5, 11);
register_bit!(reg7_clean_inv_index, c, 0);

register!(reg15_prefetch_ctrl, Reg15PrefetechCtrl, RW, u32);
register_bit!(reg15_prefetch_ctrl, double_linefill_en, 30);
register_bit!(reg15_prefetch_ctrl, instr_prefetch_en, 29);
register_bit!(reg15_prefetch_ctrl, data_prefetch_en, 28);
register_bit!(reg15_prefetch_ctrl, pref_drop_en, 24);
register_bit!(reg15_prefetch_ctrl, incr_double_linefill_en, 23);
register_bits!(reg15_prefetch_ctrl, prefetch_offset, u8, 0, 4);

