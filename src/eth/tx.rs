use core::mem::uninitialized;
use crate::{register, register_bit, register_bits, register_bits_typed, regs::*};

/// Descriptor entry
struct DescEntry {
    word0: DescWord0,
    word1: DescWord1,
}

register!(desc_word0, DescWord0, RW, u32);
register_bits!(desc_word0, address, u32, 0, 31);

register!(desc_word1, DescWord1, RW, u32);
register_bits!(desc_word1, length, u16, 0, 13);
register_bit!(desc_word1, last_buffer, 15);
register_bit!(desc_word1, no_crc_append, 16);
register_bits!(desc_word1, csum_offload_errors, u8, 20, 22);
register_bit!(desc_word1, late_collision_tx_error, 26);
register_bit!(desc_word1, ahb_frame_corruption, 27);
register_bit!(desc_word1, retry_limit_exceeded, 29);
/// marks last descriptor in list
register_bit!(desc_word1, wrap, 30);
/// true if owned by software, false if owned by hardware
register_bit!(desc_word1, used, 31);

/// Number of descriptors
pub const DESCS: usize = 8;

#[repr(C)]
pub struct DescList<'a> {
    list: [DescEntry; DESCS],
    buffers: [&'a [u8]; DESCS],
}

impl<'a> DescList<'a> {
    pub fn new(buffers: [&'a [u8]; DESCS]) -> Self {
        let mut list: [DescEntry; DESCS] = unsafe { uninitialized() };
        for i in 0..DESCS {
            let buffer_addr = &buffers[i][0] as *const _ as u32;
            list[i].word0.write(
                DescWord0::zeroed()
                    .address(buffer_addr)
            );
            list[i].word1.write(
                DescWord1::zeroed()
                    .used(true)
                    .wrap(i == DESCS - 1)
            );
        }

        DescList { list, buffers }
    }
}
