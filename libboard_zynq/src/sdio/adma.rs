/// ADMA library
use super::SDIO;
use libcortex_a9::cache;
use libregister::RegisterR;

#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct Adma2Desc32 {
    attribute: u16,
    length: u16,
    address: u32,
}

// Default::default() cannot be used as it is not a constant function...
static mut ADMA2_DESCR32_TABLE: [Adma2Desc32; 32] = [Adma2Desc32 {
    attribute: 0,
    length: 0,
    address: 0,
}; 32];

#[allow(unused)]
const DESC_MAX_LENGTH: u32 = 65536;
#[allow(unused)]
const DESC_TRANS: u16 = 0x2 << 4;
#[allow(unused)]
const DESC_INT: u16 = 0x1 << 2;
#[allow(unused)]
const DESC_END: u16 = 0x1 << 1;
#[allow(unused)]
const DESC_VALID: u16 = 0x1 << 0;

#[allow(unused)]
impl Adma2Desc32 {
    pub fn set_attribute(&mut self, attribute: u16) {
        unsafe {
            core::ptr::write_volatile(&mut self.attribute as *mut u16, attribute);
        }
    }

    pub fn set_length(&mut self, length: u16) {
        unsafe {
            core::ptr::write_volatile(&mut self.length as *mut u16, length);
        }
    }

    pub fn set_address(&mut self, address: u32) {
        unsafe {
            core::ptr::write_volatile(&mut self.address as *mut u32, address);
        }
    }
}

pub fn setup_adma2_descr32(sdio: &mut SDIO, blk_cnt: u32, buffer: &[u8]) {
    let descr_table = unsafe { &mut ADMA2_DESCR32_TABLE };
    let blk_size = sdio
        .regs
        .block_size_block_count
        .read()
        .transfer_block_size() as u32;

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
        descr_table[desc_num].set_address(ptr + (desc_num as u32) * DESC_MAX_LENGTH);
        descr_table[desc_num].set_attribute(DESC_TRANS | DESC_VALID);
        // 0 is the max length (65536)
        descr_table[desc_num].set_length(0);
    }
    descr_table[total_desc_lines - 1].set_attribute(DESC_TRANS | DESC_VALID | DESC_END);
    descr_table[total_desc_lines - 1].set_length(
        (blk_cnt * blk_size - ((total_desc_lines as u32) - 1) * DESC_MAX_LENGTH) as u16,
    );
    unsafe {
        sdio.regs
            .adma_system_address
            .write(descr_table.as_ptr() as u32);
    }
    cache::dcci_slice(descr_table);
}
