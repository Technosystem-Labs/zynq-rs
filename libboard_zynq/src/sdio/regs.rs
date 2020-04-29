use volatile_register::{RO, RW, WO};

use libregister::{register, register_at, register_bit, register_bits, register_bits_typed};

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
    v33 = 0b111,
    /// 3.0V, typ.
    v30 = 0b110,
    /// 1.8V, typ.
    v18 = 0b101,
}

#[allow(unused)]
#[repr(u8)]
pub enum DmaSelect {
    sdma = 0b00,
    adma1 = 0b01,
    adma2 = 0b10,
    adma3 = 0b11,
}

#[allow(unused)]
#[repr(u8)]
/// SDCLK Frequency divisor, d(number) means baseclock divides by (number).
pub enum SdclkFreqDivisor {
    d256 = 0x80,
    d128 = 0x40,
    d64 = 0x20,
    d32 = 0x10,
    d16 = 0x08,
    d8 = 0x04,
    d4 = 0x02,
    d2 = 0x01,
    d1 = 0x00,
}

#[repr(C)]
pub struct RegisterBlock {
    pub sdma_system_address: RM<u32>,
    pub block_size_block_count: BockSizeBlockCount,
    pub argument: RW<u32>,
    pub transfer_mode_command: TransferModeCommand,
    pub responses: [RO<u32>; 4],
    pub buffer: RW<u32>,
    pub present_state: PresentState,
    /// Host. power, block gap, wakeup control
    pub control: Control,
    /// Clock and timeout control, and software reset register.
    pub timing_control: TimingControl,
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
    24,
    29
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
regsiter_bit!(present_state, card_state_stable, 17);
register_bit!(present_state, card_inserted, 16);
register_bit!(present_state, buffer_read_en, 11,);
register_bit!(present_state, buffer_write_en, 10);
register_bit!(present_state, read_transfer_active, 9);
register_bit!(present_state, write_transfer_active, 8);
register_bit!(present_state, dat_line_active, 2);
register_bit!(present_state, command_inhibit_dat, 1);
register_bit!(present_state, command_inhibit_cmd, 0);

register!(control, Control, RW, u32);
register_bit!(
    contorl,
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
register_bits!(timing_control, sd_clk_en, 2);
register_bits!(timing_control,
    /// 1 when SD clock is stable.
    /// Note that this field is read-only.
    internal_clk_stable, 1);
register_bits!(timing_control, internal_clk_en, 0);
