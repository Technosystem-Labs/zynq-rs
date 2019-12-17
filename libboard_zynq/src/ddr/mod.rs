use libregister::{RegisterR, RegisterW, RegisterRW};
use crate::{print, println};
use super::slcr;
use super::clocks::CpuClocks;

mod regs;

#[cfg(feature = "target_zc706")]
/// Micron MT41J256M8HX-15E: 667 MHz DDR3
const DDR_FREQ: u32 = 666_666_666;

#[cfg(feature = "target_cora_z7_10")]
/// Micron MT41K256M16HA-125: 800 MHz DDR3L, max supported 533 MHz
const DDR_FREQ: u32 = 533_333_333;

/// MT41K256M16HA-125
const DCI_FREQ: u32 = 10_000_000;

pub struct DdrRam {
    regs: &'static mut regs::RegisterBlock,
}

impl DdrRam {
    pub fn new() -> Self {
        let clocks = Self::clock_setup();
        Self::calibrate_iob_impedance(&clocks);
        Self::configure_iob();

        let regs = unsafe { regs::RegisterBlock::new() };
        let mut ddr = DdrRam { regs };
        ddr.reset_ddrc();
        ddr
    }

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 10.6.1 DDR Clock Initialization
    fn clock_setup() -> CpuClocks {
        let clocks = CpuClocks::get();
        if clocks.ddr == 0 {
            CpuClocks::enable_ddr(clocks.arm);
        }
        let clocks = CpuClocks::get();
        println!("Clocks: {:?}", clocks);

        let ddr3x_clk_divisor = ((DDR_FREQ - 1 + clocks.ddr) / DDR_FREQ).min(255) as u8;
        let ddr2x_clk_divisor = 3 * ddr3x_clk_divisor / 2;
        println!("DDR 3x/2x clocks: {}/{}", clocks.ddr / u32::from(ddr3x_clk_divisor), clocks.ddr / u32::from(ddr2x_clk_divisor));

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

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 10.6.2 DDR IOB Impedance Calibration
    fn calibrate_iob_impedance(clocks: &CpuClocks) {
        let divisor0 = ((DCI_FREQ - 1 + clocks.ddr) / DCI_FREQ)
            .max(1).min(63) as u8;
        let divisor1 = (clocks.ddr / DCI_FREQ / u32::from(divisor0))
            .max(1).min(63) as u8;
        println!("DDR DCI clock: {} Hz", clocks.ddr / u32::from(divisor0) / u32::from(divisor1));

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

            let data_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::VrefDifferential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            slcr.ddriob_data0.write(data_config.clone());
            slcr.ddriob_data1.write(data_config);

            let diff_config = slcr::DdriobConfig::zeroed()
                .inp_type(slcr::DdriobInputType::Differential)
                .term_en(true)
                .dci_type(slcr::DdriobDciType::Termination)
                .output_en(slcr::DdriobOutputEn::Obuf);
            slcr.ddriob_diff0.write(diff_config.clone());
            slcr.ddriob_diff1.write(diff_config);

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

            #[cfg(feature = "target_zc706")]
            let vref_sel = slcr::DdriobVrefSel::Vref0_75V;
            #[cfg(feature = "target_cora_z7_10")]
            let vref_sel = slcr::DdriobVrefSel::Vref0_675V;

            // // Enable internal V[REF]
            // slcr.ddriob_ddr_ctrl.modify(|_, w| w
            //         .vref_ext_en_lower(false)
            //         .vref_ext_en_upper(false)
            //         .vref_sel(vref_sel)
            //         .vref_int_en(true)
            // );
            // Enable external V[REF]
            slcr.ddriob_ddr_ctrl.modify(|_, w| w
                    .vref_ext_en_lower(true)
                    .vref_ext_en_upper(true)
                    .vref_sel(vref_sel)
                    .vref_int_en(false)
            );
        });
    }

    /// Reset DDR controller
    fn reset_ddrc(&mut self) {
        self.regs.ddrc_ctrl.modify(|_, w| w
            .soft_rstb(false)
        );
        #[cfg(feature = "target_zc706")]
        let width = regs::DataBusWidth::Width32bit;
        #[cfg(feature = "target_cora_z7_10")]
        let width = regs::DataBusWidth::Width16bit;
        self.regs.ddrc_ctrl.modify(|_, w| w
            .soft_rstb(true)
            .powerdown_en(false)
            .data_bus_width(width)
        );

        while self.status() == regs::ControllerStatus::Init {}
    }

    pub fn status(&self) -> regs::ControllerStatus {
        self.regs.mode_sts_reg.read().operating_mode()
    }

    pub fn ptr<T>(&mut self) -> *mut T {
        0x0010_0000 as *mut _
    }

    pub fn size(&self) -> usize {
        #[cfg(feature = "target_zc706")]
        let megabytes = 511;
        #[cfg(feature = "target_cora_z7_10")]
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
            println!("memtest phase {} (status: {:?})", i, self.status());

            for megabyte in 0..=(slice.len() / (1024 * 1024)) {
                let start = megabyte * 1024 * 1024 / 4;
                let end = ((megabyte + 1) * 1024 * 1024 / 4).min(slice.len());
                for b in slice[start..end].iter_mut() {
                    expected.map(|expected| {
                        let read: u32 = *b;
                        if read != expected {
                            println!("{:08X}: expected {:08X}, read {:08X}", b as *mut _ as usize, expected, read);
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
