use volatile_register::{RO, RW, WO};

use libregister::{register, register_at, register_bit, register_bits, register_bits_typed};

#[repr(C)]
pub struct RegisterBlock {
    pub sdma_system_address: RW<u32>,
    pub block_size_block_count: BlockSizeBlockCount,
    pub argument: RW<u32>,
    pub transfer_mode_command: TransferModeCommand,
    pub responses: [RO<u32>; 4],
    pub buffer: RW<u32>,
    pub present_state: PresentState,
    /// Host. power, block gap, wakeup control
    pub control: Control,
    /// Clock and timeout control, and software reset register.
    pub timing_control: TimingControl,
    pub interrupt_status: InterruptStatus,
    pub interrupt_status_en: InterruptStatusEn,
    pub interrupt_signal_en: InterruptSignalEn,
    pub auto_cmd12_error_status: AutoCmd12ErrorStatus,
    pub capabilities: Capabilities,
    pub max_current_capabilities: MaxCurrentCapabilities,
    pub force_event: ForceEvent,
    pub adma_error_status: AdmaErrorStatus,
    pub adma_system_address: RW<u32>,
    pub boot_data_timeout_counter: RW<u32>,
    pub debug_selection: DebugSelection,
    pub spi_interrupt_support: SpiInterruptSupport,
    pub misc_reg: MiscReg,
}

#[allow(unused)]
#[repr(u8)]
pub enum CommandType {
    Normal = 0b00,
    Suspend = 0b01,
    Resume = 0b10,
    Abort = 0b11,
}

#[allow(unused)]
#[repr(u8)]
pub enum ResponseTypeSelect {
    NoResponse = 0b00,
    Length136 = 0b01,
    Length48 = 0b10,
    Legnth48Check = 0b11,
}

#[allow(unused)]
#[repr(u8)]
pub enum BusVoltage {
    /// 3.3V
    V33 = 0b111,
    /// 3.0V, typ.
    V30 = 0b110,
    /// 1.8V, typ.
    V18 = 0b101,
}

#[allow(unused)]
#[repr(u8)]
pub enum DmaSelect {
    SDMA = 0b00,
    ADMA1 = 0b01,
    ADMA2 = 0b10,
    ADMA3 = 0b11,
}

#[allow(unused)]
#[repr(u8)]
/// SDCLK Frequency divisor, d(number) means baseclock divided by (number).
pub enum SdclkFreqDivisor {
    D256 = 0x80,
    D128 = 0x40,
    D64 = 0x20,
    D32 = 0x10,
    D16 = 0x08,
    D8 = 0x04,
    D4 = 0x02,
    D2 = 0x01,
    D1 = 0x00,
}

#[allow(unused)]
#[repr(u8)]
pub enum AdmaErrorState {
    StStop = 0b00,
    StFds = 0b01,
    StTfr = 0b11,
}

#[allow(unused)]
#[repr(u8)]
pub enum SpecificationVersion {
    V1 = 0,
    V2 = 1,
}

register_at!(RegisterBlock, 0xE0100000, sd0);
register_at!(RegisterBlock, 0xE0101000, sd1);

register!(block_size_block_count, BlockSizeBlockCount, RW, u32);
register_bits!(
    block_size_block_count,
    /// Current transfer block count.
    blocks_count,
    u16,
    16,
    31
);
register_bits!(
    block_size_block_count,
    /// Host SDMA Buffer Size, size = 2^(val + 2) KB.
    dma_buffer_size,
    u8,
    12,
    14
);
register_bits!(
    block_size_block_count,
    /// Block size for data transfer. Unit: byte.
    transfer_block_size,
    u16,
    0,
    11
);

register!(transfer_mode_command, TransferModeCommand, RW, u32);
register_bits!(
    transfer_mode_command,
    /// Command Number.
    command_index,
    u8,
    24,
    29
);
register_bits_typed!(
    transfer_mode_command,
    /// Command type register.
    command_type,
    u8,
    CommandType,
    22,
    23
);
register_bit!(
    transfer_mode_command,
    /// 1 if data is present and shall be transferred using the DAT line.
    data_present_select,
    21
);
register_bit!(
    transfer_mode_command,
    /// If the index field shall be checked.
    index_check_en,
    20
);
register_bit!(
    transfer_mode_command,
    /// If CRC shall be checked.
    crc_check_en,
    19
);
register_bits_typed!(
    transfer_mode_command,
    /// Different type of response.
    response_type_select,
    u8,
    ResponseTypeSelect,
    16,
    17
);
register_bit!(
    transfer_mode_command,
    /// Enables the multi block DAT line data transfer.
    multi_block_en,
    5
);
register_bit!(
    transfer_mode_command,
    /// 1 if read (card to host), 0 if write (host to card).
    direction_select,
    4
);
register_bit!(
    transfer_mode_command,
    /// If CMD12 shall be issued automatically when last block transfer is completed.
    auto_cmd12_en,
    2
);
register_bit!(
    transfer_mode_command,
    /// Enable the block count register.
    block_count_en,
    1
);
register_bit!(
    transfer_mode_command,
    /// Enable DMA,
    dma_en,
    0
);

