use core::ops::Deref;
use vcell::VolatileCell;
use crate::{register, register_bit, register_bits, regs::*};
use super::MTU;

use crate::cortex_a9::asm;

#[derive(Debug)]
pub enum Error {
    HrespNotOk,
    RxOverrun,
    BufferNotAvail,
    Truncated,
}

/// Descriptor entry
#[repr(C, align(0x08))]
pub struct DescEntry {
    word0: DescWord0,
    word1: DescWord1,
}

impl DescEntry {
    pub fn zeroed() -> Self {
        DescEntry {
            word0: DescWord0 { inner: VolatileCell::new(0) },
            word1: DescWord1 { inner: VolatileCell::new(0) },
        }
    }
}

register!(desc_word0, DescWord0, VolatileCell, u32);
register_bit!(desc_word0,
              /// true if owned by software, false if owned by hardware
              used, 0);
register_bit!(desc_word0,
              /// mark last desc in list
              wrap, 1);
register_bits!(desc_word0, address, u32, 2, 31);

register!(desc_word1, DescWord1, VolatileCell, u32);
register_bits!(desc_word1, frame_length_lsbs, u16, 0, 12);
register_bit!(desc_word1, bad_fcs, 13);
register_bit!(desc_word1, start_of_frame, 14);
register_bit!(desc_word1, end_of_frame, 15);
register_bit!(desc_word1, cfi, 16);
register_bits!(desc_word1, vlan_priority, u8, 17, 19);
register_bit!(desc_word1, priority_tag, 20);
register_bit!(desc_word1, vlan_tag, 21);
register_bits!(desc_word1, bits_22_23, u8, 22, 23);
register_bit!(desc_word1, bit_24, 24);
register_bits!(desc_word1, spec_addr_which, u8, 25, 26);
register_bit!(desc_word1, spec_addr_match, 27);
register_bit!(desc_word1, uni_hash_match, 29);
register_bit!(desc_word1, multi_hash_match, 30);
register_bit!(desc_word1, global_broadcast, 31);

#[repr(C)]
pub struct DescList<'a> {
    list: &'a mut [DescEntry],
    buffers: &'a mut [[u8; MTU]],
    next: usize,
}

impl<'a> DescList<'a> {
    pub fn new(list: &'a mut [DescEntry], buffers: &'a mut [[u8; MTU]]) -> Self {
        let last = list.len().min(buffers.len()) - 1;
        for (i, (entry, buffer)) in list.iter_mut().zip(buffers.iter_mut()).enumerate() {
            let is_last = i == last;
            let buffer_addr = &mut buffer[0] as *mut _ as u32;
            assert!(buffer_addr & 0b11 == 0);
            entry.word0.write(
                DescWord0::zeroed()
                    .used(false)
                    .wrap(is_last)
                    .address(buffer_addr >> 2)
            );
            entry.word1.write(
                DescWord1::zeroed()
            );
        }
        // Ensure descriptors get written before they are read.
        asm::dmb();

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

    pub fn recv_next<'s: 'p, 'p>(&'s mut self) -> Result<Option<PktRef<'p>>, Error> {
        let list_len = self.list.len();
        let entry = &mut self.list[self.next];
        if entry.word0.read().used() {
            let word1 = entry.word1.read();
            let len = word1.frame_length_lsbs().into();
            asm::dmb();
            let buffer = &mut self.buffers[self.next][0..len];

            self.next += 1;
            if self.next >= list_len {
                self.next = 0;
            }

            let pkt = PktRef { entry, buffer };
            if word1.start_of_frame() && word1.end_of_frame() {
                Ok(Some(pkt))
            } else {
                Err(Error::Truncated)
            }
        } else {
            Ok(None)
        }
    }
}

/// Releases a buffer back to the HW upon Drop
pub struct PktRef<'a> {
    entry: &'a mut DescEntry,
    buffer: &'a mut [u8],
}

impl<'a> Drop for PktRef<'a> {
    fn drop(&mut self) {
        // Ensure that any buffer reads have finished before we
        // release the buffer to the hardware.
        asm::dmb();
        self.entry.word0.modify(|_, w| w.used(false));
    }
}

impl<'a> Deref for PktRef<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.buffer
    }
}

impl<'a> smoltcp::phy::RxToken for PktRef<'a> {
    fn consume<R, F>(self, _timestamp: smoltcp::time::Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&[u8]) -> smoltcp::Result<R>
    {
        f(self.buffer)
    }
}
