use super::regs;

const APP_CMD_PREFIX: u8 = 0x80;
#[allow(unused)]
pub mod args {
    pub const CMD8_VOL_PATTERN: u32 = 0x1AA;
    pub const RESPOCR_READY: u32 = 0x80000000;
    pub const ACMD41_HCS: u32 = 0x40000000;
    pub const ACMD41_3V3: u32 = 0x00300000;
    pub const CMD1_HIGH_VOL: u32 = 0x00FF8000;
    pub const OCR_S18: u32 = 1 << 24;
}

#[allow(unused)]
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SdCmd {
    CMD0 = 0x00,
    CMD1 = 0x01,
    CMD2 = 0x02,
    CMD3 = 0x03,
    CMD4 = 0x04,
    CMD5 = 0x05,
    CMD6 = 0x06,
    ACMD6 = APP_CMD_PREFIX + 0x06,
    CMD7 = 0x07,
    CMD8 = 0x08,
    CMD9 = 0x09,
    CMD10 = 0x0A,
    CMD11 = 0x0B,
    CMD12 = 0x0C,
    ACMD13 = APP_CMD_PREFIX + 0x0D,
    CMD16 = 0x10,
    CMD17 = 0x11,
    CMD18 = 0x12,
    CMD19 = 0x13,
    CMD21 = 0x15,
    CMD23 = 0x17,
    ACMD23 = APP_CMD_PREFIX + 0x17,
    CMD24 = 0x18,
    CMD25 = 0x19,
    CMD41 = 0x29,
    ACMD41 = APP_CMD_PREFIX + 0x29,
    ACMD42 = APP_CMD_PREFIX + 0x2A,
    ACMD51 = APP_CMD_PREFIX + 0x33,
    CMD52 = 0x34,
    CMD55 = 0x37,
    CMD58 = 0x3A,
}

pub fn require_dat(cmd: SdCmd, is_sd_card: bool) -> bool {
    use SdCmd::*;
    match cmd {
        CMD6 => is_sd_card,
        CMD8 => !is_sd_card,
        ACMD13 | CMD17 | CMD18 | CMD19 | CMD21 | CMD23 | ACMD23 | CMD24 | CMD25 | ACMD51 => true,
        _ => false,
    }
}

type CmdReg = regs::transfer_mode_command::Write;

fn resp_r1(w: CmdReg) -> CmdReg {
    w.response_type_select(regs::ResponseTypeSelect::Length48)
        .crc_check_en(true)
        .index_check_en(true)
}

fn resp_r1b(w: CmdReg) -> CmdReg {
    w.response_type_select(regs::ResponseTypeSelect::Legnth48Check)
        .crc_check_en(true)
        .index_check_en(true)
}

fn resp_r2(w: CmdReg) -> CmdReg {
    w.response_type_select(regs::ResponseTypeSelect::Length136)
        .crc_check_en(true)
}

fn resp_r3(w: CmdReg) -> CmdReg {
    w.response_type_select(regs::ResponseTypeSelect::Length48)
}

fn resp_r6(w: CmdReg) -> CmdReg {
    w.response_type_select(regs::ResponseTypeSelect::Legnth48Check)
        .crc_check_en(true)
        .index_check_en(true)
}

pub fn set_cmd_reg(cmd: SdCmd, is_sd_card: bool, w: CmdReg) -> CmdReg {
    use SdCmd::*;
    let w = w.command_index(cmd as u8 & 0x3F);
    match cmd {
        CMD1 => resp_r3(w),
        CMD2 => resp_r2(w),
        CMD3 => {
            if is_sd_card {
                resp_r6(w)
            } else {
                resp_r1(w)
            }
        }
        CMD5 => resp_r1b(w),
        CMD6 => {
            if is_sd_card {
                resp_r1(w).data_present_select(true)
            } else {
                resp_r1b(w)
            }
        }
        ACMD6 => resp_r1(w),
        CMD7 => resp_r1(w),
        CMD8 => {
            if is_sd_card {
                resp_r1(w)
            } else {
                resp_r1(w).data_present_select(true)
            }
        }
        CMD9 => resp_r2(w),
        CMD10 | CMD11 | CMD12 => resp_r1(w),
        ACMD13 => resp_r1(w).data_present_select(true),
        CMD16 => resp_r1(w),
        CMD17 | CMD18 | CMD19 | CMD21 | CMD23 | ACMD23 | CMD24 | CMD25 => resp_r1(w).data_present_select(true),
        ACMD41 => resp_r3(w),
        ACMD42 => resp_r1(w),
        ACMD51 => resp_r1(w).data_present_select(true),
        CMD52 | CMD55 => resp_r1(w),
        _ => w,
    }
}
