///! Register definitions for Application Processing Unit (mpcore)

use volatile_register::{RO, RW};
use libregister::{
    register, register_at, register_bit, register_bits,
    RegisterW, RegisterRW,
};

#[repr(C)]
pub struct RegisterBlock {
    /// SCU Control Register
    pub scu_control: ScuControl,
    /// SCU Configuration Register
    pub scu_config: ScuConfig,
    /// SCU CPU Power Status Register
    pub scu_cpu_power_status: SCUCPUPowerStatusRegister,
    /// SCU Invalidate All Registers in Secure State
    pub scu_invalidate: ScuInvalidate,
    unused0: [u32; 12],
    /// Filtering Start Address Register
    pub filtering_start_address: FilteringStartAddressRegister,
    /// Defined by FILTEREND input
    pub filtering_end_address: FilteringEndAddressRegister,
    unused1: [u32; 2],
    /// SCU Access Control (SAC) Register
    pub scu_access_control_sac: SCUAccessControlRegisterSAC,
    /// SCU Non-secure Access Control Register SNSAC
    pub scu_non_secure_access_control: SCUNonSecureAccessControlRegister,
    unused2: [u32; 42],
    /// CPU Interface Control Register
    pub iccicr: ICCICR,
    /// Interrupt Priority Mask Register
    pub iccpmr: ICCPMR,
    /// Binary Point Register
    pub iccbpr: ICCBPR,
    /// Interrupt Acknowledge Register
    pub icciar: ICCIAR,
    /// End Of Interrupt Register
    pub icceoir: ICCEOIR,
    /// Running Priority Register
    pub iccrpr: ICCRPR,
    /// Highest Pending Interrupt Register
    pub icchpir: ICCHPIR,
    /// Aliased Non-secure Binary Point Register
    pub iccabpr: ICCABPR,
    unused3: [u32; 55],
    /// CPU Interface Implementer Identification Register
    pub iccidr: ICCIDR,
    /// Global Timer Counter Register 0
    pub global_timer_counter0: ValueRegister,
    pub global_timer_counter1: ValueRegister,
    /// Global Timer Control Register
    pub global_timer_control: GlobalTimerControl,
    /// Global Timer Interrupt Status Register
    pub global_timer_interrupt_status: GlobalTimerInterruptStatusRegister,
    /// Comparator Value Register_0
    pub comparator_value0: ValueRegister,
    pub comparator_value1: ValueRegister,
    /// Auto-increment Register
    pub auto_increment: RW<u32>,
    unused4: [u32; 249],
    /// Private Timer Load Register
    pub private_timer_load: RW<u32>,
    /// Private Timer Counter Register
    pub private_timer_counter: RW<u32>,
    /// Private Timer Control Register
    pub private_timer_control: PrivateTimerControlRegister,
    /// Private Timer Interrupt Status Register
    pub private_timer_interrupt_status: PrivateTimerInterruptStatusRegister,
    unused5: [u32; 4],
    /// Watchdog Load Register
    pub watchdog_load: RW<u32>,
    /// Watchdog Counter Register
    pub watchdog_counter: RW<u32>,
    /// Watchdog Control Register
    pub watchdog_control: WatchdogControlRegister,
    /// Watchdog Interrupt Status Register
    pub watchdog_interrupt_status: WatchdogInterruptStatusRegister,
    /// Watchdog Reset Status Register
    pub watchdog_reset_status: WatchdogResetStatusRegister,
    /// Watchdog Disable Register
    pub watchdog_disable: RW<u32>,
    unused6: [u32; 626],
    /// Distributor Control Register
    pub icddcr: ICDDCR,
    /// Interrupt Controller Type Register
    pub icdictr: ICDICTR,
    /// Distributor Implementer Identification Register
    pub icdiidr: ICDIIDR,
    unused7: [u32; 29],
    /// Interrupt Security Register
    pub icdisr0: RW<u32>,
    pub icdisr1: RW<u32>,
    pub icdisr2: RW<u32>,
    unused8: [u32; 29],
    /// Interrupt Set-enable Registers
    pub icdiser: [RW<u32>; 3],
    unused9: [u32; 29],
    /// Interrupt Clear-Enable Register 0
    pub icdicer0: RW<u32>,
    /// Interrupt Clear-Enable Register 1
    pub icdicer1: RW<u32>,
    /// Interrupt Clear-Enable Register 2
    pub icdicer2: RW<u32>,
    unused10: [u32; 29],
    /// Interrupt Set-pending Register
    pub icdispr0: RW<u32>,
    pub icdispr1: RW<u32>,
    pub icdispr2: RW<u32>,
    unused11: [u32; 29],
    /// Interrupt Clear-Pending Register
    pub icdicpr0: RW<u32>,
    pub icdicpr1: RW<u32>,
    pub icdicpr2: RW<u32>,
    unused12: [u32; 29],
    /// Active Bit register
    pub icdabr0: RW<u32>,
    pub icdabr1: RW<u32>,
    pub icdabr2: RW<u32>,
    unused13: [u32; 61],
    /// Interrupt Priority Register
    pub icdipr0: RW<u32>,
    pub icdipr1: RW<u32>,
    pub icdipr2: RW<u32>,
    pub icdipr3: RW<u32>,
    pub icdipr4: RW<u32>,
    pub icdipr5: RW<u32>,
    pub icdipr6: RW<u32>,
    pub icdipr7: RW<u32>,
    pub icdipr8: RW<u32>,
    pub icdipr9: RW<u32>,
    pub icdipr10: RW<u32>,
    pub icdipr11: RW<u32>,
    pub icdipr12: RW<u32>,
    pub icdipr13: RW<u32>,
    pub icdipr14: RW<u32>,
    pub icdipr15: RW<u32>,
    pub icdipr16: RW<u32>,
    pub icdipr17: RW<u32>,
    pub icdipr18: RW<u32>,
    pub icdipr19: RW<u32>,
    pub icdipr20: RW<u32>,
    pub icdipr21: RW<u32>,
    pub icdipr22: RW<u32>,
    pub icdipr23: RW<u32>,
    unused14: [u32; 232],
    /// Interrupt Processor Targets Registers
    pub icdiptr: [RW<u32>; 24],
    unused15: [u32; 232],
    /// Interrupt Configuration Registers
    pub icdicfr: [RW<u32>; 6],
    unused16: [u32; 58],
    /// PPI Status Register
    pub ppi_status: PpiStatus,
    /// SPI Status Register 0
    pub spi_status_0: RO<u32>,
    /// SPI Status Register 1
    pub spi_status_1: RO<u32>,
    unused17: [u32; 125],
    /// Software Generated Interrupt Register
    pub icdsgir: ICDSGIR,
}

