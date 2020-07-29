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
    /// Interrupt Set-enable Register 0
    pub icdiser0: RW<u32>,
    /// Interrupt Set-enable Register 1
    pub icdiser1: RW<u32>,
    /// Interrupt Set-enable Register 2
    pub icdiser2: RW<u32>,
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
    /// Interrupt Processor Targets Register 0
    pub icdiptr0: ICDIPTR0,
    /// Interrupt Processor Targets Register 1
    pub icdiptr1: ICDIPTR1,
    /// Interrupt Processor Targets Register 2
    pub icdiptr2: ICDIPTR2,
    /// Interrupt Processor Targets Register 3
    pub icdiptr3: ICDIPTR3,
    /// Interrupt Processor Targets Register 4
    pub icdiptr4: RW<u32>,
    /// Interrupt Processor Targets Register 5
    pub icdiptr5: RO<u32>,
    /// Interrupt Processor Targets Register 6
    pub icdiptr6: ICDIPTR6,
    /// Interrupt Processor Targets Register 7
    pub icdiptr7: ICDIPTR7,
    /// Interrupt Processor Targets Register 8
    pub icdiptr8: ICDIPTR8,
    /// Interrupt Processor Targets Register 9
    pub icdiptr9: ICDIPTR9,
    /// Interrupt Processor Targets Register 10
    pub icdiptr10: ICDIPTR10,
    /// Interrupt Processor Targets Register 11
    pub icdiptr11: ICDIPTR11,
    /// Interrupt Processor Targets Register 12
    pub icdiptr12: ICDIPTR12,
    /// Interrupt Processor Targets Register 13
    pub icdiptr13: ICDIPTR13,
    /// Interrupt Processor Targets Register 14
    pub icdiptr14: ICDIPTR14,
    /// Interrupt Processor Targets Register 15
    pub icdiptr15: ICDIPTR15,
    /// Interrupt Processor Targets Register 16
    pub icdiptr16: ICDIPTR16,
    /// Interrupt Processor Targets Register 17
    pub icdiptr17: ICDIPTR17,
    /// Interrupt Processor Targets Register 18
    pub icdiptr18: ICDIPTR18,
    /// Interrupt Processor Targets Register 19
    pub icdiptr19: ICDIPTR19,
    /// Interrupt Processor Targets Register 20
    pub icdiptr20: ICDIPTR20,
    /// Interrupt Processor Targets Register 21
    pub icdiptr21: ICDIPTR21,
    /// Interrupt Processor Targets Register 22
    pub icdiptr22: ICDIPTR22,
    /// Interrupt Processor Targets Register 23
    pub icdiptr23: ICDIPTR23,
    unused15: [u32; 232],
    /// Interrupt Configuration Register 0
    pub icdicfr0: ICDICFR0,
    /// Interrupt Configuration Register 1
    pub icdicfr1: ICDICFR1,
    /// Interrupt Configuration Register 2
    pub icdicfr2: ICDICFR2,
    /// Interrupt Configuration Register 3
    pub icdicfr3: ICDICFR3,
    /// Interrupt Configuration Register 4
    pub icdicfr4: ICDICFR4,
    /// Interrupt Configuration Register 5
    pub icdicfr5: ICDICFR5,
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

register!(icdipt_r0, ICDIPTR0, RO, u32);
register_bits!(icdipt_r0, target_3, u8, 24, 25);
register_bits!(icdipt_r0, target_2, u8, 16, 17);
register_bits!(icdipt_r0, target_1, u8, 8, 9);
register_bits!(icdipt_r0, target_0, u8, 0, 1);

register!(icdipt_r1, ICDIPTR1, RO, u32);
register_bits!(icdipt_r1, target_7, u8, 24, 25);
register_bits!(icdipt_r1, target_6, u8, 16, 17);
register_bits!(icdipt_r1, target_5, u8, 8, 9);
register_bits!(icdipt_r1, target_4, u8, 0, 1);

register!(icdipt_r2, ICDIPTR2, RO, u32);
register_bits!(icdipt_r2, target_11, u8, 24, 25);
register_bits!(icdipt_r2, target_10, u8, 16, 17);
register_bits!(icdipt_r2, target_9, u8, 8, 9);
register_bits!(icdipt_r2, target_8, u8, 0, 1);

