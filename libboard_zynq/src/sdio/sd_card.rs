use super::{adma::setup_adma2_descr32, cmd, CardType, CmdTransferError, SDIO};
use libcortex_a9::cache;
use libregister::{RegisterR, RegisterRW, RegisterW};
use log::debug;

#[derive(Debug)]
pub enum CardInitializationError {
    AlreadyInitialized,
    NoCardInserted,
    InitializationFailedOther,
    InitializationFailedCmd(CmdTransferError),
}

impl From<CmdTransferError> for CardInitializationError {
    fn from(error: CmdTransferError) -> Self {
        CardInitializationError::InitializationFailedCmd(error)
    }
}

#[derive(Debug)]
enum CardVersion {
    SdVer1,
    SdVer2,
}

pub struct SdCard {
    sdio: SDIO,
    card_version: CardVersion,
    hcs: bool,
    card_id: [u32; 4],
    rel_card_addr: u32,
    sector_cnt: u32,
    switch_1v8: bool,
    width_4_bit: bool,
}

const BLK_SIZE_MASK: u16 = 0x00000FFF;

impl core::fmt::Display for SdCard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "SdCard: \n  card version: {:?}\n  hcs: {}\n  card id: {:?}\n rel card addr: {}\n sector count: {}",
            self.card_version, self.hcs, self.card_id, self.rel_card_addr, self.sector_cnt)
    }
}

impl SdCard {
    fn sd_card_initialize(&mut self) -> Result<(), CardInitializationError> {
        use cmd::{args::*, SdCmd::*};
        if !self.sdio.is_card_inserted() {
            return Err(CardInitializationError::NoCardInserted);
        }
        // CMD0
        self.sdio.cmd_transfer(CMD0, 0, 0)?;
        match self.sdio.cmd_transfer(CMD8, CMD8_VOL_PATTERN, 0) {
            Err(CmdTransferError::CmdTimeout) => {
                // reset
                self.sdio
                    .regs
                    .clock_control
                    .modify(|_, w| w.software_reset_cmd(true));
                // wait until reset is completed
                while self.sdio.regs.clock_control.read().software_reset_cmd() {}
            }
            // for other error, return initialization failed
            Err(e) => return Err(CardInitializationError::from(e)),
            _ => (),
        }

        self.card_version = if self.sdio.regs.responses[0].read() != CMD8_VOL_PATTERN {
            CardVersion::SdVer1
        } else {
            CardVersion::SdVer2
        };

        // send ACMD41 while card is still busy with power up
        loop {
            self.sdio.cmd_transfer(CMD55, 0, 0)?;
            self.sdio
                .cmd_transfer(ACMD41, ACMD41_HCS | ACMD41_3V3 | (0x1FF << 15), 0)?;

            if (self.sdio.regs.responses[0].read() & RESPOCR_READY) != 0 {
                break;
            }
        }

        let response = self.sdio.regs.responses[0].read();
        // update HCS support flag
        self.hcs = (response & ACMD41_HCS) != 0;
        if (response & OCR_S18) != 0 {
            self.switch_1v8 = true;
            self.sdio.switch_voltage()?;
        }

        self.sdio.cmd_transfer(CMD2, 0, 0)?;
        for i in 0..=3 {
            self.card_id[i] = self.sdio.regs.responses[i].read();
        }

        self.rel_card_addr = 0;
        while self.rel_card_addr == 0 {
            self.sdio.cmd_transfer(CMD3, 0, 0)?;
            self.rel_card_addr = self.sdio.regs.responses[0].read() & 0xFFFF0000;
        }

        self.sdio.cmd_transfer(CMD9, self.rel_card_addr, 0)?;
        self.sdio
            .regs
            .interrupt_status
            .modify(|_, w| w.transfer_complete());

        let mut csd: [u32; 4] = [0, 0, 0, 0];
        for i in 0..=3 {
            csd[i] = self.sdio.regs.responses[i].read();
            debug!("CSD[{}] = {:0X}", i, csd[i]);
        }

        const CSD_STRUCT_MSK: u32 = 0x00C00000;
        const C_SIZE_MULT_MASK: u32 = 0x00000380;
        const C_SIZE_LOWER_MASK: u32 = 0xFFC00000;
        const C_SIZE_UPPER_MASK: u32 = 0x00000003;
        const READ_BLK_LEN_MASK: u32 = 0x00000F00;
        const CSD_V2_C_SIZE_MASK: u32 = 0x3FFFFF00;
        const XSDPS_BLK_SIZE_512_MASK: u32 = 0x200;
        if ((csd[3] & CSD_STRUCT_MSK) >> 22) == 0 {
            let blk_len = 1 << ((csd[2] & READ_BLK_LEN_MASK) >> 8);
            let mult = 1 << (((csd[1] & C_SIZE_MULT_MASK) >> 7) + 2);
            let mut device_size = (csd[1] & C_SIZE_LOWER_MASK) >> 22;
            device_size |= (csd[2] & C_SIZE_UPPER_MASK) << 10;
            device_size = (device_size + 1) * mult;
            device_size = device_size * blk_len;
            self.sector_cnt = device_size / XSDPS_BLK_SIZE_512_MASK;
        } else if ((csd[3] & CSD_STRUCT_MSK) >> 22) == 1 {
            self.sector_cnt = (((csd[1] & CSD_V2_C_SIZE_MASK) >> 8) + 1) * 1024;
        } else {
            return Err(CardInitializationError::InitializationFailedOther);
        }

        self.sdio.change_clk_freq(25_000_000);
        // CMD7: select card
        self.sdio.cmd_transfer(CMD7, self.rel_card_addr, 0)?;

        // pull up
        self.sdio.cmd_transfer(CMD55, self.rel_card_addr, 0)?;
        self.sdio.cmd_transfer(ACMD42, 0, 0)?;

        let mut scr: [u8; 32] = [0; 32];
        self.get_bus_width(&mut scr)?;
        debug!("{:?}", scr);
        if scr[1] & 0x4 != 0 {
            // 4bit support
            debug!("4 bit support");
            self.change_bus_width()?;
        }

        self.sdio.set_block_size(512)?;

        Ok(())
    }

