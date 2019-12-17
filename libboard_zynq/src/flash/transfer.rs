use libregister::{RegisterR, RegisterW, RegisterRW};
use super::regs;
use super::{SpiWord, Flash, Manual};

pub struct Transfer<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> {
    flash: &'a mut Flash<Manual>,
    args: Args,
    sent: usize,
    received: usize,
    len: usize,
}

impl<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> Transfer<'a, Args, W> {
    pub fn new(flash: &'a mut Flash<Manual>, args: Args, len: usize) -> Self {
        flash.regs.config.modify(|_, w| w.pcs(false));
        flash.regs.enable.write(
            regs::Enable::zeroed()
                .spi_en(true)
        );

        let mut xfer = Transfer {
            flash,
            args,
            sent: 0,
            received: 0,
            len,
        };
        xfer.fill_tx_fifo();
        xfer.flash.regs.config.modify(|_, w| w.man_start_com(true));
        xfer
    }

    fn fill_tx_fifo(&mut self) {
        while self.sent < self.len && !self.flash.regs.intr_status.read().tx_fifo_full() {
            let arg = self.args.next()
                .map(|n| n.into())
                .unwrap_or(SpiWord::W32(0));
            match arg {
                SpiWord::W32(w) => {
                    // println!("txd0 {:08X}", w);
                    unsafe {
                        self.flash.regs.txd0.write(w);
                    }
                    self.sent += 4;
                }
                // Only txd0 can be used without flushing
                _ => {
                    if !self.flash.regs.intr_status.read().tx_fifo_not_full() {
                        // Flush if necessary
                        self.flash.regs.config.modify(|_, w| w.man_start_com(true));
                        self.flash.wait_tx_fifo_flush();
                    }

                    match arg {
                        SpiWord::W8(w) => {
                            // println!("txd1 {:02X}", w);
                            unsafe {
                                self.flash.regs.txd1.write(u32::from(w) << 24);
                            }
                            self.sent += 1;
                        }
                        SpiWord::W16(w) => {
                            unsafe {
                                self.flash.regs.txd2.write(u32::from(w) << 16);
                            }
                            self.sent += 2;
                        }
                        SpiWord::W24(w) => {
                            unsafe {
                                self.flash.regs.txd3.write(w << 8);
                            }
                            self.sent += 3;
                        }
                        SpiWord::W32(_) => unreachable!(),
                    }

                    self.flash.regs.config.modify(|_, w| w.man_start_com(true));
                    self.flash.wait_tx_fifo_flush();
                }
            }
        }
    }

    fn can_read(&mut self) -> bool {
        self.flash.regs.intr_status.read().rx_fifo_not_empty()
    }

    fn read(&mut self) -> u32 {
        let rx = self.flash.regs.rx_data.read();
        self.received += 4;
        rx
    }
}

impl<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> Drop for Transfer<'a, Args, W> {
    fn drop(&mut self) {
        // Discard remaining rx_data
        while self.can_read() {
            self.read();
        }

        // Stop
        self.flash.regs.enable.write(
            regs::Enable::zeroed()
                .spi_en(false)
        );
        self.flash.regs.config.modify(|_, w| w
            .pcs(true)
            .man_start_com(false)
        );
    }
}

impl<'a, Args: Iterator<Item = W>, W: Into<SpiWord>> Iterator for Transfer<'a, Args, W> {
    type Item = u32;

    fn next<'s>(&'s mut self) -> Option<u32> {
        if self.received >= self.len {
            return None;
        }

        self.fill_tx_fifo();

        while !self.can_read() {}
        Some(self.read())
    }
}
