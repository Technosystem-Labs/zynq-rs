///! Register definitions for Application Processing Unit (mpcore)

use volatile_register::{RO, RW};
use crate::{register, register_at, register_bit, register_bits,
            regs::RegisterW, regs::RegisterRW};

#[repr(C)]
pub struct RegisterBlock {
    pub scu_control: ScuControl,
    pub scu_config: RO<u32>,
    pub scu_cpu_power: RW<u32>,
    pub scu_invalidate: ScuInvalidate,
    reserved0: [u32; 12],
    pub filter_start: RW<u32>,
    pub filter_end: RW<u32>,
    reserved1: [u32; 2],
    pub scu_access_control: RW<u32>,
    pub scu_non_secure_access_control: RW<u32>,
    // there is plenty more (unimplemented)
}
register_at!(RegisterBlock, 0xF8F00000, new);

register!(scu_control, ScuControl, RW, u32);
register_bit!(scu_control, ic_standby_enable, 6);
register_bit!(scu_control, scu_standby_enable, 5);
register_bit!(scu_control, force_to_port0_enable, 4);
register_bit!(scu_control, scu_speculative_linefill_enable, 3);
register_bit!(scu_control, scu_rams_parity_enable, 2);
register_bit!(scu_control, address_filtering_enable, 1);
register_bit!(scu_control, enable, 0);

impl ScuControl {
    pub fn start(&mut self) {
        self.modify(|_, w| w.enable(true));
    }
}

register!(scu_invalidate, ScuInvalidate, WO, u32);
register_bits!(scu_invalidate, cpu0_ways, u8, 0, 3);
register_bits!(scu_invalidate, cpu1_ways, u8, 4, 7);
register_bits!(scu_invalidate, cpu2_ways, u8, 8, 11);
register_bits!(scu_invalidate, cpu3_ways, u8, 12, 15);

impl ScuInvalidate {
    pub fn invalidate_all_cores(&mut self) {
        self.write(ScuInvalidate::zeroed()
            .cpu0_ways(0xf)
            .cpu1_ways(0xf)
            .cpu2_ways(0xf)
            .cpu3_ways(0xf)
        );
    }

    pub fn invalidate_core1(&mut self) {
        self.write(ScuInvalidate::zeroed()
            .cpu1_ways(0xf)
        );
    }
}
