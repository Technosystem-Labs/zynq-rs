use volatile_register::{RO, WO, RW};
use bit_field::BitField;

#[repr(packed)]
pub struct RegisterBlock {
    pub control: RW<u32>,
    pub mode: RW<u32>,
    pub intrpt_en: RW<u32>,
    pub intrpt_dis: RW<u32>,
    pub intrpt_mask: RO<u32>,
    pub chnl_int_sts: WO<u32>,
    pub baud_rate_gen: RW<u32>,
    pub rcvr_timeout: RW<u32>,
    pub rcvr_fifo_trigger_level: RW<u32>,
    pub modem_ctrl: RW<u32>,
    pub modem_sts: RW<u32>,
    pub channel_sts: RO<u32>,
    pub tx_rx_fifo: RW<u32>,
    pub baud_rate_divider: RW<u32>,
    pub flow_delay: RW<u32>,
    pub reserved0: RO<u32>,
    pub reserved1: RO<u32>,
    pub tx_fifo_trigger_level: RW<u32>,
}

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
        unsafe {
            // Confiugre UART character frame
            // * Disable clock-divider
            // * 8-bit
            // * no parity
            // * 1 stop bit
            // * Normal channel mode
            self.mode.write(0x20);

            // Configure the Buad Rate
            self.disable_rx();
            self.disable_tx();

            // 9,600 baud
            self.baud_rate_gen.write(651);
            self.baud_rate_divider.write(7);

            self.reset_rx();
            self.reset_tx();
            self.enable_rx();
            self.enable_tx();
        }
    }

    pub unsafe fn write_byte(&self, value: u8) {
        self.tx_rx_fifo.write(value.into());
    }

    const CONTROL_RXEN: usize = 2;
    const CONTROL_RXDIS: usize = 3;
    const CONTROL_TXEN: usize = 4;
    const CONTROL_TXDIS: usize = 5;
    const CONTROL_TXRST: usize = 1;
    const CONTROL_RXRST: usize = 0;

    unsafe fn disable_rx(&self) {
        self.control.modify(|mut x| {
            x.set_bit(Self::CONTROL_RXEN, false);
            x.set_bit(Self::CONTROL_RXDIS, true);
            x
        })
    }
    
    unsafe fn disable_tx(&self) {
        self.control.modify(|mut x| {
            x.set_bit(Self::CONTROL_TXEN, false);
            x.set_bit(Self::CONTROL_TXDIS, true);
            x
        })
    }
    
    unsafe fn enable_rx(&self) {
        self.control.modify(|mut x| {
            x.set_bit(Self::CONTROL_RXEN, true);
            x.set_bit(Self::CONTROL_RXDIS, false);
            x
        })
    }
    
    unsafe fn enable_tx(&self) {
        self.control.modify(|mut x| {
            x.set_bit(Self::CONTROL_TXEN, true);
            x.set_bit(Self::CONTROL_TXDIS, false);
            x
        })
    }
    
    unsafe fn reset_rx(&self) {
        self.control.modify(|mut x| {
            x.set_bit(Self::CONTROL_RXRST, true);
            x
        })
    }
    
    unsafe fn reset_tx(&self) {
        self.control.modify(|mut x| {
            x.set_bit(Self::CONTROL_TXRST, true);
            x
        })
    }

    const CHANNEL_STS_TXFULL: usize = 4;
    
    pub fn tx_fifo_full(&self) -> bool {
        self.channel_sts.read().get_bit(Self::CHANNEL_STS_TXFULL)
    }
}
