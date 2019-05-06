use volatile_register::{RO, WO, RW};
use bit_field::BitField;

use crate::{register, register_bit, register_bits, regs::Register};

#[repr(u8)]
pub enum ParityMode {
    EvenParity = 0b000,
    OddParity  = 0b001,
    ForceTo0   = 0b010,
    ForceTo1   = 0b011,
}

#[repr(packed)]
pub struct RegisterBlock {
    control: Control,
    mode: Mode,
    intrpt_en: RW<u32>,
    intrpt_dis: RW<u32>,
    intrpt_mask: RO<u32>,
    chnl_int_sts: WO<u32>,
    baud_rate_gen: BaudRateGen,
    rcvr_timeout: RW<u32>,
    rcvr_fifo_trigger_level: RW<u32>,
    modem_ctrl: RW<u32>,
    modem_sts: RW<u32>,
    channel_sts: ChannelSts,
    tx_rx_fifo: TxRxFifo,
    baud_rate_divider: BaudRateDiv,
    flow_delay: RW<u32>,
    reserved0: RO<u32>,
    reserved1: RO<u32>,
    tx_fifo_trigger_level: RW<u32>,
}

register!(control, Control, u32);
register_bit!(control, rxrst, 0);
register_bit!(control, txrst, 1);
register_bit!(control, rxen, 2);
register_bit!(control, rxdis, 3);
register_bit!(control, txen, 4);
register_bit!(control, txdis, 5);

register!(mode, Mode, u32);
register_bits!(mode, par, u8, 3, 5);

register!(baud_rate_gen, BaudRateGen, u32);
register_bits!(baud_rate_gen, cd, u16, 0, 15);

register!(channel_sts, ChannelSts, u32);
register_bit!(channel_sts, txfull, 4);

register!(tx_rx_fifo, TxRxFifo, u32);
register_bits!(tx_rx_fifo, data, u32, 0, 31);

register!(baud_rate_div, BaudRateDiv, u32);
register_bits!(baud_rate_div, bdiv, u8, 0, 7);

impl RegisterBlock {
    const UART0: *mut Self = 0xE0000000 as *mut _;
    const UART1: *mut Self = 0xE0001000 as *mut _;

    pub fn uart0() -> &'static mut Self {
        unsafe { &mut *Self::UART0 }
    }

    pub fn uart1() -> &'static mut Self {
        unsafe { &mut *Self::UART1 }
    }

    pub fn configure(&self) {
        // Confiugre UART character frame
        // * Disable clock-divider
        // * 8-bit
        // * 1 stop bit
        // * Normal channel mode
        // * no parity
        let parity_mode = ParityMode::ForceTo0;
        self.mode.write(Mode::zeroed().par(parity_mode as u8));

        // Configure the Baud Rate
        self.disable_rx();
        self.disable_tx();

        // 9,600 baud
        self.baud_rate_gen.write(BaudRateGen::zeroed().cd(651));
        self.baud_rate_divider.write(BaudRateDiv::zeroed().bdiv(7));

        self.reset_rx();
        self.reset_tx();
        self.enable_rx();
        self.enable_tx();
    }

    pub fn write_byte(&self, value: u8) {
        self.tx_rx_fifo.write(TxRxFifo::zeroed().data(value.into()));
    }

    fn disable_rx(&self) {
        self.control.modify(|_, w| {
            w.rxen(false)
             .rxdis(true)
        })
    }
    
    fn disable_tx(&self) {
        self.control.modify(|_, w| {
            w.txen(false)
             .txdis(true)
        })
    }
    
    fn enable_rx(&self) {
        self.control.modify(|_, w| {
            w.rxen(true)
             .rxdis(false)
        })
    }
    
    fn enable_tx(&self) {
        self.control.modify(|_, w| {
            w.txen(true)
             .txdis(false)
        })
    }
    
    fn reset_rx(&self) {
        self.control.modify(|_, w| {
            w.rxrst(true)
        })
    }
    
    fn reset_tx(&self) {
        self.control.modify(|_, w| {
            w.txrst(true)
        })
    }
    
    pub fn tx_fifo_full(&self) -> bool {
        self.channel_sts.read().txfull()
    }
}
