use volatile_register::{RO, WO, RW};

use crate::{register, register_bit, register_bits, register_at, regs::*};

pub enum ChannelMode {
    Normal = 0b00,
    AutomaticEcho = 0b01,
    LocalLoopback = 0b10,
    RemoteLoopback = 0b11,
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
/// Channel mode: Defines the mode of operation of the UART.
register_bits!(mode, chmode, u8, 8, 9);
register_bits!(mode, par, u8, 3, 5);

register!(baud_rate_gen, BaudRateGen, RW, u32);
register_bits!(baud_rate_gen, cd, u16, 0, 15);

register!(channel_sts, ChannelSts, RO, u32);
register_bit!(channel_sts, txfull, 4);

register!(tx_rx_fifo, TxRxFifo, RW, u32);
register_bits!(tx_rx_fifo, data, u32, 0, 31);

register!(baud_rate_div, BaudRateDiv, RW, u32);
register_bits!(baud_rate_div, bdiv, u8, 0, 7);
