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

#[repr(C)]
pub struct RegisterBlock {
    pub sdma_system_address: RM<u32>,
    pub block_size_block_count: BockSizeBlockCount,
    pub argument: RW<u32>,
    pub transfer_mode_command: TransferModeCommand,
    pub responses: [RO<u32>; 4],
    pub buffer: RW<u32>,
    pub present_state: PresentState,
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
regsiter_bit!(
    present_state,
    card_state_stable,
    17
);
register_bit!(
    present_state,
    card_inserted,
    16
);
register_bit!(
    present_state,
    buffer_read_en,
    11,
);
register_bit!(
    present_state,
    buffer_write_en,
    10
);
register_bit!(
    present_state,
    read_transfer_active,
    9
);
register_bit!(
    present_state,
    write_transfer_active,
    8
);
register_bit!(
    present_state,
    dat_line_active,
    2
);
register_bit!(
    present_state,
    command_inhibit_dat,
    1
);
register_bit!(
    present_state,
    command_inhibit_cmd,
    0
);