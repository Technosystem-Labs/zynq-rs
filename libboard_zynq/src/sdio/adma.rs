/// ADMA library
use core::mem::MaybeUninit;

use libcortex_a9::cache;
use libregister::{RegisterR, RegisterRW, RegisterW, VolatileCell, register, register_bit};

use super::Sdio;

#[repr(C, align(4))]
pub struct Adma2Desc32 {
    attribute: Desc32Attribute,
    length: VolatileCell<u16>,
    address: VolatileCell<u32>,
}

const DESC_MAX_LENGTH: u32 = 65536;

register!(desc32_attribute, Desc32Attribute, VolatileCell, u16);
register_bit!(desc32_attribute, trans, 5);
register_bit!(desc32_attribute, int, 2);
register_bit!(desc32_attribute, end, 1);
register_bit!(desc32_attribute, valid, 0);

pub struct Adma2DescTable([Adma2Desc32; 32]);

impl Adma2DescTable {
    pub fn new() -> Self {
        let table = MaybeUninit::zeroed();
        let table = unsafe { table.assume_init() };
        Adma2DescTable(table)
    }

    /// Initialize the table and setup `adma_system_address`
    pub fn setup(&mut self, sdio: &mut Sdio, blk_cnt: u32, buffer: &[u8]) {
        let descr_table = &mut self.0;
        let blk_size = sdio.regs.block_size_block_count.read().transfer_block_size() as u32;

        let total_desc_lines = if blk_size * blk_cnt < DESC_MAX_LENGTH {
            1
        } else {
            blk_size * blk_cnt / DESC_MAX_LENGTH
                + if (blk_size * blk_cnt) % DESC_MAX_LENGTH == 0 {
                    0
                } else {
                    1
                }
        } as usize;

        let ptr = buffer.as_ptr() as u32;
        for desc_num in 0..total_desc_lines {
            descr_table[desc_num]
                .address
                .set(ptr + (desc_num as u32) * DESC_MAX_LENGTH);
            descr_table[desc_num]
                .attribute
                .write(Desc32Attribute::zeroed().trans(true).valid(true));
            // 0 is the max length (65536)
            descr_table[desc_num].length.set(0);
        }
        descr_table[total_desc_lines - 1].attribute.modify(|_, w| w.end(true));
        descr_table[total_desc_lines - 1]
            .length
            .set((blk_cnt * blk_size - ((total_desc_lines as u32) - 1) * DESC_MAX_LENGTH) as u16);
        unsafe {
            sdio.regs.adma_system_address.write(descr_table.as_ptr() as u32);
        }
        cache::dcci_slice(descr_table);
    }
}