register_at!(RegisterBlock, 0xF8F00000, new);

register!(value_register, ValueRegister, RW, u32);
register_bits!(value_register, value, u32, 0, 31);

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

register!(scu_config, ScuConfig, RO, u32);
register_bits!(scu_config, tag_ram_sizes, u8, 8, 15);
register_bits!(scu_config, cpus_smp, u8, 4, 7);
register_bits!(scu_config, cpu_number, u8, 0, 1);

register!(scu_cpu_power_status, SCUCPUPowerStatusRegister, RW, u32);
register_bits!(scu_cpu_power_status, cpu3_status, u8, 24, 25);
register_bits!(scu_cpu_power_status, cpu2_status, u8, 16, 17);
register_bits!(scu_cpu_power_status, cpu1_status, u8, 8, 9);
register_bits!(scu_cpu_power_status, cpu0_status, u8, 0, 1);

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

register!(filtering_start_address, FilteringStartAddressRegister, RW, u32);
register_bits!(filtering_start_address, filtering_start_address, u32, 20, 31);
register_bits!(filtering_start_address, sbz, u32, 0, 19);

register!(filtering_end_address, FilteringEndAddressRegister, RW, u32);
register_bits!(filtering_end_address, filtering_end_address, u32, 20, 31);
register_bits!(filtering_end_address, sbz, u32, 0, 19);

register!(scu_access_control_sac, SCUAccessControlRegisterSAC, RW, u32);
register_bit!(scu_access_control_sac, cp_u3, 3);
register_bit!(scu_access_control_sac, cp_u2, 2);
register_bit!(scu_access_control_sac, cp_u1, 1);
register_bit!(scu_access_control_sac, cp_u0, 0);

register!(scu_non_secure_access_control, SCUNonSecureAccessControlRegister, RO, u32);
register_bits!(scu_non_secure_access_control, sbz, u32, 12, 31);
register_bit!(scu_non_secure_access_control, cpu3_global_timer, 11);
register_bit!(scu_non_secure_access_control, cpu2_global_timer, 10);
register_bit!(scu_non_secure_access_control, cpu1_global_timer, 9);
register_bit!(scu_non_secure_access_control, cpu0_global_timer, 8);
register_bit!(scu_non_secure_access_control, private_timers_for_cpu3, 7);
register_bit!(scu_non_secure_access_control, private_timers_for_cpu2, 6);
register_bit!(scu_non_secure_access_control, private_timers_for_cpu1, 5);
register_bit!(scu_non_secure_access_control, private_timers_for_cpu0, 4);
register_bit!(scu_non_secure_access_control, component_access_for_cpu3, 3);
register_bit!(scu_non_secure_access_control, component_access_for_cpu2, 2);
register_bit!(scu_non_secure_access_control, component_access_for_cpu1, 1);
register_bit!(scu_non_secure_access_control, component_access_for_cpu0, 0);

