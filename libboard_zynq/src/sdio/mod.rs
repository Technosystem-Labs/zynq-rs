pub mod sd_card;

mod adma;
mod cmd;
mod regs;
use embedded_hal::timer::CountDown;
use libregister::{RegisterR, RegisterRW, RegisterW};
use log::{debug, trace};

use super::{clocks::Clocks, slcr, time::Milliseconds};

/// Basic SDIO Struct with common low-level functions.
pub struct Sdio {
    regs: &'static mut regs::RegisterBlock,
    count_down: super::timer::global::CountDown<Milliseconds>,
    input_clk_hz: u32,
    card_type: CardType,
    card_detect: bool,
}

#[derive(Debug)]
pub enum CmdTransferError {
    CmdInhibited,
    DatLineInhibited,
    CmdTimeout,
    Other(regs::interrupt_status::Read),
}

impl core::fmt::Display for CmdTransferError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use CmdTransferError::*;
        write!(f, "Command transfer error: ")?;
        match self {
            CmdInhibited => write!(f, "Command line inhibited."),
            DatLineInhibited => write!(f, "Data line inhibited, possibly due to ongonging data transfer."),
            CmdTimeout => write!(f, "Command timeout, check if the card is inserted properly."),
            Other(x) => write!(f, "Unknown Error, interrupt status = 0x{:0X}", x.inner),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CardType {
    CardNone,
    CardSd,
    CardMmc,
}

impl Sdio {
    /// Initialize SDIO0
    /// card_detect means if we would use the card detect pin,
    /// false to disable card detection (assume there is card inserted)
    pub fn sdio0(card_detect: bool) -> Self {
        // initialization according to ps7_init.c
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.mio_pin_40.write(
                slcr::MioPin40::zeroed()
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .speed(true),
            );
            slcr.mio_pin_41.write(
                slcr::MioPin41::zeroed()
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .speed(true),
            );
            slcr.mio_pin_42.write(
                slcr::MioPin42::zeroed()
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .speed(true),
            );
            slcr.mio_pin_43.write(
                slcr::MioPin43::zeroed()
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .speed(true),
            );
            slcr.mio_pin_44.write(
                slcr::MioPin44::zeroed()
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .speed(true),
            );
            slcr.mio_pin_45.write(
                slcr::MioPin45::zeroed()
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .speed(true),
            );
            // zc706 card detect pin
            #[cfg(feature = "target_zc706")]
            {
                unsafe {
                    slcr.sd0_wp_cd_sel.write(0x000E000F);
                }
                slcr.mio_pin_14.write(
                    slcr::MioPin14::zeroed()
                        .io_type(slcr::IoBufferType::Lvcmos18)
                        .pullup(true)
                        .tri_enable(true),
                );
            }
            // cora card detect pin
            #[cfg(feature = "target_coraz7")]
            {
                unsafe {
                    slcr.sd0_wp_cd_sel.write(47 << 16);
                }
                slcr.mio_pin_47.write(
                    slcr::MioPin47::zeroed()
                        .io_type(slcr::IoBufferType::Lvcmos18)
                        .speed(true),
                );
            }
            // kasli_soc and redpitaya card detect pin
            #[cfg(any(feature = "target_kasli_soc", feature = "target_redpitaya"))]
            {
                unsafe {
                    slcr.sd0_wp_cd_sel.write(46 << 16);
                }
                slcr.mio_pin_46.write(
                    slcr::MioPin46::zeroed()
                        .io_type(slcr::IoBufferType::Lvcmos25)
                        .speed(true),
                );
            }
            // ebaz4205 card detect pin
            #[cfg(feature = "target_ebaz4205")]
            {
                unsafe {
                    slcr.sd0_wp_cd_sel.write(34 << 16);
                }
                slcr.mio_pin_34.write(
                    slcr::MioPin34::zeroed()
                        .io_type(slcr::IoBufferType::Lvcmos33)
                        .pullup(true)
                        .speed(true),
                );
            }

            slcr.sdio_rst_ctrl.reset_sdio0();
            slcr.aper_clk_ctrl.enable_sdio0();
            slcr.sdio_clk_ctrl.enable_sdio0();
        });
        let clocks = Clocks::get();
        let mut self_ = Sdio {
            regs: regs::RegisterBlock::sdio0(),
            count_down: unsafe { super::timer::GlobalTimer::get() }.countdown(),
            input_clk_hz: clocks.sdio_ref_clk(),
            card_type: CardType::CardNone,
            card_detect,
        };
        self_.init();
        self_
    }

    /// Change clock frequency to the value less than or equal to the given value.
    /// From XSdPs_Change_ClkFreq in xsdps_options.c. SPEC_V3 related code is removed as
    /// our board would only be V1 or V2.
    fn change_clk_freq(&mut self, freq: u32) {
        debug!("Changing clock frequency to {}", freq);
        self.regs
            .clock_control
            .modify(|_, w| w.sd_clk_en(false).internal_clk_en(false));

        const XSDPS_CC_MAX_DIV_CNT: u32 = 256;
        // calculate clock divisor
        let mut div_cnt: u32 = 0x1;
        let mut divisor = 0;
        while div_cnt <= XSDPS_CC_MAX_DIV_CNT {
            if (self.input_clk_hz / div_cnt) <= freq {
                divisor = div_cnt / 2;
                break;
            }
            div_cnt <<= 1;
        }
        if div_cnt > XSDPS_CC_MAX_DIV_CNT {
            panic!("No valid divisor!");
        }
        // enable internal clock
        self.regs
            .clock_control
            .modify(|_, w| w.sdclk_freq_divisor(divisor as u8).internal_clk_en(true));
        while !self.regs.clock_control.read().internal_clk_stable() {}

        // enable SD clock
        self.regs.clock_control.modify(|_, w| w.sd_clk_en(true));
    }

    /// Initialization based on XSdPs_CfgInitialize function in xsdps.c
    fn init(&mut self) {
        // poweroff
        self.regs
            .control
            .modify(|_, w| w.bus_voltage(regs::BusVoltage::V0).bus_power(false));

        if self.regs.misc_reg.read().spec_ver() == regs::SpecificationVersion::V3 {
            // The documentation said the field can only be V1 or V2,
            // so the code is written for V1 and V2. V3 requires special handling
            // which is currently not implemented.
            // I hope that this would never trigger but it is safer to put a check here.
            panic!("The code written is for V1 and V2");
        }
        // delay to poweroff card
        self.delay(1);

        // reset all
        debug!("Reset SDIO!");
        self.regs.clock_control.modify(|_, w| w.software_reset_all(true));
        while self.regs.clock_control.read().software_reset_all() {}

        // set power to 3.3V
        self.regs
            .control
            .modify(|_, w| w.bus_voltage(regs::BusVoltage::V33).bus_power(true));
        // set clock frequency
        self.change_clk_freq(400_000);
        // select voltage
        let capabilities = self.regs.capabilities.read();
        let voltage = if capabilities.voltage_3_3() {
            regs::BusVoltage::V33
        } else if capabilities.voltage_3_0() {
            regs::BusVoltage::V30
        } else if capabilities.voltage_1_8() {
            regs::BusVoltage::V18
        } else {
            regs::BusVoltage::V0
        };
        self.regs.control.modify(|_, w| w.bus_voltage(voltage));

        self.regs.control.modify(|_, w| w.dma_select(regs::DmaSelect::ADMA2_32));

        // enable all interrupt status except card interrupt
        self.regs
            .interrupt_status_en
            .write((regs::interrupt_status_en::Write { inner: 0xFFFFFFFF }).card_interrupt_status_en(false));

        // disable all interrupt signals
        self.regs.interrupt_signal_en.write(regs::InterruptSignalEn::zeroed());

        // set block size to 512 by default
        self.regs
            .block_size_block_count
            .modify(|_, w| w.transfer_block_size(512));
    }

    /// Delay for SDIO operations, simple wrapper for nb.
    pub fn delay(&mut self, ms: u64) {
        self.count_down.start(Milliseconds(ms));
        nb::block!(self.count_down.wait()).unwrap();
    }

    /// Send SD command. Basically `cmd_transfer_with_mode` with mode
    /// `regs::TransferModeCommand::zeroed()`.
    /// Return: Ok if success, Err(status) if failed.
    fn cmd_transfer(&mut self, cmd: cmd::SdCmd, arg: u32, block_cnt: u16) -> Result<(), CmdTransferError> {
        self.cmd_transfer_with_mode(cmd, arg, block_cnt, regs::TransferModeCommand::zeroed())
    }

    /// Send SD Command with additional transfer mode.
    /// This function would block until response is ready.
    /// Return: Ok if success, Err(status) if failed.
    fn cmd_transfer_with_mode(
        &mut self,
        cmd: cmd::SdCmd,
        arg: u32,
        block_cnt: u16,
        transfer_mode: regs::transfer_mode_command::Write,
    ) -> Result<(), CmdTransferError> {
        trace!("Send Cmd {:?}", cmd);
        let state = self.regs.present_state.read();
        if state.command_inhibit_cmd() {
            return Err(CmdTransferError::CmdInhibited);
        }
        self.regs
            .block_size_block_count
            .modify(|_, w| w.blocks_count(block_cnt));
        self.regs.clock_control.modify(|_, w| w.timeout_counter_value(0xE));
        unsafe {
            self.regs.argument.write(arg);
        }
        self.regs
            .interrupt_status_en
            .write(regs::interrupt_status_en::Write { inner: 0xFFFFFFFF });

        let is_sd_card = self.card_type == CardType::CardSd;
        // Check DAT Line
        if cmd != cmd::SdCmd::CMD21 && cmd != cmd::SdCmd::CMD19 {
            if self.regs.present_state.read().command_inhibit_dat() && cmd::require_dat(cmd, is_sd_card) {
                return Err(CmdTransferError::DatLineInhibited);
            }
        }

        // Set the command registers.
        self.regs
            .transfer_mode_command
            .write(cmd::set_cmd_reg(cmd, is_sd_card, transfer_mode));

        // polling for response
        loop {
            let status = self.regs.interrupt_status.read();
            if cmd == cmd::SdCmd::CMD21 || cmd == cmd::SdCmd::CMD19 {
                if status.buffer_read_ready() {
                    self.regs.interrupt_status.modify(|_, w| w.buffer_read_ready());
                    break;
                }
            }
            if status.command_complete() {
                break;
            }
            self.check_error(&status)?;
        }
        // wait for command complete
        while !self.regs.interrupt_status.read().command_complete() {}
        self.regs.interrupt_status.modify(|_, w| w.command_complete());
        Ok(())
    }

    /// Check if card is inserted.
    pub fn is_card_inserted(&self) -> bool {
        !self.card_detect || self.regs.present_state.read().card_inserted()
    }

    /// Switch voltage from 3.3V to 1.8V.
    fn switch_voltage(&mut self) -> Result<(), CmdTransferError> {
        use cmd::SdCmd::*;
        // send switch voltage command
        self.cmd_transfer(CMD11, 0, 0)?;
        // wait for the lines to go low
        let mut state = self.regs.present_state.read();
        while state.cmd_line_level()
            || state.dat0_level()
            || state.dat1_level()
            || state.dat2_level()
            || state.dat3_level()
        {
            state = self.regs.present_state.read();
        }
        // stop the clock
        self.regs
            .clock_control
            .modify(|_, w| w.sd_clk_en(false).internal_clk_en(false));
        // enabling 1.8v in controller
        self.regs.control.modify(|_, w| w.bus_voltage(regs::BusVoltage::V18));

        // wait minimum 5ms
        self.delay(5);

        if self.regs.control.read().bus_voltage() != regs::BusVoltage::V18 {
            // I should not wrap the error of this function into another type later.
            // actually this is not correct.
            return Err(CmdTransferError::CmdTimeout);
        }

        // wait for internal clock to stabilize
        self.regs.clock_control.modify(|_, w| w.internal_clk_en(true));
        while !self.regs.clock_control.read().internal_clk_stable() {}

        // enable SD clock
        self.regs.clock_control.modify(|_, w| w.sd_clk_en(true));

        // wait for 1ms
        self.delay(1);

        // wait for CMD and DATA line to go high
        state = self.regs.present_state.read();
        while !state.cmd_line_level()
            || !state.dat0_level()
            || !state.dat1_level()
            || !state.dat2_level()
            || !state.dat3_level()
        {
            state = self.regs.present_state.read();
        }

        Ok(())
    }

    /// Detect inserted card type, and set the corresponding field.
    /// Return Ok(CardType) on success, Err(CmdTransferError) when failed to identify.
    pub fn identify_card(&mut self) -> Result<CardType, CmdTransferError> {
        use cmd::{SdCmd::*, args::*};
        // actually the delay for this one is unclear in the xilinx code.
        self.delay(10);
        self.cmd_transfer(CMD0, 0, 0)?;

        self.card_type = match self.cmd_transfer(CMD1, ACMD41_HCS | CMD1_HIGH_VOL, 0) {
            Ok(()) => CardType::CardMmc,
            Err(_) => CardType::CardSd,
        };
        // clear all status
        self.regs
            .interrupt_status
            .write(regs::interrupt_status::Write { inner: 0xF3FFFFFF });
        self.regs.clock_control.modify(|_, w| w.software_reset_cmd(true));
        // wait for reset completion
        while self.regs.clock_control.read().software_reset_cmd() {}
        Ok(self.card_type)
    }

    /// Modify transfer block size.
    fn set_block_size(&mut self, block_size: u16) -> Result<(), CmdTransferError> {
        use cmd::SdCmd::*;
        let state = self.regs.present_state.read();
        if state.command_inhibit_cmd()
            || state.command_inhibit_dat()
            || state.write_transfer_active()
            || state.read_transfer_active()
        {
            return Err(CmdTransferError::CmdInhibited);
        }

        debug!("Set block size to {}", block_size);
        // send block write command
        self.cmd_transfer(CMD16, block_size as u32, 0)?;
        // set block size
        self.regs
            .block_size_block_count
            .modify(|_, w| w.transfer_block_size(block_size));
        Ok(())
    }

    /// Check if error occured, and reset the error status.
    /// Return Err(CmdTransferError) if error occured, Ok(()) otherwise.
    fn check_error(&mut self, status: &regs::interrupt_status::Read) -> Result<(), CmdTransferError> {
        if status.error_interrupt() {
            let err_status = if status.inner & 0xFFFE0000 == 0 {
                CmdTransferError::CmdTimeout
            } else {
                CmdTransferError::Other(regs::interrupt_status::Read { inner: status.inner })
            };
            // reset all error status
            self.regs
                .interrupt_status
                .write(regs::interrupt_status::Write { inner: 0xF3FF0000 });
            return Err(err_status);
        }
        Ok(())
    }
}