register!(icdipt_r3, ICDIPTR3, RO, u32);
register_bits!(icdipt_r3, target_15, u8, 24, 25);
register_bits!(icdipt_r3, target_14, u8, 16, 17);
register_bits!(icdipt_r3, target_13, u8, 8, 9);
register_bits!(icdipt_r3, target_12, u8, 0, 1);

register!(icdipt_r6, ICDIPTR6, RO, u32);
register_bits!(icdipt_r6, target_27, u8, 24, 25);

register!(icdipt_r7, ICDIPTR7, RO, u32);
register_bits!(icdipt_r7, target_31, u8, 24, 25);
register_bits!(icdipt_r7, target_30, u8, 16, 17);
register_bits!(icdipt_r7, target_29, u8, 8, 9);
register_bits!(icdipt_r7, target_28, u8, 0, 1);

register!(icdipt_r8, ICDIPTR8, RW, u32);
register_bits!(icdipt_r8, target_35, u8, 24, 25);
register_bits!(icdipt_r8, target_34, u8, 16, 17);
register_bits!(icdipt_r8, target_33, u8, 8, 9);
register_bits!(icdipt_r8, target_32, u8, 0, 1);

register!(icdipt_r9, ICDIPTR9, RW, u32);
register_bits!(icdipt_r9, target_39, u8, 24, 25);
register_bits!(icdipt_r9, target_38, u8, 16, 17);
register_bits!(icdipt_r9, target_37, u8, 8, 9);
register_bits!(icdipt_r9, target_36, u8, 0, 1);

register!(icdipt_r10, ICDIPTR10, RW, u32);
register_bits!(icdipt_r10, target_43, u8, 24, 25);
register_bits!(icdipt_r10, target_42, u8, 16, 17);
register_bits!(icdipt_r10, target_41, u8, 8, 9);
register_bits!(icdipt_r10, target_40, u8, 0, 1);

register!(icdipt_r11, ICDIPTR11, RW, u32);
register_bits!(icdipt_r11, target_47, u8, 24, 25);
register_bits!(icdipt_r11, target_46, u8, 16, 17);
register_bits!(icdipt_r11, target_45, u8, 8, 9);
register_bits!(icdipt_r11, target_44, u8, 0, 1);

register!(icdipt_r12, ICDIPTR12, RW, u32);
register_bits!(icdipt_r12, target_51, u8, 24, 25);
register_bits!(icdipt_r12, target_50, u8, 16, 17);
register_bits!(icdipt_r12, target_49, u8, 8, 9);
register_bits!(icdipt_r12, target_48, u8, 0, 1);

register!(icdipt_r13, ICDIPTR13, RW, u32);
register_bits!(icdipt_r13, target_55, u8, 24, 25);
register_bits!(icdipt_r13, target_54, u8, 16, 17);
register_bits!(icdipt_r13, target_53, u8, 8, 9);
register_bits!(icdipt_r13, target_52, u8, 0, 1);

register!(icdipt_r14, ICDIPTR14, RW, u32);
register_bits!(icdipt_r14, target_59, u8, 24, 25);
register_bits!(icdipt_r14, target_58, u8, 16, 17);
register_bits!(icdipt_r14, target_57, u8, 8, 9);
register_bits!(icdipt_r14, target_56, u8, 0, 1);

register!(icdipt_r15, ICDIPTR15, RW, u32);
register_bits!(icdipt_r15, target_63, u8, 24, 25);
register_bits!(icdipt_r15, target_62, u8, 16, 17);
register_bits!(icdipt_r15, target_61, u8, 8, 9);
register_bits!(icdipt_r15, target_60, u8, 0, 1);

register!(icdipt_r16, ICDIPTR16, RW, u32);
register_bits!(icdipt_r16, target_67, u8, 24, 25);
register_bits!(icdipt_r16, target_66, u8, 16, 17);
register_bits!(icdipt_r16, target_65, u8, 8, 9);
register_bits!(icdipt_r16, target_64, u8, 0, 1);

register!(icdipt_r17, ICDIPTR17, RW, u32);
register_bits!(icdipt_r17, target_71, u8, 24, 25);
register_bits!(icdipt_r17, target_70, u8, 16, 17);
register_bits!(icdipt_r17, target_69, u8, 8, 9);
register_bits!(icdipt_r17, target_68, u8, 0, 1);

register!(icdipt_r18, ICDIPTR18, RW, u32);
register_bits!(icdipt_r18, target_75, u8, 24, 25);
register_bits!(icdipt_r18, target_74, u8, 16, 17);
register_bits!(icdipt_r18, target_73, u8, 8, 9);
register_bits!(icdipt_r18, target_72, u8, 0, 1);