    /// Convert SDIO into SdCard struct, error if no card inserted or it is not an SD card.
    pub fn from_sdio(mut sdio: SDIO) -> Result<Self, CardInitializationError> {
        match sdio.identify_card()? {
            CardType::CardSd => (),
            _ => return Err(CardInitializationError::NoCardInserted),
        };
        let mut _self = SdCard {
            sdio,
            card_version: CardVersion::SdVer1,
            hcs: false,
            card_id: [0, 0, 0, 0],
            rel_card_addr: 0,
            sector_cnt: 0,
            switch_1v8: false,
            width_4_bit: false,
        };
        _self.sd_card_initialize()?;
        Ok(_self)
    }

    /// Convert SdCard struct back to SDIO struct.
    pub fn to_sdio(self) -> SDIO {
        self.sdio
    }

    /// read blocks starting from an address. Each block has length 512 byte.
    /// Note that the address is block address, i.e. 0 for 0~512, 1 for 512~1024, etc.
    pub fn read_block(
        &mut self,
        address: u32,
        block_cnt: u16,
        buffer: &mut [u8],
    ) -> Result<(), CmdTransferError> {
        assert!(buffer.len() >= (block_cnt as usize) * 512);
        // set block size if not set already
        if self
            .sdio
            .regs
            .block_size_block_count
            .read()
            .transfer_block_size()
            != 512
        {
            self.sdio.set_block_size(512)?;
        }

        setup_adma2_descr32(&mut self.sdio, block_cnt as u32, buffer);
        // invalidate D cache, required for ZC706, not sure for Cora Z7 10
        cache::dcci_slice(buffer);

        let cmd = if block_cnt == 1 {
            cmd::SdCmd::CMD17
        } else {
            cmd::SdCmd::CMD18
        };

        let mode = if block_cnt == 1 {
            super::regs::TransferModeCommand::zeroed()
                .block_count_en(true)
                .direction_select(true)
                .dma_en(true)
        } else {
            super::regs::TransferModeCommand::zeroed()
                .auto_cmd12_en(true)
                .block_count_en(true)
                .direction_select(true)
                .multi_block_en(true)
                .dma_en(true)
        };

        self.sdio
            .cmd_transfer_with_mode(cmd, address, block_cnt, mode)?;

        self.wait_transfer_complete()?;
        cache::dcci_slice(buffer);
        Ok(())
    }

