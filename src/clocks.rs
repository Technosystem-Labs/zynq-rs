use crate::slcr;
use crate::regs::RegisterR;

#[cfg(feature = "target_zc706")]
const PS_CLK: u32 = 33_333_333;
#[cfg(feature = "target_cora_z7_10")]
const PS_CLK: u32 = 50_000_000;

enum CpuClockMode {
    /// Clocks run in 4:2:2:1 mode
    C421,
    /// Clocks run in 6:3:2:1 mode
    C621,
}

impl CpuClockMode {
    pub fn get() -> Self {
        let regs = slcr::RegisterBlock::new();
        if regs.clk_621_true.read().clk_621_true() {
            CpuClockMode::C621
        } else {
            CpuClockMode::C421
        }
    }    
}

#[derive(Debug, Clone)]
pub struct CpuClocks {
    /// ARM PLL: Recommended clock source for the CPUs and the interconnect
    pub arm: u32,
    /// DDR PLL: Recommended clock for the DDR DRAM controller and AXI_HP interfaces
    pub ddr: u32,
    /// I/O PLL: Recommended clock for I/O peripherals
    pub io: u32,
}

impl CpuClocks {
    pub fn get() -> Self {
        let regs = slcr::RegisterBlock::new();
        let arm = u32::from(regs.arm_pll_ctrl.read().pll_fdiv()) * PS_CLK;
        let ddr = u32::from(regs.ddr_pll_ctrl.read().pll_fdiv()) * PS_CLK;
        let io = u32::from(regs.io_pll_ctrl.read().pll_fdiv()) * PS_CLK;
        CpuClocks { arm, ddr, io }
    }

    pub fn cpu_6x4x(&self) -> u32 {
        let regs = slcr::RegisterBlock::new();
        let arm_clk_ctrl = regs.arm_clk_ctrl.read();
        let pll = match arm_clk_ctrl.srcsel() {
            slcr::ArmPllSource::ArmPll => self.arm,
            slcr::ArmPllSource::DdrPll => self.ddr,
            slcr::ArmPllSource::IoPll => self.io,
        };
        pll / u32::from(arm_clk_ctrl.divisor())
    }

    pub fn cpu_3x2x(&self) -> u32 {
        self.cpu_6x4x() / 2
    }

    pub fn cpu_2x(&self) -> u32 {
        match CpuClockMode::get() {
            CpuClockMode::C421 =>
                self.cpu_6x4x() / 2,
            CpuClockMode::C621 =>
                self.cpu_6x4x() / 3,
        }
    }

    pub fn cpu_1x(&self) -> u32 {
        match CpuClockMode::get() {
            CpuClockMode::C421 =>
                self.cpu_6x4x() / 4,
            CpuClockMode::C621 =>
                self.cpu_6x4x() / 6,
        }
    }

    pub fn uart_ref_clk(&self) -> u32 {
        let regs = slcr::RegisterBlock::new();
        let uart_clk_ctrl = regs.uart_clk_ctrl.read();
        let pll = match uart_clk_ctrl.srcsel() {
            slcr::PllSource::ArmPll =>
                self.arm,
            slcr::PllSource::DdrPll =>
                self.ddr,
            slcr::PllSource::IoPll =>
                self.io,
        };
        pll / u32::from(uart_clk_ctrl.divisor())
    }
}