register!(present_state, PresentState, RO, u32);
register_bit!(
    present_state,
    /// CMD Line Signal Level.
    cmd_line_level,
    24
);
register_bit!(
    present_state,
    /// Signal level in DAT[3]
    dat3_level,
    23
);
register_bit!(
    present_state,
    /// Signal level in DAT[2]
    dat2_level,
    22
);
register_bit!(
    present_state,
    /// Signal level in DAT[1]
    dat1_level,
    21
);
register_bit!(
    present_state,
    /// Signal level in DAT[0]
    dat0_level,
    20
);
register_bit!(
    present_state,
    /// Write enabled and inverse of SDx_WP pin level.
    write_enabled,
    19
);
register_bit!(
    present_state,
    /// Card detected and inverse of SDx_CDn pin level.
    card_detected,
    18
);
register_bit!(present_state, card_state_stable, 17);
register_bit!(present_state, card_inserted, 16);
register_bit!(present_state, buffer_read_en, 11);
register_bit!(present_state, buffer_write_en, 10);
register_bit!(present_state, read_transfer_active, 9);
register_bit!(present_state, write_transfer_active, 8);
register_bit!(present_state, dat_line_active, 2);
register_bit!(present_state, command_inhibit_dat, 1);
register_bit!(present_state, command_inhibit_cmd, 0);

register!(control, Control, RW, u32);
register_bit!(
    control,
    /// Enable wakeup event via SD card removal assertion.
    wakeup_on_removal,
    26
);
register_bit!(
    control,
    /// Enable wakeup event via SD card insertion assertion.
    wakeup_on_insertion,
    25
);
register_bit!(
    control,
    /// Enable wakeup event via card interrupt assertion.
    wakeup_on_interrupt,
    24
);
register_bit!(
    control,
    ///Enable interrupt detection at the block gap for a multiple block transfer.
    interrupt_at_block_gap,
    19
);
register_bit!(
    control,
    /// Enable the use of the read wait protocol.
    read_wait_control,
    18
);
register_bit!(
    control,
    /// Restart a trasaction which was stopped using the stop at block gap request.
    continue_req,
    17
);
register_bit!(
    control,
    /// Stop executing a transaction at the next block gap.
    stop_at_block_gap_req,
    16
);
register_bits_typed!(control, bus_voltage, u8, BusVoltage, 9, 11);
register_bit!(control, bus_power, 8);
register_bit!(
    control,
    /// Selects source for card detection. 0 for SDCD#, 1 for card detect test level.
    card_detect_signal,
    7
);
register_bit!(
    control,
    /// Indicates card inserted or not. Enabled when card detect signal is 1.
    card_detect_test_level,
    6
);
register_bits_typed!(control, dma_select, u8, DmaSelect, 3, 4);
register_bit!(control, high_speed_en, 2);
register_bit!(
    control,
    /// Select the data width of the HC. 1 for 4-bit, 0 for 1-bit.
    data_width_select,
    1
);
register_bit!(
    control,
    /// 1 for LED on, 0 for LED off.
    led_control,
    0
);

register!(timing_control, TimingControl, RW, u32);
register_bit!(
    timing_control,
    /// Software reset for DAT line.
    software_reset_dat,
    26
);
register_bit!(
    timing_control,
    /// Software reset for CMD line.
    software_reset_cmd,
    25
);
register_bit!(
    timing_control,
    /// Software reset for ALL.
    software_reset_all,
    24
);
register_bits!(
    timing_control,
    /// Determines the interval by which DAT line time-outs are detected.
    /// Interval = TMCLK * 2^(13 + val)
    /// Note: 0b1111 is reserved.
    timeout_counter_value,
    u8,
    16,
    19
);
register_bits_typed!(
    timing_control,
    /// Selects the frequency divisor, thus the clock frequency for SDCLK.
    /// Choose the smallest possible divisor which results in a clock frequency
    /// that is less than or equal to the target frequency.
    sdclk_freq_divisor,
    u8,
    SdclkFreqDivisor,
    8,
    15
);
register_bit!(timing_control, sd_clk_en, 2);
register_bit!(
    timing_control,
    /// 1 when SD clock is stable.
    /// Note that this field is read-only.
    internal_clk_stable,
    1,
    RO
);
register_bit!(timing_control, internal_clk_en, 0);

