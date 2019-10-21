use crate::regs::RegisterW;
use super::slcr;
use super::clocks::CpuClocks;

/// Micron MT41J256M8HX-15E: 667 MHz
const DDR_FREQ: u32 = 666_666_666;

pub struct DdrRam {
}

impl DdrRam {
    pub fn new() -> Self {
        Self::clock_setup();
        
        let ram = DdrRam {};
        // TODO: ram.
        ram
    }

    fn clock_setup() {
        let clocks = CpuClocks::get();
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
}
