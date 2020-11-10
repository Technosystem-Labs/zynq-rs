use libregister::{RegisterR, RegisterW, RegisterRW};
use log::{debug, info, error};
use crate::{print, println};
use super::slcr::{self, DdriobVrefSel};
use super::clocks::{Clocks, source::{DdrPll, ClockSource}};

#[cfg(feature = "target_redpitaya")]
use super::ps7_init;

mod regs;

#[cfg(feature = "target_zc706")]
/// Micron MT41J256M8HX-15E: 667 MHz DDR3
const DDR_FREQ: u32 = 666_666_666;

#[cfg(feature = "target_cora_z7_10")]
/// Micron MT41K256M16HA-125: 800 MHz DDR3L, max supported 533 MHz
const DDR_FREQ: u32 = 525_000_000;

#[cfg(feature = "target_redpitaya")]
/// Alliance Memory AS4C256M16D3B: 800 MHz DDR3
const DDR_FREQ: u32 = 800_000_000;

/// MT41K256M16HA-125
const DCI_FREQ: u32 = 10_000_000;

pub struct DdrRam {
    regs: &'static mut regs::RegisterBlock,
}

impl DdrRam {
    pub fn ddrram() -> Self {
        if cfg!(feature = "target_redpitaya") {
            // We have not yet fixed red pitaya initialization yet.  It seems
            // that the clock configuration, iob settings and ddr settings are
            // all problematic
            #[cfg(feature = "target_redpitaya")]
            ps7_init::apply();
            let regs = regs::RegisterBlock::ddrc();
            DdrRam { regs }
        } else {
            let clocks = Self::clock_setup();
            Self::configure_iob();
            Self::calibrate_iob_impedance(&clocks);
            let regs = regs::RegisterBlock::ddrc();
            let mut ddr = DdrRam { regs };
            ddr.reset_ddrc(|ddr| ddr.configure());
            ddr
        }
    }

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 10.6.1 DDR Clock Initialization
    fn clock_setup() -> Clocks {
        DdrPll::setup(2 * DDR_FREQ);

        let clocks = Clocks::get();
        let ddr3x_clk_divisor = 2;
        let ddr2x_clk_divisor = 3;
        debug!("DDR 3x/2x clocks: {}/{}", clocks.ddr / u32::from(ddr3x_clk_divisor), clocks.ddr / u32::from(ddr2x_clk_divisor));

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.ddr_clk_ctrl.write(
                slcr::DdrClkCtrl::zeroed()
                    .ddr_2xclkact(true)
                    .ddr_3xclkact(true)
                    .ddr_2xclk_divisor(ddr2x_clk_divisor)
                    .ddr_3xclk_divisor(ddr3x_clk_divisor)
            );
        });
        clocks
    }

    fn calculate_dci_divisors(clocks: &Clocks) -> (u8, u8) {
        let target = (DCI_FREQ - 1 + clocks.ddr) / DCI_FREQ;

        let mut best = None;
        let mut best_error = 0;
        for divisor0 in 1..63 {
            for divisor1 in 1..63 {
                let current = (divisor0 as u32) * (divisor1 as u32);
                let error = if current > target {
                    current - target
                } else {
                    target - current
                };
                if best.is_none() || best_error > error {
                    best = Some((divisor0, divisor1));
                    best_error = error;
                }
            }
        }
        best.unwrap()
    }

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 10.6.2 DDR IOB Impedance Calibration
    fn calibrate_iob_impedance(clocks: &Clocks) {
        let (divisor0, divisor1) = Self::calculate_dci_divisors(clocks);
        debug!("DDR DCI clock: {} Hz (divisors={}*{})",
               clocks.ddr / u32::from(divisor0) / u32::from(divisor1),
               divisor0, divisor1);

        slcr::RegisterBlock::unlocked(|slcr| {
            // Step 1.
            slcr.dci_clk_ctrl.write(
                slcr::DciClkCtrl::zeroed()
                    .clkact(true)
                    .divisor0(divisor0)
                    .divisor1(divisor1)
            );

            // Step 2.a.
            slcr.ddriob_dci_ctrl.modify(|_, w|
                w.reset(false)
            );
            slcr.ddriob_dci_ctrl.modify(|_, w|
                w.reset(true)
            );
            // Step 3.b. for DDR3/DDR3L
            slcr.ddriob_dci_ctrl.modify(|_, w|
                w.nref_opt1(0)
                 .nref_opt2(0)
                 .nref_opt4(1)
                 .pref_opt1(0)
                 .pref_opt2(0)
            );
            // Step 2.c.
            slcr.ddriob_dci_ctrl.modify(|_, w|
                w.update_control(false)
            );
            // Step 2.d.
            slcr.ddriob_dci_ctrl.modify(|_, w|
                w.enable(true)
            );
            // Step 2.e.
            while ! slcr.ddriob_dci_status.read().done() {}
        });
    }

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 10.6.3 DDR IOB Configuration
    fn configure_iob() {
        slcr::RegisterBlock::unlocked(|slcr| {
            let addr_config = slcr::DdriobConfig::zeroed()
                .output_en(slcr::DdriobOutputEn::Obuf);
            slcr.ddriob_addr0.write(addr_config.clone());
            slcr.ddriob_addr1.write(addr_config);

            #[cfg(feature = "target_zc706")]
            let data0_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::VrefDifferential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            #[cfg(feature = "target_zc706")]
            let data1_config = data0_config.clone();
            #[cfg(feature = "target_cora_z7_10")]
            let data0_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::VrefDifferential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            #[cfg(feature = "target_cora_z7_10")]
            let data1_config = slcr::DdriobConfig::zeroed()
                .pullup_en(true);
            #[cfg(feature = "target_redpitaya")]
            let data0_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::VrefDifferential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            #[cfg(feature = "target_redpitaya")]
            let data1_config = slcr::DdriobConfig::zeroed()
                .pullup_en(true);
            slcr.ddriob_data0.write(data0_config);
            slcr.ddriob_data1.write(data1_config);

            #[cfg(feature = "target_zc706")]
            let diff0_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::Differential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            #[cfg(feature = "target_zc706")]
            let diff1_config = diff0_config.clone();
            #[cfg(feature = "target_cora_z7_10")]
            let diff0_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::Differential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            #[cfg(feature = "target_cora_z7_10")]
            let diff1_config = slcr::DdriobConfig::zeroed()
                .pullup_en(true);
            #[cfg(feature = "target_redpitaya")]
            let diff0_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::Differential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            #[cfg(feature = "target_redpitaya")]
            let diff1_config = slcr::DdriobConfig::zeroed()
                .pullup_en(true);
            slcr.ddriob_diff0.write(diff0_config);
            slcr.ddriob_diff1.write(diff1_config);

            slcr.ddriob_clock.write(
                slcr::DdriobConfig::zeroed()
                    .output_en(slcr::DdriobOutputEn::Obuf)
            );

            unsafe {
                // Not documented in Technical Reference Manual
                slcr.ddriob_drive_slew_addr.write(0x0018C61C);
                slcr.ddriob_drive_slew_data.write(0x00F9861C);
                slcr.ddriob_drive_slew_diff.write(0x00F9861C);
                slcr.ddriob_drive_slew_clock.write(0x00F9861C);
            }

            #[cfg(feature = "target_cora_z7_10")]
            slcr.ddriob_ddr_ctrl.modify(|_, w| w
                    .vref_int_en(false)
                    .vref_ext_en_lower(true)
                    .vref_ext_en_upper(false)
                    .refio_en(true)
            );
            #[cfg(feature = "target_zc706")]
            slcr.ddriob_ddr_ctrl.modify(|_, w| w
                    .vref_int_en(true)
                    .vref_sel(DdriobVrefSel::Vref0_75V)
                    .vref_ext_en_lower(false)
                    .vref_ext_en_upper(false)
            );
            #[cfg(feature = "target_redpitaya")]
            slcr.ddriob_ddr_ctrl.modify(|_, w| w
                    .vref_int_en(false)
                    .vref_ext_en_lower(true)
                    .vref_ext_en_upper(false)
            );
        });
    }

    fn configure(&mut self) {
        #[cfg(feature = "target_cora_z7_10")]
        self.regs.dram_param0.write(
            regs::DramParam0::zeroed()
                .t_rc(0x1a)
                .t_rfc_min(0x9e)
                .post_selfref_gap_x32(0x10)
        );
        #[cfg(feature = "target_zc706")]
        self.regs.dram_param0.write(
            regs::DramParam0::zeroed()
                .t_rc(0x1b)
                .t_rfc_min(0x56)
                .post_selfref_gap_x32(0x10)
        );

        self.regs.dram_param2.write(
            regs::DramParam2::zeroed()
                .write_latency(0x5)
                .rd2wr(0x7)
                .wr2rd(0xe)
                .t_xp(0x4)
                .pad_pd(0x0)
                .rd2pre(0x4)
                .t_rcd(0x7)
        );

        self.regs.dram_emr_mr.write(
            regs::DramEmrMr::zeroed()
                .mr(0x930)
                .emr(0x4)
        );

        #[cfg(feature = "target_cora_z7_10")]
        self.regs.phy_config2.modify(
            |_, w| w.data_slice_in_use(false)
        );
        #[cfg(feature = "target_cora_z7_10")]
        self.regs.phy_config3.modify(
            |_, w| w.data_slice_in_use(false)
        );

        self.regs.phy_cmd_timeout_rddata_cpt.modify(
            |_, w| w
                .rd_cmd_to_data(0x0)
                .wr_cmd_to_data(0x0)
                .we_to_re_delay(0x8)
                .rdc_fifo_rst_disable(false)
                .use_fixed_re(true)
                .rdc_fifo_rst_err_cnt_clr(false)
                .dis_phy_ctrl_rstn(false)
                .clk_stall_level(false)
                .gatelvl_num_of_dq0(0x7)
                .wrlvl_num_of_dq0(0x7)
        );

        self.regs.reg_2c.write(
            regs::Reg2C::zeroed()
                .wrlvl_max_x1024(0xfff)
                .rdlvl_max_x1024(0xfff)
                .twrlvl_max_error(false)
                .trdlvl_max_error(false)
                .dfi_wr_level_en(true)
                .dfi_rd_dqs_gate_level(true)
                .dfi_rd_data_eye_train(true)
        );

        self.regs.dfi_timing.write(
            regs::DfiTiming::zeroed()
                .rddata_en(0x6)
                .ctrlup_min(0x3)
                .ctrlup_max(0x40)
        );

        #[cfg(feature = "target_zc706")]
        self.regs.phy_init_ratio3.write(
            regs::PhyInitRatio::zeroed()
                .wrlvl_init_ratio(0x21)
                .gatelvl_init_ratio(0xee)
        );

        #[cfg(feature = "target_cora_z7_10")]
        self.regs.reg_64.modify(
            |_, w| w
                .phy_ctrl_slave_ratio(0x100)
                .phy_invert_clkout(true)
        );

        self.regs.reg_65.write(
            regs::Reg65::zeroed()
                .wr_rl_delay(0x2)
                .rd_rl_delay(0x4)
                .dll_lock_diff(0xf)
                .use_wr_level(true)
                .use_rd_dqs_gate_level(true)
                .use_rd_data_eye_level(true)
                .dis_calib_rst(false)
                .ctrl_slave_delay(0x0)
        );
    }

    /// Reset DDR controller
    fn reset_ddrc<F: FnMut(&mut Self)>(&mut self, mut f: F) {
        #[cfg(feature = "target_zc706")]
        let width = regs::DataBusWidth::Width32bit;
        #[cfg(feature = "target_cora_z7_10")]
        let width = regs::DataBusWidth::Width16bit;
        #[cfg(feature = "target_redpitaya")]
        let width = regs::DataBusWidth::Width16bit;
        self.regs.ddrc_ctrl.modify(|_, w| w
            .soft_rstb(false)
            .powerdown_en(false)
            .data_bus_width(width)
        );
        f(self);

        #[cfg(feature = "target_zc706")]
        unsafe {
            // row/column address bits
            self.regs.dram_addr_map_bank.write(0x00000777);
            self.regs.dram_addr_map_col.write(0xFFF00000);
            self.regs.dram_addr_map_row.write(0x0F666666);
        }
        #[cfg(feature = "target_cora_z7_10")]
        unsafe {
            // row/column address bits
            self.regs.dram_addr_map_bank.write(0x00000666);
            self.regs.dram_addr_map_col.write(0xFFFF0000);
            self.regs.dram_addr_map_row.write(0x0F555555);
        }

        self.regs.ddrc_ctrl.modify(|_, w| w
            .soft_rstb(true)
            .powerdown_en(false)
            .data_bus_width(width)
        );

        while self.status() == regs::ControllerStatus::Init {}
    }

    pub fn status(&self) -> regs::ControllerStatus {
        self.regs.mode_sts.read().operating_mode()
    }

    pub fn ptr<T>(&mut self) -> *mut T {
        0x0010_0000 as *mut _
    }

    /// actually there's 1 MB more but starting at 0x0000_0000
    /// overlaps with OCM.
    pub fn size(&self) -> usize {
        #[cfg(feature = "target_zc706")]
        let megabytes = 1023;
        #[cfg(feature = "target_cora_z7_10")]
        let megabytes = 512;
        #[cfg(feature = "target_redpitaya")]
        let megabytes = 511;

        megabytes * 1024 * 1024
    }

    pub fn memtest(&mut self) {
        let slice = unsafe {
            core::slice::from_raw_parts_mut(self.ptr(), self.size())
        };
        let patterns: &'static [u32] = &[0xffff_ffff, 0x5555_5555, 0xaaaa_aaaa, 0];
        let mut expected = None;
        for (i, pattern) in patterns.iter().enumerate() {
            info!("memtest phase {} (status: {:?})", i, self.status());

            for megabyte in 0..slice.len() / (1024 * 1024) {
                let start = megabyte * 1024 * 1024 / 4;
                let end = (megabyte + 1) * 1024 * 1024 / 4;
                for b in slice[start..end].iter_mut() {
                    expected.map(|expected| {
                        let read: u32 = *b;
                        if read != expected {
                            error!("{:08X}: expected {:08X}, read {:08X}", b as *mut _ as usize, expected, read);
                        }
                    });
                    *b = *pattern;
                }

                print!("\r{} MB", megabyte);
            }
            println!(" Ok");

            expected = Some(*pattern);
        }
    }
}
