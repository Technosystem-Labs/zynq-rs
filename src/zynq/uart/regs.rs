use volatile_register::{RO, WO, RW};

use crate::{register, register_bit, register_bits, register_bits_typed, register_at};

#[repr(u8)]
pub enum ChannelMode {
    Normal         = 0b00,
    AutomaticEcho  = 0b01,
    LocalLoopback  = 0b10,
    RemoteLoopback = 0b11,
}

#[repr(u8)]
pub enum ParityMode {
    EvenParity = 0b000,
    OddParity  = 0b001,
    ForceTo0   = 0b010,
    ForceTo1   = 0b011,
    None       = 0b100,
}

#[repr(u8)]
pub enum StopBits {
    One        = 0b00,
    OneAndHalf = 0b01,
    Two        = 0b10,
}

#[repr(C)]
pub struct RegisterBlock {
    pub control: Control,
    pub mode: Mode,
    pub intrpt_en: RW<u32>,
    pub intrpt_dis: RW<u32>,
    pub intrpt_mask: RO<u32>,
    pub chnl_int_sts: WO<u32>,
    pub baud_rate_gen: BaudRateGen,
    pub rcvr_timeout: RW<u32>,
    pub rcvr_fifo_trigger_level: RW<u32>,
    pub modem_ctrl: RW<u32>,
    pub modem_sts: RW<u32>,
    pub channel_sts: ChannelSts,
    pub tx_rx_fifo: TxRxFifo,
    pub baud_rate_divider: BaudRateDiv,
    pub flow_delay: RW<u32>,
    pub unused0: RO<u32>,
    pub unused1: RO<u32>,
    pub tx_fifo_trigger_level: RW<u32>,
}
register_at!(RegisterBlock, 0xE0000000, uart0);
register_at!(RegisterBlock, 0xE0001000, uart1);

register!(control, Control, RW, u32);
register_bit!(control, rxrst, 0);
register_bit!(control, txrst, 1);
register_bit!(control, rxen, 2);
register_bit!(control, rxdis, 3);
register_bit!(control, txen, 4);
register_bit!(control, txdis, 5);
register_bit!(control, rstto, 6);
register_bit!(control, sttbrk, 7);
register_bit!(control, stpbrk, 8);

register!(mode, Mode, RW, u32);
register_bits_typed!(mode,
                     /// Channel mode: Defines the mode of operation of the UART.
                     chmode, u8, ChannelMode, 8, 9);
register_bits_typed!(mode,
                     /// Number of stop bits
                     nbstop, u8, StopBits, 6, 7);
register_bits_typed!(mode,
                     /// Parity type select
                     par, u8, ParityMode, 3, 5);
register_bits!(mode,
               /// Character length select
               chrl, u8, 1, 2);
register_bit!(mode,
              /// Clock source select
              clks, 0);

register!(baud_rate_gen, BaudRateGen, RW, u32);
register_bits!(baud_rate_gen, cd, u16, 0, 15);

register!(channel_sts, ChannelSts, RO, u32);
register_bit!(channel_sts,
              /// Transmitter FIFO Nearly Full
              tnful, 14);
register_bit!(channel_sts,
              /// Tx FIFO fill level is greater than or equal to TTRIG?
              ttrig, 13);
register_bit!(channel_sts,
              /// Rx FIFO fill level is greater than or equal to FDEL?
              flowdel, 12);
register_bit!(channel_sts,
              /// Transmitter state machine active?
              tactive, 11);
register_bit!(channel_sts,
              /// Receiver state machine active?
              ractive, 10);
register_bit!(channel_sts,
              /// Tx FIFO is full?
              txfull, 4);
register_bit!(channel_sts,
              /// Tx FIFO is empty?
              txempty, 3);
register_bit!(channel_sts,
              /// Rx FIFO is full?
              rxfull, 2);
register_bit!(channel_sts,
              /// Rx FIFO is empty?
              rxempty, 1);
register_bit!(channel_sts,
              /// Rx FIFO fill level is greater than or equal to RTRIG?
              rxovr, 0);

register!(tx_rx_fifo, TxRxFifo, RW, u32);
register_bits!(tx_rx_fifo, data, u32, 0, 31);

register!(baud_rate_div, BaudRateDiv, RW, u32);
register_bits!(baud_rate_div, bdiv, u8, 0, 7);
