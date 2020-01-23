use libregister::{RegisterR, RegisterW, RegisterRW};
use super::slcr;
pub use slcr::ArmPllSource;

pub mod source;
use source::*;

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
pub struct Clocks {
    /// ARM PLL: Recommended clock source for the CPUs and the interconnect
    pub arm: u32,
    /// DDR PLL: Recommended clock for the DDR DRAM controller and AXI_HP interfaces
    pub ddr: u32,
    /// I/O PLL: Recommended clock for I/O peripherals
    pub io: u32,
}

impl Clocks {
    pub fn get() -> Self {
        Clocks {
            arm: ArmPll::freq(),
            ddr: DdrPll::freq(),
            io: IoPll::freq(),
        }
    }

    pub fn set_cpu_freq(target_freq: u32) {
        let arm_pll = ArmPll::freq();
        // 1 and 3 cannot be used
        let mut div = 2u8;
        while div == 3 || (div < 63 && arm_pll / u32::from(div) > target_freq) {
            div += 1;
        }

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.arm_clk_ctrl.modify(|_, w| w
                .srcsel(ArmPllSource::ArmPll)
                .divisor(div)
            );
        })
    }

    pub fn cpu_6x4x(&self) -> u32 {
        let slcr = slcr::RegisterBlock::new();
        let arm_clk_ctrl = slcr.arm_clk_ctrl.read();
        let pll = match arm_clk_ctrl.srcsel() {
            ArmPllSource::ArmPll => self.arm,
            ArmPllSource::DdrPll => self.ddr,
            ArmPllSource::IoPll => self.io,
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
