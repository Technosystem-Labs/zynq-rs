use log::debug;
use libregister::{RegisterR, RegisterW, RegisterRW};
use super::slcr;

#[cfg(feature = "target_zc706")]
pub const PS_CLK: u32 = 33_333_333;
#[cfg(feature = "target_coraz7")]
pub const PS_CLK: u32 = 50_000_000;
#[cfg(feature = "target_ebaz4205")]
pub const PS_CLK: u32 = 33_333_333;
#[cfg(feature = "target_redpitaya")]
pub const PS_CLK: u32 = 33_333_333;
#[cfg(feature = "target_kasli_soc")]
pub const PS_CLK: u32 = 33_333_333;

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

pub trait ClockSource {
    /// picks this ClockSource's registers from the SLCR block
    fn pll_regs(slcr: &mut crate::slcr::RegisterBlock)
                -> (&mut crate::slcr::PllCtrl,
                    &mut crate::slcr::PllCfg,
                    &mut crate::slcr::PllStatus
                   );

    /// query PLL lock status
    fn pll_locked(pll_status: &mut crate::slcr::PllStatus) -> bool;

    /// get configured frequency
    fn freq() -> u32 {
        let mut slcr = slcr::RegisterBlock::slcr();
        let (pll_ctrl, _, _) = Self::pll_regs(&mut slcr);
        u32::from(pll_ctrl.read().pll_fdiv()) * PS_CLK
    }

    fn name() -> &'static str;

    /// Zynq-7000 AP SoC Technical Reference Manual:
    /// 25.10.4 PLLs
    fn setup(target_freq: u32) {
        let fdiv = (target_freq / PS_CLK).min(66) as u16;
        let (pll_cp, pll_res, lock_cnt) = PLL_FDIV_LOCK_PARAM.iter()
            .filter(|(fdiv_max, _)| fdiv <= *fdiv_max)
            .nth(0)
            .expect("PLL_FDIV_LOCK_PARAM")
            .1.clone();

        debug!("Set {} to {} Hz", Self::name(), target_freq);
        slcr::RegisterBlock::unlocked(|slcr| {
            let (pll_ctrl, pll_cfg, pll_status) = Self::pll_regs(slcr);

            // Bypass
            pll_ctrl.modify(|_, w| w
                            .pll_pwrdwn(false)
                            .pll_bypass_force(true)
                            .pll_fdiv(fdiv)
            );
            // Configure
            pll_cfg.write(
                slcr::PllCfg::zeroed()
                    .pll_res(pll_res)
                    .pll_cp(pll_cp)
                    .lock_cnt(lock_cnt)
            );
            // Reset
            pll_ctrl.modify(|_, w| w.pll_reset(true));
            pll_ctrl.modify(|_, w| w.pll_reset(false));
            // Wait for PLL lock
            while ! Self::pll_locked(pll_status) {}
            // Remove bypass
            pll_ctrl.modify(|_, w| w
                            .pll_bypass_force(false)
                            .pll_bypass_qual(false)
            );
        });
    }
}

/// ARM PLL: Recommended clock source for the CPUs and the interconnect
pub struct ArmPll;

impl ClockSource for ArmPll {
    #[inline]
    fn pll_regs(slcr: &mut crate::slcr::RegisterBlock)
                -> (&mut crate::slcr::PllCtrl,
                    &mut crate::slcr::PllCfg,
                    &mut crate::slcr::PllStatus
    ) {
        (&mut slcr.arm_pll_ctrl,
         &mut slcr.arm_pll_cfg,
         &mut slcr.pll_status
        )
    }

    #[inline]
    fn pll_locked(pll_status: &mut crate::slcr::PllStatus) -> bool {
        pll_status.read().arm_pll_lock()
    }

    fn name() -> &'static str {
        &"ARM_PLL"
    }
}

/// DDR PLL: Recommended clock for the DDR DRAM controller and AXI_HP interfaces
pub struct DdrPll;

impl ClockSource for DdrPll {
    #[inline]
    fn pll_regs(slcr: &mut crate::slcr::RegisterBlock)
                -> (&mut crate::slcr::PllCtrl,
                    &mut crate::slcr::PllCfg,
                    &mut crate::slcr::PllStatus
    ) {
        (&mut slcr.ddr_pll_ctrl,
         &mut slcr.ddr_pll_cfg,
         &mut slcr.pll_status
        )
    }

    #[inline]
    fn pll_locked(pll_status: &mut crate::slcr::PllStatus) -> bool {
        pll_status.read().ddr_pll_lock()
    }

    fn name() -> &'static str {
        &"DDR_PLL"
    }
}

/// I/O PLL: Recommended clock for I/O peripherals
pub struct IoPll;


impl ClockSource for IoPll {
    #[inline]
    fn pll_regs(slcr: &mut crate::slcr::RegisterBlock)
                -> (&mut crate::slcr::PllCtrl,
                    &mut crate::slcr::PllCfg,
                    &mut crate::slcr::PllStatus
    ) {
        (&mut slcr.io_pll_ctrl,
         &mut slcr.io_pll_cfg,
         &mut slcr.pll_status
        )
    }

    #[inline]
    fn pll_locked(pll_status: &mut crate::slcr::PllStatus) -> bool {
        pll_status.read().io_pll_lock()
    }

    fn name() -> &'static str {
        &"IO_PLL"
    }
}