register!(iccicr, ICCICR, RW, u32);
register_bit!(iccicr, sbpr, 4);
register_bit!(iccicr, fiq_en, 3);
register_bit!(iccicr, ack_ctl, 2);
register_bit!(iccicr, enable_ns, 1);
register_bit!(iccicr, enable_s, 0);

register!(iccpmr, ICCPMR, RW, u32);
register_bits!(iccpmr, priority, u8, 0, 7);

register!(iccbpr, ICCBPR, RW, u32);
register_bits!(iccbpr, binary_point, u8, 0, 2);

register!(icciar, ICCIAR, RW, u32);
register_bits!(icciar, cpuid, u8, 10, 12);
register_bits!(icciar, ackintid, u32, 0, 9);

register!(icceoir, ICCEOIR, RW, u32);
register_bits!(icceoir, cpuid, u8, 10, 12);
register_bits!(icceoir, eoiintid, u32, 0, 9);

register!(iccrpr, ICCRPR, RW, u32);
register_bits!(iccrpr, priority, u8, 0, 7);

register!(icchpir, ICCHPIR, RW, u32);
register_bits!(icchpir, cpuid, u8, 10, 12);
register_bits!(icchpir, pendintid, u32, 0, 9);

register!(iccabpr, ICCABPR, RW, u32);
register_bits!(iccabpr, binary_point, u8, 0, 2);

register!(iccidr, ICCIDR, RO, u32);
register_bits!(iccidr, part_number, u32, 20, 31);
register_bits!(iccidr, architecture_version, u8, 16, 19);
register_bits!(iccidr, revision_number, u8, 12, 15);
register_bits!(iccidr, implementer, u32, 0, 11);

register!(global_timer_control, GlobalTimerControl, RW, u32);
register_bits!(global_timer_control, prescaler, u8, 8, 15);
register_bit!(global_timer_control, auto_increment_mode, 3);
register_bit!(global_timer_control, irq_enable, 2);
register_bit!(global_timer_control, comp_enablea, 1);
register_bit!(global_timer_control, timer_enable, 0);

register!(global_timer_interrupt_status, GlobalTimerInterruptStatusRegister, RW, u32);
register_bit!(global_timer_interrupt_status, event_flag, 0);

register!(private_timer_control, PrivateTimerControlRegister, RW, u32);
register_bits!(private_timer_control, sbzp, u32, 16, 31);
register_bits!(private_timer_control, prescaler, u8, 8, 15);
register_bits!(private_timer_control, unk_sbzp, u8, 3, 7);
register_bit!(private_timer_control, irq_enable, 2);
register_bit!(private_timer_control, auto_reload, 1);
register_bit!(private_timer_control, timer_enable, 0);

register!(private_timer_interrupt_status, PrivateTimerInterruptStatusRegister, RW, u32);
register_bits!(private_timer_interrupt_status, unk_sbzp, u32, 1, 31);

register!(watchdog_control, WatchdogControlRegister, RW, u32);
register_bits!(watchdog_control, prescaler, u8, 8, 15);
register_bit!(watchdog_control, watchdog_mode, 3);
register_bit!(watchdog_control, it_enable, 2);
register_bit!(watchdog_control, auto_reload, 1);
register_bit!(watchdog_control, watchdog_enable, 0);

register!(watchdog_interrupt_status, WatchdogInterruptStatusRegister, RW, u32);
register_bit!(watchdog_interrupt_status, event_flag, 0);

register!(watchdog_reset_status, WatchdogResetStatusRegister, RW, u32);
register_bit!(watchdog_reset_status, reset_flag, 0);

register!(icddcr, ICDDCR, RW, u32);
register_bit!(icddcr, enable_non_secure, 1);
register_bit!(icddcr, enable_secure, 0);

register!(icdictr, ICDICTR, RO, u32);
register_bits!(icdictr, lspi, u8, 11, 15);
register_bit!(icdictr, security_extn, 10);
register_bits!(icdictr, sbz, u8, 8, 9);
register_bits!(icdictr, cpu_number, u8, 5, 7);
register_bits!(icdictr, it_lines_number, u8, 0, 4);

register!(icdiidr, ICDIIDR, RO, u32);
register_bits!(icdiidr, implementation_version, u8, 24, 31);
register_bits!(icdiidr, revision_number, u32, 12, 23);
register_bits!(icdiidr, implementer, u32, 0, 11);

register!(ppi_status, PpiStatus, RO, u32);
register_bits!(ppi_status, ppi_status, u8, 11, 15);
register_bits!(ppi_status, sbz, u32, 0, 10);

register!(icdsgir, ICDSGIR, RW, u32);
register_bits!(icdsgir, target_list_filter, u8, 24, 25);
register_bits!(icdsgir, cpu_target_list, u8, 16, 23);
register_bit!(icdsgir, satt, 15);
register_bits!(icdsgir, sbz, u32, 4, 14);
register_bits!(icdsgir, sgiintid, u8, 0, 3);

