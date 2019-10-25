use crate::regs::{RegisterR, RegisterW, RegisterRW};
use super::slcr;

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

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 25.10.4 PLLs
    pub fn enable_ddr(target_clock: u32) {
        let fdiv = (target_clock / PS_CLK).min(66) as u16;
        let regs = slcr::RegisterBlock::new();
        regs.ddr_pll_ctrl.modify(|_, w| w
            .pll_pwrdwn(false)
            .pll_bypass_force(true)
            .pll_fdiv(fdiv)
        );
        let (pll_res, pll_cp, lock_cnt) = PLL_FDIV_LOCK_PARAM.iter()
            .filter(|(fdiv_max, _)| fdiv <= *fdiv_max)
            .last()
            .expect("PLL_FDIV_LOCK_PARAM")
            .1.clone();
        regs.ddr_pll_cfg.write(
            slcr::PllCfg::zeroed()
                .pll_res(pll_res)
                .pll_cp(pll_cp)
                .lock_cnt(lock_cnt)
        );
        regs.ddr_pll_ctrl.modify(|_, w| w
            .pll_reset(true)
        );
        regs.ddr_pll_ctrl.modify(|_, w| w
            .pll_reset(false)
        );
        while ! regs.pll_status.read().ddr_pll_lock() {}
        regs.ddr_pll_ctrl.modify(|_, w| w
            .pll_bypass_force(false)
            .pll_bypass_qual(false)
        );
    }
}

/// (pll_fdiv_max, (pll_cp, pll_res, lock_cnt))
const PLL_FDIV_LOCK_PARAM: &[(u16, (u8, u8, u16))] = &[
    (13, (2, 6, 750)),
    (14, (2, 6, 700)),
    (15, (2, 6, 650)),
    (16, (2, 10, 625)),
    (17, (2, 10, 575)),
    (18, (2, 10, 550)),
    (19, (2, 10, 525)),
    (20, (2, 12, 500)),
    (21, (2, 12, 475)),
    (22, (2, 12, 450)),
    (23, (2, 12, 425)),
    (25, (2, 12, 400)),
    (26, (2, 12, 375)),
    (28, (2, 12, 350)),
    (30, (2, 12, 325)),
    (33, (2, 2, 300)),
    (36, (2, 2, 275)),
    (40, (2, 2, 250)),
    (47, (3, 12, 250)),
    (66, (2, 4, 250)),
];