register!(icdipt_r19, ICDIPTR19, RW, u32);
register_bits!(icdipt_r19, target_79, u8, 24, 25);
register_bits!(icdipt_r19, target_78, u8, 16, 17);
register_bits!(icdipt_r19, target_77, u8, 8, 9);
register_bits!(icdipt_r19, target_76, u8, 0, 1);

register!(icdipt_r20, ICDIPTR20, RW, u32);
register_bits!(icdipt_r20, target_83, u8, 24, 25);
register_bits!(icdipt_r20, target_82, u8, 16, 17);
register_bits!(icdipt_r20, target_81, u8, 8, 9);
register_bits!(icdipt_r20, target_80, u8, 0, 1);

register!(icdipt_r21, ICDIPTR21, RW, u32);
register_bits!(icdipt_r21, target_87, u8, 24, 25);
register_bits!(icdipt_r21, target_86, u8, 16, 17);
register_bits!(icdipt_r21, target_85, u8, 8, 9);
register_bits!(icdipt_r21, target_84, u8, 0, 1);

register!(icdipt_r22, ICDIPTR22, RW, u32);
register_bits!(icdipt_r22, target_91, u8, 24, 25);
register_bits!(icdipt_r22, target_90, u8, 16, 17);
register_bits!(icdipt_r22, target_89, u8, 8, 9);
register_bits!(icdipt_r22, target_88, u8, 0, 1);

register!(icdipt_r23, ICDIPTR23, RW, u32);
register_bits!(icdipt_r23, target_95, u8, 24, 25);
register_bits!(icdipt_r23, target_94, u8, 16, 17);
register_bits!(icdipt_r23, target_93, u8, 8, 9);
register_bits!(icdipt_r23, target_92, u8, 0, 1);

register!(icdicf_r0, ICDICFR0, RO, u32);
register_bits!(icdicf_r0, config_15, u8, 30, 31);
register_bits!(icdicf_r0, config_14, u8, 28, 29);
register_bits!(icdicf_r0, config_13, u8, 26, 27);
register_bits!(icdicf_r0, config_12, u8, 24, 25);
register_bits!(icdicf_r0, config_11, u8, 22, 23);
register_bits!(icdicf_r0, config_10, u8, 20, 21);
register_bits!(icdicf_r0, config_9, u8, 18, 19);
register_bits!(icdicf_r0, config_8, u8, 16, 17);
register_bits!(icdicf_r0, config_7, u8, 14, 15);
register_bits!(icdicf_r0, config_6, u8, 12, 13);
register_bits!(icdicf_r0, config_5, u8, 10, 11);
register_bits!(icdicf_r0, config_4, u8, 8, 9);
register_bits!(icdicf_r0, config_3, u8, 6, 7);
register_bits!(icdicf_r0, config_2, u8, 4, 5);
register_bits!(icdicf_r0, config_1, u8, 2, 3);
register_bits!(icdicf_r0, config_0, u8, 0, 1);

register!(icdicf_r1, ICDICFR1, RW, u32);
register_bits!(icdicf_r1, config_31, u8, 30, 31);
register_bits!(icdicf_r1, config_30, u8, 28, 29);
register_bits!(icdicf_r1, config_29, u8, 26, 27);
register_bits!(icdicf_r1, config_28, u8, 24, 25);
register_bits!(icdicf_r1, config_27, u8, 22, 23);

register!(icdicf_r2, ICDICFR2, RW, u32);
register_bits!(icdicf_r2, config_47, u8, 30, 31);
register_bits!(icdicf_r2, config_46, u8, 28, 29);
register_bits!(icdicf_r2, config_45, u8, 26, 27);
register_bits!(icdicf_r2, config_44, u8, 24, 25);
register_bits!(icdicf_r2, config_43, u8, 22, 23);
register_bits!(icdicf_r2, config_42, u8, 20, 21);
register_bits!(icdicf_r2, config_41, u8, 18, 19);
register_bits!(icdicf_r2, config_40, u8, 16, 17);
register_bits!(icdicf_r2, config_39, u8, 14, 15);
register_bits!(icdicf_r2, config_38, u8, 12, 13);
register_bits!(icdicf_r2, config_37, u8, 10, 11);
register_bits!(icdicf_r2, config_36, u8, 8, 9);
register_bits!(icdicf_r2, config_35, u8, 6, 7);
register_bits!(icdicf_r2, config_34, u8, 4, 5);
register_bits!(icdicf_r2, config_33, u8, 2, 3);
register_bits!(icdicf_r2, config_32, u8, 0, 1);