register!(interrupt_status, InterruptStatus, RW, u32, 1 << 15 | 1 << 8);
register_bit!(interrupt_status, ceata_error, 29, WTC);
register_bit!(interrupt_status, target_response_error, 28, WTC);
register_bit!(interrupt_status, adma_error, 25, WTC);
register_bit!(interrupt_status, auto_cmd12_error, 24, WTC);
register_bit!(interrupt_status, current_limit_error, 23, WTC);
register_bit!(interrupt_status, data_end_bit_error, 22, WTC);
register_bit!(interrupt_status, data_crc_error, 21, WTC);
register_bit!(interrupt_status, data_timeout_error, 20, WTC);
register_bit!(interrupt_status, command_index_error, 19, WTC);
register_bit!(interrupt_status, command_end_bit_error, 18, WTC);
register_bit!(interrupt_status, command_crc_error, 17, WTC);
register_bit!(interrupt_status, command_timeout_error, 16, WTC);
register_bit!(interrupt_status, error_interrupt, 15, RO);
register_bit!(interrupt_status, boot_terminate_interrupt, 10, WTC);
register_bit!(interrupt_status, boot_ack_rcv, 9, WTC);
register_bit!(interrupt_status, card_interrupt, 8, RO);
register_bit!(interrupt_status, card_removal, 7, WTC);
register_bit!(interrupt_status, card_insertion, 6, WTC);
register_bit!(interrupt_status, buffer_read_ready, 5, WTC);
register_bit!(interrupt_status, buffer_write_ready, 4, WTC);
register_bit!(interrupt_status, dma_interrupt, 3, WTC);
register_bit!(interrupt_status, block_gap_event, 2, WTC);
register_bit!(interrupt_status, transfer_complete, 1, WTC);
register_bit!(interrupt_status, command_complete, 0, WTC);

register!(interrupt_status_en, InterruptStatusEn, RW, u32);
register_bit!(interrupt_status_en, ceata_error_status_en, 29);
register_bit!(interrupt_status_en, target_response_error_status_en, 28);
register_bit!(interrupt_status_en, adma_error_status_en, 25);
register_bit!(interrupt_status_en, auto_cmd12_error_status_en, 24);
register_bit!(interrupt_status_en, current_limit_error_status_en, 23);
register_bit!(interrupt_status_en, data_end_bit_error_status_en, 22);
register_bit!(interrupt_status_en, data_crc_error_status_en, 21);
register_bit!(interrupt_status_en, data_timeout_error_status_en, 20);
register_bit!(interrupt_status_en, cmd_index_error_status_en, 19);
register_bit!(interrupt_status_en, cmd_end_bit_error_status_en, 18);
register_bit!(interrupt_status_en, cmd_crc_error_status_en, 17);
register_bit!(interrupt_status_en, cmd_timeout_error_status_en, 16);
register_bit!(interrupt_status_en, fixed_to_0, 15, RO);
register_bit!(interrupt_status_en, boot_terminate_interrupt_en, 10);
register_bit!(interrupt_status_en, boot_ack_rcv_en, 9);
register_bit!(interrupt_status_en, card_interrupt_status_en, 8);
register_bit!(interrupt_status_en, card_removal_status_en, 7);
register_bit!(interrupt_status_en, card_insertion_status_en, 6);
register_bit!(interrupt_status_en, buffer_read_ready_status_en, 5);
register_bit!(interrupt_status_en, buffer_write_ready_status_en, 4);
register_bit!(interrupt_status_en, dma_interrupt_status_en, 3);
register_bit!(interrupt_status_en, block_gap_evt_status_en, 2);
register_bit!(interrupt_status_en, transfer_complete_status_en, 1);
register_bit!(interrupt_status_en, cmd_complete_status_en, 0);

register!(interrupt_signal_en, InterruptSignalEn, RW, u32);
register_bit!(interrupt_signal_en, ceata_error_signal_en, 29);
register_bit!(interrupt_signal_en, target_response_error_signal_en, 28);
register_bit!(interrupt_signal_en, adma_error_signal_en, 25);
register_bit!(interrupt_signal_en, auto_cmd12_error_signal_en, 24);
register_bit!(interrupt_signal_en, current_limit_error_signal_en, 23);
register_bit!(interrupt_signal_en, data_end_bit_error_signal_en, 22);
register_bit!(interrupt_signal_en, data_crc_error_signal_en, 21);
register_bit!(interrupt_signal_en, data_timeout_error_signal_en, 20);
register_bit!(interrupt_signal_en, cmd_index_error_signal_en, 19);
register_bit!(interrupt_signal_en, cmd_end_bit_error_signal_en, 18);
register_bit!(interrupt_signal_en, cmd_crc_error_signal_en, 17);
register_bit!(interrupt_signal_en, cmd_timeout_error_signal_en, 16);
register_bit!(interrupt_signal_en, fixed_to_0, 15, RO);
register_bit!(interrupt_signal_en, boot_terminate_interrupt_signal_en, 10);
register_bit!(interrupt_signal_en, boot_ack_rcv_signal_en, 9);
register_bit!(interrupt_signal_en, card_interrupt_signal_en, 8);
register_bit!(interrupt_signal_en, card_removal_signal_en, 7);
register_bit!(interrupt_signal_en, card_insertion_signal_en, 6);
register_bit!(interrupt_signal_en, buffer_read_ready_signal_en, 5);
register_bit!(interrupt_signal_en, buffer_write_ready_signal_en, 4);
register_bit!(interrupt_signal_en, dma_interrupt_signal_en, 3);
register_bit!(interrupt_signal_en, block_gap_evt_signal_en, 2);
register_bit!(interrupt_signal_en, transfer_complete_signal_en, 1);
register_bit!(interrupt_signal_en, cmd_complete_signal_en, 0);

