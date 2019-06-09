use core::mem::uninitialized;
use crate::{register, register_bit, register_bits, register_bits_typed, regs::*};

/// Descriptor entry
#[repr(C)]
struct DescEntry {
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

/// Number of descriptors
pub const DESCS: usize = 8;

#[repr(C)]
pub struct DescList<'a> {
    list: [DescEntry; DESCS],
    buffers: [&'a mut [u8]; DESCS],
}

impl<'a> DescList<'a> {
    pub fn new(buffers: [&'a mut [u8]; DESCS]) -> Self {
        let mut list: [DescEntry; DESCS] = unsafe { uninitialized() };
        for i in 0..DESCS {
            assert!(buffers[i].len() >= 1536);
            let buffer_addr = &mut buffers[i][0] as *mut _ as u32;
            assert!(buffer_addr & 0b11 == 0);
            list[i].word0.write(
                DescWord0::zeroed()
                    .used(false)
                    .wrap(i == DESCS - 1)
                    .address(buffer_addr >> 2)
            );
            list[i].word1.write(
                DescWord1::zeroed()
            );
        }

        DescList { list, buffers }
    }
}
