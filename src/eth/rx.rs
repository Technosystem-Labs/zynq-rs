use core::ops::Deref;
use crate::{register, register_bit, register_bits, register_bits_typed, regs::*};

/// Descriptor entry
#[repr(C)]
pub struct DescEntry {
    word0: DescWord0,
    word1: DescWord1,
}

register!(desc_word0, DescWord0, RW, u32);
/// true if owned by software, false if owned by hardware
register_bit!(desc_word0, used, 0);
/// mark last desc in list
register_bit!(desc_word0, wrap, 1);
register_bits!(desc_word0, address, u32, 2, 31);

register!(desc_word1, DescWord1, RW, u32);
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
    buffers: &'a mut [[u8; 1536]],
    next: usize,
}

impl<'a> DescList<'a> {
    pub fn new(list: &'a mut [DescEntry], buffers: &'a mut [[u8; 1536]]) -> Self {
        let last = list.len().min(buffers.len()) - 1;
        for (i, (entry, buffer)) in list.iter_mut().zip(buffers.iter_mut()).enumerate() {
            let is_last = i == last;
            assert!(buffer.len() >= 1536);
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

        DescList {
            list,
            buffers,
            next: 0,
        }
    }

    pub fn list_addr(&self) -> u32 {
        &self.list[0] as *const _ as u32
    }

    pub fn recv_next<'s: 'p, 'p>(&'s mut self) -> Option<PktRef<'p>> {
        let list_len = self.list.len();
        let entry = &mut self.list[self.next];
        if entry.word0.read().used() {
            let len = entry.word1.read()
                .frame_length_lsbs().into();
            // TODO: check no split pkt across multiple buffers
            let buffer = &self.buffers[self.next][0..len];

            self.next += 1;
            if self.next >= list_len {
                self.next = 0;
            }

            Some(PktRef { entry, buffer })
        } else {
            None
        }
    }
}

/// Releases a buffer back to the HW upon Drop
pub struct PktRef<'a> {
    entry: &'a mut DescEntry,
    buffer: &'a [u8],
}

impl<'a> Drop for PktRef<'a> {
    fn drop(&mut self) {
        self.entry.word0.modify(|_, w| w.used(false));
    }
}

impl<'a> Deref for PktRef<'a> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.buffer
    }
}