register!(auto_cmd12_error_status, AutoCmd12ErrorStatus, RO, u32);
register_bit!(
    auto_cmd12_error_status,
    cmd_not_issued_by_auto_cmd12_error,
    7
);
register_bit!(auto_cmd12_error_status, index_error, 4);
register_bit!(auto_cmd12_error_status, end_bit_error, 3);
register_bit!(auto_cmd12_error_status, crc_error, 2);
register_bit!(auto_cmd12_error_status, timeout_error, 1);
register_bit!(auto_cmd12_error_status, not_executed, 0);

register!(capabilities, Capabilities, RO, u32);
register_bit!(capabilities, spi_block_mode, 30);
register_bit!(capabilities, spi_mode, 29);
register_bit!(capabilities, support_64bit, 28);
register_bit!(capabilities, interrupt_mode, 27);
register_bit!(capabilities, voltage_1_8, 26);
register_bit!(capabilities, voltage_3_0, 25);
register_bit!(capabilities, voltage_3_3, 24);
register_bit!(capabilities, suspend_resume, 23);
register_bit!(capabilities, sdma, 22);
register_bit!(capabilities, hgih_speed, 21);
register_bit!(capabilities, adma2, 19);
register_bit!(capabilities, extended_media_bus, 18);
register_bits!(
    capabilities,
    /// Length = 2^(9 + v) bytes.
    max_block_len,
    u8,
    16,
    17
);
register_bit!(capabilities, timeout_clock_unit, 7);

register!(max_current_capabilities, MaxCurrentCapabilities, RO, u32);
register_bits!(max_current_capabilities, max_current_1_8v, u8, 16, 23);
register_bits!(max_current_capabilities, max_current_3_0v, u8, 8, 15);
register_bits!(max_current_capabilities, max_current_3_3v, u8, 0, 7);

register!(force_event, ForceEvent, WO, u32);
register_bit!(force_event, ceata_error, 29);
register_bit!(force_event, target_response_error, 28);
register_bit!(force_event, adma_error, 25);
register_bit!(force_event, auto_cmd12_error, 24);
register_bit!(force_event, current_limit_error, 23);
register_bit!(force_event, data_end_bit_error, 22);
register_bit!(force_event, data_crc_error, 21);
register_bit!(force_event, data_timeout_error, 20);
register_bit!(force_event, cmd_index_error, 19);
register_bit!(force_event, cmd_end_bit_error, 18);
register_bit!(force_event, cmd_crc_error, 17);
register_bit!(force_event, cmd_timeout_error, 16);
register_bit!(force_event, cmd_not_issued_by_auto_cmd12_error, 7);
register_bit!(force_event, auto_cmd12_index_error, 4);
register_bit!(force_event, auto_cmd12_end_bit_error, 3);
register_bit!(force_event, auto_cmd12_crc_error, 2);
register_bit!(force_event, auto_cmd12_timeout_error, 1);
register_bit!(force_event, auto_cmd12_not_executed, 0);

register!(adma_error_status, AdmaErrorStatus, RW, u32, 0b11);
register_bit!(adma_error_status, length_mismatch_error, 2, WTC);
register_bits_typed!(adma_error_status, error_state, u8, AdmaErrorState, 0, 1);

register!(debug_selection, DebugSelection, WO, u32);
register_bit!(debug_selection, debug_select, 0);

register!(spi_interrupt_support, SpiInterruptSupport, RW, u32);
register_bits!(
    spi_interrupt_support,
    /// There should be a problem with the documentation of this field.
    spi_int_support,
    u8,
    0,
    7
);

register!(misc_reg, MiscReg, RO, u32);
register_bits!(misc_reg, vendor_version_num, u8, 24, 31);
register_bits_typed!(misc_reg, spec_ver, u8, SpecificationVersion, 16, 23);
register_bits!(
    misc_reg,
    /// Logical OR of interrupt signal and wakeup signal for each slot.
    slot_interrupt_signal,
    u8,
    0,
    7
);