register!(icdicf_r3, ICDICFR3, RW, u32);
register_bits!(icdicf_r3, config_63, u8, 30, 31);
register_bits!(icdicf_r3, config_62, u8, 28, 29);
register_bits!(icdicf_r3, config_61, u8, 26, 27);
register_bits!(icdicf_r3, config_60, u8, 24, 25);
register_bits!(icdicf_r3, config_59, u8, 22, 23);
register_bits!(icdicf_r3, config_58, u8, 20, 21);
register_bits!(icdicf_r3, config_57, u8, 18, 19);
register_bits!(icdicf_r3, config_56, u8, 16, 17);
register_bits!(icdicf_r3, config_55, u8, 14, 15);
register_bits!(icdicf_r3, config_54, u8, 12, 13);
register_bits!(icdicf_r3, config_53, u8, 10, 11);
register_bits!(icdicf_r3, config_52, u8, 8, 9);
register_bits!(icdicf_r3, config_51, u8, 6, 7);
register_bits!(icdicf_r3, config_50, u8, 4, 5);
register_bits!(icdicf_r3, config_49, u8, 2, 3);
register_bits!(icdicf_r3, config_48, u8, 0, 1);

register!(icdicf_r4, ICDICFR4, RW, u32);
register_bits!(icdicf_r4, config_79, u8, 30, 31);
register_bits!(icdicf_r4, config_78, u8, 28, 29);
register_bits!(icdicf_r4, config_77, u8, 26, 27);
register_bits!(icdicf_r4, config_76, u8, 24, 25);
register_bits!(icdicf_r4, config_75, u8, 22, 23);
register_bits!(icdicf_r4, config_74, u8, 20, 21);
register_bits!(icdicf_r4, config_73, u8, 18, 19);
register_bits!(icdicf_r4, config_72, u8, 16, 17);
register_bits!(icdicf_r4, config_71, u8, 14, 15);
register_bits!(icdicf_r4, config_70, u8, 12, 13);
register_bits!(icdicf_r4, config_69, u8, 10, 11);
register_bits!(icdicf_r4, config_68, u8, 8, 9);
register_bits!(icdicf_r4, config_67, u8, 6, 7);
register_bits!(icdicf_r4, config_66, u8, 4, 5);
register_bits!(icdicf_r4, config_65, u8, 2, 3);
register_bits!(icdicf_r4, config_64, u8, 0, 1);

register!(icdicf_r5, ICDICFR5, RW, u32);
register_bits!(icdicf_r5, config_95, u8, 30, 31);
register_bits!(icdicf_r5, config_94, u8, 28, 29);
register_bits!(icdicf_r5, config_93, u8, 26, 27);
register_bits!(icdicf_r5, config_92, u8, 24, 25);
register_bits!(icdicf_r5, config_91, u8, 22, 23);
register_bits!(icdicf_r5, config_90, u8, 20, 21);
register_bits!(icdicf_r5, config_89, u8, 18, 19);
register_bits!(icdicf_r5, config_88, u8, 16, 17);
register_bits!(icdicf_r5, config_87, u8, 14, 15);
register_bits!(icdicf_r5, config_86, u8, 12, 13);
register_bits!(icdicf_r5, config_85, u8, 10, 11);
register_bits!(icdicf_r5, config_84, u8, 8, 9);
register_bits!(icdicf_r5, config_83, u8, 6, 7);
register_bits!(icdicf_r5, config_82, u8, 4, 5);
register_bits!(icdicf_r5, config_81, u8, 2, 3);
register_bits!(icdicf_r5, config_80, u8, 0, 1);

register!(ppi_status, PpiStatus, RO, u32);
register_bits!(ppi_status, ppi_status, u8, 11, 15);
register_bits!(ppi_status, sbz, u32, 0, 10);

register!(icdsgir, ICDSGIR, RW, u32);
register_bits!(icdsgir, target_list_filter, u8, 24, 25);
register_bits!(icdsgir, cpu_target_list, u8, 16, 23);
register_bit!(icdsgir, satt, 15);
register_bits!(icdsgir, sbz, u32, 4, 14);
register_bits!(icdsgir, sgiintid, u8, 0, 3);

