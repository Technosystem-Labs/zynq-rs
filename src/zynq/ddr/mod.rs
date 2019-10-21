use crate::regs::{RegisterR, RegisterW, RegisterRW};
use super::slcr;
use super::clocks::CpuClocks;

/// Micron MT41J256M8HX-15E: 667 MHz
const DDR_FREQ: u32 = 666_666_666;
const DCI_FREQ: u32 = 10_000_000;

pub struct DdrRam {
}

impl DdrRam {
    pub fn new() -> Self {
        let clocks = CpuClocks::get();
        Self::clock_setup(&clocks);
        Self::calibrate_iob_impedance(&clocks);

        let ram = DdrRam {};
        ram
    }

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
}