    /// write blocks starting from an address. Each block has length 512 byte.
    /// Note that the address is block address, i.e. 0 for 0~512, 1 for 512~1024, etc.
    pub fn write_block(
        &mut self,
        address: u32,
        block_cnt: u16,
        buffer: &mut [u8],
    ) -> Result<(), CmdTransferError> {
        assert!(buffer.len() >= (block_cnt as usize) * 512);
        // set block size if not set already
        if self
            .sdio
            .regs
            .block_size_block_count
            .read()
            .transfer_block_size()
            != 512
        {
            self.sdio.set_block_size(512)?;
        }

        setup_adma2_descr32(&mut self.sdio, block_cnt as u32, buffer);
        // invalidate D cache, required for ZC706, not sure for Cora Z7 10
        cache::dcci_slice(buffer);

        let cmd = if block_cnt == 1 {
            cmd::SdCmd::CMD24
        } else {
            cmd::SdCmd::CMD25
        };

        let mode = if block_cnt == 1 {
            super::regs::TransferModeCommand::zeroed()
                .block_count_en(true)
                .dma_en(true)
        } else {
            super::regs::TransferModeCommand::zeroed()
                .auto_cmd12_en(true)
                .block_count_en(true)
                .multi_block_en(true)
                .dma_en(true)
        };

        self.sdio
            .cmd_transfer_with_mode(cmd, address, block_cnt, mode)?;
        // wait for transfer complete interrupt
        self.wait_transfer_complete()?;

        cache::dcci_slice(buffer);
        Ok(())
    }

    fn get_bus_width(&mut self, buf: &mut [u8]) -> Result<(), CmdTransferError> {
        use cmd::SdCmd::*;
        debug!("Getting bus width");
        for i in 0..8 {
            buf[i] = 0;
        }
        // send block write command
        self.sdio.cmd_transfer(CMD55, self.rel_card_addr, 0)?;

        let blk_cnt: u16 = 1;
        let blk_size: u16 = 8 & BLK_SIZE_MASK;
        self.sdio
            .regs
            .block_size_block_count
            .modify(|_, w| w.transfer_block_size(blk_size));

        setup_adma2_descr32(&mut self.sdio, blk_cnt as u32, buf);
        cache::dcci_slice(buf);
        self.sdio.cmd_transfer_with_mode(
            ACMD51,
            0,
            blk_cnt,
            super::regs::TransferModeCommand::zeroed()
                .dma_en(true)
                .direction_select(true),
        )?;

        self.wait_transfer_complete()?;
        cache::dcci_slice(buf);
        Ok(())
    }

    fn change_bus_width(&mut self) -> Result<(), CmdTransferError> {
        use cmd::SdCmd::*;
        debug!("Changing bus speed");
        self.sdio.cmd_transfer(CMD55, self.rel_card_addr, 0)?;
        self.width_4_bit = true;
        self.sdio.cmd_transfer(ACMD6, 0x2, 0)?;
        self.sdio.delay(1);
        self.sdio
            .regs
            .control
            .modify(|_, w| w.data_width_select(true));
        Ok(())
    }

    fn wait_transfer_complete(&mut self) -> Result<(), CmdTransferError> {
        debug!("Wait for transfer complete");
        let mut status = self.sdio.regs.interrupt_status.read();
        while !status.transfer_complete() {
            self.sdio.check_error(&status)?;
            status = self.sdio.regs.interrupt_status.read();
        }
        debug!("Clearing transfer complete");
        self.sdio
            .regs
            .interrupt_status
            .modify(|_, w| w.transfer_complete());
        Ok(())
    }
}
