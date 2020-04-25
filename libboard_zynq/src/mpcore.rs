///! Register definitions for Application Processing Unit (mpcore)

use volatile_register::{RO, RW};
use libregister::{
    register, register_at, register_bit, register_bits,
    RegisterW, RegisterRW,
};

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
    reserved2: [u32; 42],
    pub iccicr: RW<u32>,
    pub iccpmw: RW<u32>,
    pub iccbpr: RW<u32>,
    pub icciar: RW<u32>,
    pub icceoir: RW<u32>,
    pub iccrpr: RW<u32>,
    pub icchpir: RW<u32>,
    pub iccabpr: RW<u32>,
    reserved3: [u32; 55],
    pub iccidr: RW<u32>,
    pub global_timer_counter0: ValueRegister,
    pub global_timer_counter1: ValueRegister,
    pub global_timer_control: GlobalTimerControl,
    pub global_timer_interrupt_status: RW<u32>,
    pub comparator_value0: ValueRegister,
    pub comparator_value1: ValueRegister,
    pub auto_increment: ValueRegister,
    reserved4: [u32; 249],
    pub private_timer_load: ValueRegister,
    pub private_timer_counter: ValueRegister,
    pub private_timer_control: RW<u32>,
    pub private_timer_interrupt_status: RW<u32>,
    reserved5: [u32; 4],
    pub watchdog_load: ValueRegister,
    pub watchdog_counter: ValueRegister,
    pub watchdog_control: RW<u32>,
    pub watchdog_interrupt_status: RW<u32>,
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

register!(value_register, ValueRegister, RW, u32);
register_bits!(value_register, value, u32, 0, 31);

register!(global_timer_control, GlobalTimerControl, RW, u32);
register_bits!(global_timer_control, prescaler, u8, 8, 15);
register_bit!(global_timer_control, auto_increment_mode, 3);
register_bit!(global_timer_control, irq_enable, 2);
register_bit!(global_timer_control, comp_enablea, 1);
register_bit!(global_timer_control, timer_enable, 0);
