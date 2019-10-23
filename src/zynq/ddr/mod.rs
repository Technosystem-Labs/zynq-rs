use crate::regs::{RegisterR, RegisterW, RegisterRW};
use super::slcr;
use super::clocks::CpuClocks;

/// Micron MT41J256M8HX-15E: 667 MHz DDR3
const DDR_FREQ: u32 = 666_666_666;
const DCI_FREQ: u32 = 10_000_000;

pub struct DdrRam {
}

impl DdrRam {
    pub fn new() -> Self {
        let clocks = CpuClocks::get();
        Self::clock_setup(&clocks);
        Self::calibrate_iob_impedance(&clocks);
        Self::configure_iob();

        let ram = DdrRam {};
        ram
    }

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 10.6.1 DDR Clock Initialization
    fn clock_setup(clocks: &CpuClocks) {
        let ddr3x_clk_divisor = ((clocks.ddr - 1) / DDR_FREQ + 1).min(255) as u8;
        let ddr2x_clk_divisor = 3 * ddr3x_clk_divisor / 2;

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.ddr_pll_ctrl.write(
                slcr::PllCtrl::zeroed()
            );

            slcr.ddr_clk_ctrl.write(
                slcr::DdrClkCtrl::zeroed()
                    .ddr_2xclkact(true)
                    .ddr_3xclkact(true)
                    .ddr_2xclk_divisor(ddr2x_clk_divisor)
                    .ddr_3xclk_divisor(ddr3x_clk_divisor)
            );
        });
    }

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 10.6.2 DDR IOB Impedance Calibration
    fn calibrate_iob_impedance(clocks: &CpuClocks) {
        let divisor0 = (clocks.ddr / DCI_FREQ)
            .max(1).min(63) as u8;
        let divisor1 = (clocks.ddr / DCI_FREQ / u32::from(divisor0))
            .max(1).min(63) as u8;

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
            // Step 3.b. for DDR3
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

            // Enable internal V[REF]
            slcr.ddriob_ddr_ctrl.modify(|_, w| w
                    .vref_ext_en_lower(false)
                    .vref_ext_en_upper(false)
                    .vref_sel(slcr::DdriobVrefSel::Vref0_75V)
                    .vref_int_en(true)
            );
        });
    }
}
