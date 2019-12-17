use core::ops::{Deref, DerefMut};
use libregister::*;
use super::{MTU, regs};

/// Descriptor entry
#[repr(C, align(0x08))]
pub struct DescEntry {
    word0: DescWord0,
    word1: DescWord1,
}

register!(desc_word0, DescWord0, VolatileCell, u32);
register_bits!(desc_word0, address, u32, 0, 31);

register!(desc_word1, DescWord1, VolatileCell, u32);
register_bits!(desc_word1, length, u16, 0, 13);
register_bit!(desc_word1, last_buffer, 15);
register_bit!(desc_word1, no_crc_append, 16);
register_bits!(desc_word1, csum_offload_errors, u8, 20, 22);
register_bit!(desc_word1, late_collision_tx_error, 26);
register_bit!(desc_word1, ahb_frame_corruption, 27);
register_bit!(desc_word1, retry_limit_exceeded, 29);
register_bit!(desc_word1,
              /// marks last descriptor in list
              wrap, 30);
register_bit!(desc_word1,
              /// true if owned by software, false if owned by hardware
              used, 31);

impl DescEntry {
    pub fn zeroed() -> Self {
        DescEntry {
            word0: DescWord0 { inner: VolatileCell::new(0) },
            word1: DescWord1 { inner: VolatileCell::new(0) },
        }
    }
}

/// Number of descriptors
pub const DESCS: usize = 8;

#[repr(C)]
pub struct DescList<'a> {
    list: &'a mut [DescEntry],
    buffers: &'a mut [[u8; MTU]],
    next: usize,
}

impl<'a> DescList<'a> {
    pub fn new(list: &'a mut [DescEntry], buffers: &'a mut [[u8; MTU]]) -> Self {
        let last = list.len().min(buffers.len()) - 1;
        // Sending seems to not work properly with only one packet
        // buffer (two duplicates get send with every packet), so
        // check that at least 2 are allocated, i.e. that the index of
        // the last one is at least one.
        assert!(last > 0);

        for (i, (entry, buffer)) in list.iter_mut().zip(buffers.iter_mut()).enumerate() {
            let is_last = i == last;
            let buffer_addr = &mut buffer[0] as *mut _ as u32;
            assert!(buffer_addr & 0b11 == 0);
            entry.word0.write(
                DescWord0::zeroed()
                    .address(buffer_addr)
            );
            entry.word1.write(
                DescWord1::zeroed()
                    .used(true)
                    .wrap(is_last)
                    // every frame contains 1 packet
                    .last_buffer(true)
            );
        }

        DescList {
            // Shorten the list of descriptors to the required number.
            list: &mut list[0..=last],
            buffers,
            next: 0,
        }
    }

    pub fn list_addr(&self) -> u32 {
        &self.list[0] as *const _ as u32
    }

    pub fn send<'s: 'p, 'p>(&'s mut self, regs: &'s mut regs::RegisterBlock, length: usize) -> Option<PktRef<'p>> {
        let list_len = self.list.len();
        let entry = &mut self.list[self.next];
        if entry.word1.read().used() {
            let buffer = &mut self.buffers[self.next][0..length];
            entry.word1.write(DescWord1::zeroed()
                .length(length as u16)
                .last_buffer(true)
                .wrap(self.next >= list_len - 1)
                .used(true)
            );

            self.next += 1;
            if self.next >= list_len {
                self.next = 0;
            }

            Some(PktRef { entry, buffer, regs })
        } else {
            // Still in use by HW (sending too fast, ring exceeded)
            None
        }
    }
}

/// Releases a buffer back to the HW upon Drop, and start the TX
/// engine
pub struct PktRef<'a> {
    entry: &'a mut DescEntry,
    buffer: &'a mut [u8],
    regs: &'a mut regs::RegisterBlock,
}

impl<'a> Drop for PktRef<'a> {
    fn drop(&mut self) {
        self.entry.word1.modify(|_, w| w.used(false));
        if ! self.regs.tx_status.read().tx_go() {
            self.regs.net_ctrl.modify(|_, w|
                                      w.start_tx(true)
            );
        }
    }
}

impl<'a> Deref for PktRef<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.buffer
    }
}

impl<'a> DerefMut for PktRef<'a> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        self.buffer
    }
}

/// TxToken for smoltcp support
pub struct Token<'a, 'tx: 'a> {
    pub regs: &'a mut regs::RegisterBlock,
    pub desc_list: &'a mut DescList<'tx>,
}

impl<'a, 'tx: 'a> smoltcp::phy::TxToken for Token<'a, 'tx> {
    fn consume<R, F>(self, _timestamp: smoltcp::time::Instant, len: usize, f: F) -> smoltcp::Result<R>
        where F: FnOnce(&mut [u8]) -> smoltcp::Result<R>
    {
        match self.desc_list.send(self.regs, len) {
            None =>
                Err(smoltcp::Error::Exhausted),
            Some(mut pktref) => {
                let result = f(pktref.deref_mut());
                // TODO: on result.is_err() don;t send
                drop(pktref);
                result
            }
        }
    }
}
