///! Register definitions for System Level Control

use volatile_register::{RO, RW};
use libregister::{
    register, register_at,
    register_bit, register_bits, register_bits_typed,
    RegisterW, RegisterRW,
};

#[repr(u8)]
pub enum PllSource {
    IoPll  = 0b00,
    ArmPll = 0b10,
    DdrPll = 0b11,
}

#[repr(u8)]
pub enum ArmPllSource {
    ArmPll = 0b00,
    DdrPll = 0b10,
    IoPll  = 0b11,
}

#[repr(u8)]
pub enum DdriobInputType {
    Off = 0b00,
    /// For SSTL, HSTL
    VrefDifferential = 0b01,
    Differential = 0b10,
    Lvcmos = 0b11,
}

#[repr(u8)]
pub enum DdriobDciType {
    /// DDR2/3L Addr and Clock
    Disabled = 0b00,
    /// LPDDR2
    Drive = 0b01,
    /// DDR2/3/3L Data and Diff
    Termination = 0b11,
}

#[repr(u8)]
pub enum DdriobOutputEn {
    Ibuf = 0b00,
    Obuf = 0b11,
}


#[repr(u8)]
pub enum DdriobVrefSel {
    /// For LPDDR2 with 1.2V IO
    Vref0_6V = 0b0001,
    /// For DDR3L with 1.35V IO
    Vref0_675V = 0b0010,
    /// For DDR3 with 1.5V IO
    Vref0_75V = 0b0100,
    /// For DDR2 with 1.8V IO
    Vref0_9V = 0b1000,
}

#[repr(u8)]
pub enum LevelShifterEnable {
    DisableAll = 0x0,
    EnablePsToPl = 0xA,
    EnableAll = 0xF,
}


#[repr(C)]
pub struct RegisterBlock {
    pub scl: RW<u32>,
    pub slcr_lock: SlcrLock,
    pub slcr_unlock: SlcrUnlock,
    pub slcr_locksta: RO<u32>,
    reserved0: [u32; 60],
    pub arm_pll_ctrl: PllCtrl,
    pub ddr_pll_ctrl: PllCtrl,
    pub io_pll_ctrl: PllCtrl,
    pub pll_status: PllStatus,
    pub arm_pll_cfg: PllCfg,
    pub ddr_pll_cfg: PllCfg,
    pub io_pll_cfg: PllCfg,
    reserved1: [u32; 1],
    pub arm_clk_ctrl: ArmClkCtrl,
    pub ddr_clk_ctrl: DdrClkCtrl,
    pub dci_clk_ctrl: DciClkCtrl,
    pub aper_clk_ctrl: AperClkCtrl,
    pub usb0_clk_ctrl: RW<u32>,
    pub usb1_clk_ctrl: RW<u32>,
    pub gem0_rclk_ctrl: RclkCtrl,
    pub gem1_rclk_ctrl: RclkCtrl,
    pub gem0_clk_ctrl: GemClkCtrl,
    pub gem1_clk_ctrl: GemClkCtrl,
    pub smc_clk_ctrl: RW<u32>,
    pub lqspi_clk_ctrl: LqspiClkCtrl,
    pub sdio_clk_ctrl: SdioClkCtrl,
    pub uart_clk_ctrl: UartClkCtrl,
    pub spi_clk_ctrl: RW<u32>,
    pub can_clk_ctrl: RW<u32>,
    pub can_mioclk_ctrl: RW<u32>,
    pub dbg_clk_ctrl: RW<u32>,
    pub pcap_clk_ctrl: RW<u32>,
    pub topsw_clk_ctrl: RW<u32>,
    pub fpga0_clk_ctrl: Fpga0ClkCtrl,
    pub fpga0_thr_ctrl: RW<u32>,
    pub fpga0_thr_cnt: RW<u32>,
    pub fpga0_thr_sta: RO<u32>,
    pub fpga1_clk_ctrl: Fpga1ClkCtrl,
    pub fpga1_thr_ctrl: RW<u32>,
    pub fpga1_thr_cnt: RW<u32>,
    pub fpga1_thr_sta: RO<u32>,
    pub fpga2_clk_ctrl: Fpga2ClkCtrl,
    pub fpga2_thr_ctrl: RW<u32>,
    pub fpga2_thr_cnt: RW<u32>,
    pub fpga2_thr_sta: RO<u32>,
    pub fpga3_clk_ctrl: Fpga3ClkCtrl,
    pub fpga3_thr_ctrl: RW<u32>,
    pub fpga3_thr_cnt: RW<u32>,
    pub fpga3_thr_sta: RO<u32>,
    reserved2: [u32; 5],
    pub clk_621_true: Clk621True,
    reserved3: [u32; 14],
    pub pss_rst_ctrl: PssRstCtrl,
    pub ddr_rst_ctrl: RW<u32>,
    pub topsw_rst_ctrl: RW<u32>,
    pub dmac_rst_ctrl: RW<u32>,
    pub usb_rst_ctrl: RW<u32>,
    pub gem_rst_ctrl: RW<u32>,
    pub sdio_rst_ctrl: SdioRstCtrl,
    pub spi_rst_ctrl: RW<u32>,
    pub can_rst_ctrl: RW<u32>,
    pub i2c_rst_ctrl: RW<u32>,
    pub uart_rst_ctrl: UartRstCtrl,
    pub gpio_rst_ctrl: GpioRstCtrl,
    pub lqspi_rst_ctrl: LqspiRstCtrl,
    pub smc_rst_ctrl: RW<u32>,
    pub ocm_rst_ctrl: RW<u32>,
    reserved4: [u32; 1],
    pub fpga_rst_ctrl: FpgaRstCtrl,
    pub a9_cpu_rst_ctrl: A9CpuRstCtrl,
    reserved5: [u32; 1],
    pub rs_awdt_ctrl: RW<u32>,
    reserved6: [u32; 2],
    pub reboot_status: RW<u32>,
    pub boot_mode: BootMode,
    reserved7: [u32; 40],
    pub apu_ctrl: RW<u32>,
    pub wdt_clk_sel: RW<u32>,
    reserved8: [u32; 78],
    pub tz_dma_ns: RW<u32>,
    pub tz_dma_irq_ns: RW<u32>,
    pub tz_dma_periph_ns: RW<u32>,
    reserved9: [u32; 57],
    pub pss_idcode: RW<u32>,
    reserved10: [u32; 51],
    pub ddr_urgent: RW<u32>,
    reserved11: [u32; 2],
    pub ddr_cal_start: RW<u32>,
    reserved12: [u32; 1],
    pub ddr_ref_start: RW<u32>,
    pub ddr_cmd_sta: RW<u32>,
    pub ddr_urgent_sel: RW<u32>,
    pub ddr_dfi_status: RW<u32>,
    reserved13: [u32; 55],
    pub mio_pin_00: MioPin00,
    pub mio_pin_01: MioPin01,
    pub mio_pin_02: MioPin02,
    pub mio_pin_03: MioPin03,
    pub mio_pin_04: MioPin04,
    pub mio_pin_05: MioPin05,
    pub mio_pin_06: MioPin06,
    pub mio_pin_07: MioPin07,
    pub mio_pin_08: MioPin08,
    pub mio_pin_09: MioPin09,
    pub mio_pin_10: MioPin10,
    pub mio_pin_11: MioPin11,
    pub mio_pin_12: MioPin12,
    pub mio_pin_13: MioPin13,
    pub mio_pin_14: MioPin14,
    pub mio_pin_15: MioPin15,
    pub mio_pin_16: MioPin16,
    pub mio_pin_17: MioPin17,
    pub mio_pin_18: MioPin18,
    pub mio_pin_19: MioPin19,
    pub mio_pin_20: MioPin20,
    pub mio_pin_21: MioPin21,
    pub mio_pin_22: MioPin22,
    pub mio_pin_23: MioPin23,
    pub mio_pin_24: MioPin24,
    pub mio_pin_25: MioPin25,
    pub mio_pin_26: MioPin26,
    pub mio_pin_27: MioPin27,
    pub mio_pin_28: MioPin28,
    pub mio_pin_29: MioPin29,
    pub mio_pin_30: MioPin30,
    pub mio_pin_31: MioPin31,
    pub mio_pin_32: MioPin32,
    pub mio_pin_33: MioPin33,
    pub mio_pin_34: MioPin34,
    pub mio_pin_35: MioPin35,
    pub mio_pin_36: MioPin36,
    pub mio_pin_37: MioPin37,
    pub mio_pin_38: MioPin38,
    pub mio_pin_39: MioPin39,
    pub mio_pin_40: MioPin40,
    pub mio_pin_41: MioPin41,
    pub mio_pin_42: MioPin42,
    pub mio_pin_43: MioPin43,
    pub mio_pin_44: MioPin44,
    pub mio_pin_45: MioPin45,
    pub mio_pin_46: MioPin46,
    pub mio_pin_47: MioPin47,
    pub mio_pin_48: MioPin48,
    pub mio_pin_49: MioPin49,
    pub mio_pin_50: MioPin50,
    pub mio_pin_51: MioPin51,
    pub mio_pin_52: MioPin52,
    pub mio_pin_53: MioPin53,
    reserved14: [u32; 11],
    pub mio_loopback: RW<u32>,
    reserved15: [u32; 1],
    pub mio_mst_tri0: RW<u32>,
    pub mio_mst_tri1: RW<u32>,
    reserved16: [u32; 7],
    pub sd0_wp_cd_sel: RW<u32>,
    pub sd1_wp_cd_sel: RW<u32>,
    reserved17: [u32; 50],
    pub lvl_shftr_en: LvlShftr,
    reserved18: [u32; 3],
    pub ocm_cfg: RW<u32>,
    reserved19: [u32; 123],
    pub gpiob_ctrl: GpiobCtrl,
    pub gpiob_cfg_cmos18: RW<u32>,
    pub gpiob_cfg_cmos25: RW<u32>,
    pub gpiob_cfg_cmos33: RW<u32>,
    reserved20: [u32; 1],
    pub gpiob_cfg_hstl: RW<u32>,
    pub gpiob_drvr_bias_ctrl: RW<u32>,
    reserved21: [u32; 9],
    pub ddriob_addr0: DdriobConfig,
    pub ddriob_addr1: DdriobConfig,
    pub ddriob_data0: DdriobConfig,
    pub ddriob_data1: DdriobConfig,
    pub ddriob_diff0: DdriobConfig,
    pub ddriob_diff1: DdriobConfig,
    pub ddriob_clock: DdriobConfig,
    pub ddriob_drive_slew_addr: RW<u32>,
    pub ddriob_drive_slew_data: RW<u32>,
    pub ddriob_drive_slew_diff: RW<u32>,
    pub ddriob_drive_slew_clock: RW<u32>,
    pub ddriob_ddr_ctrl: DdriobDdrCtrl,
    pub ddriob_dci_ctrl: DdriobDciCtrl,
    pub ddriob_dci_status: DdriobDciStatus,
}
register_at!(RegisterBlock, 0xF8000000, slcr);

impl RegisterBlock {
    /// Required to modify any sclr register
    pub fn unlocked<F: FnMut(&mut Self) -> R, R>(mut f: F) -> R {
        let mut self_ = Self::slcr();
        self_.slcr_unlock.unlock();
        let r = f(&mut self_);
        self_.slcr_lock.lock();
        r
    }

    pub fn init_preload_fpga(&mut self) {
        // Assert FPGA top level output resets
        self.fpga_rst_ctrl.write(
            FpgaRstCtrl::zeroed()
                .fpga0_out_rst(true)
                .fpga1_out_rst(true)
                .fpga2_out_rst(true)
                .fpga3_out_rst(true)
        );
        // Disable level shifters
        self.lvl_shftr_en.write(
            LvlShftr::zeroed()
        );
        // Enable output level shifters
        self.lvl_shftr_en.write(
            LvlShftr::zeroed()
                .user_lvl_shftr_en(LevelShifterEnable::EnablePsToPl)
        );
    }

    pub fn init_postload_fpga(&mut self) {
        // Enable level shifters
        self.lvl_shftr_en.write(
            LvlShftr::zeroed()
                .user_lvl_shftr_en(LevelShifterEnable::EnableAll)
        );
        // Deassert AXI interface resets
        self.fpga_rst_ctrl.write(
            FpgaRstCtrl::zeroed()
        );
    }
}

register!(slcr_lock, SlcrLock, WO, u32);
register_bits!(slcr_lock, lock_key, u16, 0, 15);
impl SlcrLock {
    pub fn lock(&mut self) {
        self.write(
            Self::zeroed()
                .lock_key(0x767B)
        );
    }
}

register!(slcr_unlock, SlcrUnlock, WO, u32);
register_bits!(slcr_unlock, unlock_key, u16, 0, 15);
impl SlcrUnlock {
    pub fn unlock(&mut self) {
        self.write(
            Self::zeroed()
                .unlock_key(0xDF0D)
        );
    }
}

register!(pll_ctrl, PllCtrl, RW, u32);
register_bits!(pll_ctrl, pll_fdiv, u16, 12, 18);
register_bit!(pll_ctrl, pll_bypass_force, 4);
register_bit!(pll_ctrl, pll_bypass_qual, 3);
register_bit!(pll_ctrl, pll_pwrdwn, 1);
register_bit!(pll_ctrl, pll_reset, 0);

register!(pll_status, PllStatus, RO, u32);
register_bit!(pll_status, arm_pll_lock, 0);
register_bit!(pll_status, ddr_pll_lock, 1);
register_bit!(pll_status, io_pll_lock, 2);
register_bit!(pll_status, arm_pll_stable, 3);
register_bit!(pll_status, ddr_pll_stable, 4);
register_bit!(pll_status, io_pll_stable, 5);

impl core::fmt::Display for pll_status::Read {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        write!(fmt, "ARM: {}/{} DDR: {}/{} IO: {}/{}",
               if self.arm_pll_lock() { "locked" } else { "NOT locked" },
               if self.arm_pll_stable() { "stable" } else { "UNSTABLE" },
               if self.ddr_pll_lock() { "locked" } else { "NOT locked" },
               if self.ddr_pll_stable() { "stable" } else { "UNSTABLE" },
               if self.io_pll_lock() { "locked" } else { "NOT locked" },
               if self.io_pll_stable() { "stable" } else { "UNSTABLE" },
        )
    }
}

register!(pll_cfg, PllCfg, RW, u32);
register_bits!(pll_cfg, pll_res, u8, 4, 7);
register_bits!(pll_cfg, pll_cp, u8, 8, 11);
register_bits!(pll_cfg, lock_cnt, u16, 12, 21);

register!(arm_clk_ctrl, ArmClkCtrl, RW, u32);
register_bit!(arm_clk_ctrl,
              /// Clock active
              cpu_peri_clkact, 28);
register_bit!(arm_clk_ctrl, cpu_1xclkact, 27);
register_bit!(arm_clk_ctrl, cpu_2xclkact, 26);
register_bit!(arm_clk_ctrl, cpu_3or2xclkact, 25);
register_bit!(arm_clk_ctrl, cpu_6or4xclkact, 24);
register_bits!(arm_clk_ctrl,
               /// should be divisible by 2 (see TRM: 25.2 CPU Clock)
               divisor, u8, 8, 13);
register_bits_typed!(arm_clk_ctrl, srcsel, u8, ArmPllSource, 4, 5);

register!(ddr_clk_ctrl, DdrClkCtrl, RW, u32);
register_bit!(ddr_clk_ctrl, ddr_3xclkact, 0);
register_bit!(ddr_clk_ctrl, ddr_2xclkact, 1);
register_bits!(ddr_clk_ctrl, ddr_3xclk_divisor, u8, 20, 25);
register_bits!(ddr_clk_ctrl, ddr_2xclk_divisor, u8, 26, 31);

register!(dci_clk_ctrl, DciClkCtrl, RW, u32);
register_bit!(dci_clk_ctrl, clkact, 0);
register_bits!(dci_clk_ctrl, divisor0, u8, 8, 13);
register_bits!(dci_clk_ctrl, divisor1, u8, 20, 25);

register!(clk_621_true, Clk621True, RW, u32);
register_bit!(clk_621_true, clk_621_true, 0);

register!(aper_clk_ctrl, AperClkCtrl, RW, u32);
register_bit!(aper_clk_ctrl, uart1_cpu_1xclkact, 21);
register_bit!(aper_clk_ctrl, uart0_cpu_1xclkact, 20);
register_bit!(aper_clk_ctrl, sdio1_cpu_1xclkact, 11);
register_bit!(aper_clk_ctrl, sdio0_cpu_1xclkact, 10);
impl AperClkCtrl {
    pub fn enable_uart0(&mut self) {
        self.modify(|_, w| w.uart0_cpu_1xclkact(true));
    }

    pub fn enable_uart1(&mut self) {
        self.modify(|_, w| w.uart1_cpu_1xclkact(true));
    }

    pub fn enable_sdio0(&mut self) {
        self.modify(|_, w| w.sdio0_cpu_1xclkact(true));
    }

    pub fn enable_sdio1(&mut self) {
        self.modify(|_, w| w.sdio1_cpu_1xclkact(true));
    }
}

register!(rclk_ctrl, RclkCtrl, RW, u32);
register_bit!(rclk_ctrl,
              /// Ethernet controller Rx clock control
              clkact, 0);
register_bit!(rclk_ctrl,
              /// false: MIO, true: EMIO
              srcsel, 4);

register!(gem_clk_ctrl, GemClkCtrl, RW, u32);
register_bits!(gem_clk_ctrl,
               /// 2nd divisor for source clock
               divisor1, u8, 20, 25);
register_bits!(gem_clk_ctrl,
               /// 1st divisor for source clock
               divisor, u8, 8, 13);
register_bits_typed!(gem_clk_ctrl,
                     /// Source to generate the ref clock
                     srcsel, u8, PllSource, 4, 6);
register_bit!(gem_clk_ctrl,
              /// SMC reference clock control
              clkact, 0);

register!(sdio_clk_ctrl, SdioClkCtrl, RW, u32);
register_bit!(sdio_clk_ctrl, clkact0, 0);
register_bit!(sdio_clk_ctrl, clkact1, 1);
register_bits!(sdio_clk_ctrl, divisor, u8, 8, 13);
register_bits_typed!(sdio_clk_ctrl, srcsel, u8, PllSource, 4, 5);
impl SdioClkCtrl {
    pub fn enable_sdio0(&mut self) {
        self.modify(|_, w| {
            w.divisor(0x14).srcsel(PllSource::IoPll).clkact0(true)
        })
    }
    pub fn enable_sdio1(&mut self) {
        self.modify(|_, w| {
            w.divisor(0x14).srcsel(PllSource::IoPll).clkact1(true)
        })
    }
}

register!(uart_clk_ctrl, UartClkCtrl, RW, u32);
register_bit!(uart_clk_ctrl, clkact0, 0);
register_bit!(uart_clk_ctrl, clkact1, 1);
register_bits!(uart_clk_ctrl, divisor, u8, 8, 13);
register_bits_typed!(uart_clk_ctrl, srcsel, u8, PllSource, 4, 5);
register_at!(UartClkCtrl, 0xF8000154, new);
impl UartClkCtrl {
    pub fn enable_uart0(&mut self) {
        self.modify(|_, w| {
            // a. Clock divisor, slcr.UART_CLK_CTRL[DIVISOR] = 0x14.
            // b. Select the IO PLL, slcr.UART_CLK_CTRL[SRCSEL] = 0.
            // c. Enable the UART 0 Reference clock, slcr.UART_CLK_CTRL [CLKACT0] = 1.
            w.divisor(0x14)
             .srcsel(PllSource::IoPll)
             .clkact0(true)
        })
    }

    pub fn enable_uart1(&mut self) {
        self.modify(|_, w| {
            // a. Clock divisor, slcr.UART_CLK_CTRL[DIVISOR] = 0x14.
            // b. Select the IO PLL, slcr.UART_CLK_CTRL[SRCSEL] = 0.
            // c. Enable the UART 1 Reference clock, slcr.UART_CLK_CTRL [CLKACT1] = 1.
            w.divisor(0x14)
             .srcsel(PllSource::IoPll)
             .clkact1(true)
        })
    }
}

register!(sdio_rst_ctrl, SdioRstCtrl, RW, u32);
register_bit!(sdio_rst_ctrl, sdio1_ref_rst, 5);
register_bit!(sdio_rst_ctrl, sdio0_ref_rst, 4);
register_bit!(sdio_rst_ctrl, sdio1_cpu1x_rst, 1);
register_bit!(sdio_rst_ctrl, sdio0_cpu1x_rst, 0);
impl SdioRstCtrl {
    pub fn reset_sdio0(&mut self) {
        self.modify(|_, w|
            w.sdio0_ref_rst(true)
             .sdio0_cpu1x_rst(true)
        );
        self.modify(|_, w|
            w.sdio0_ref_rst(false)
             .sdio0_cpu1x_rst(false)
        );
    }
    pub fn reset_sdio1(&mut self) {
        self.modify(|_, w|
            w.sdio1_ref_rst(true)
             .sdio1_cpu1x_rst(true)
        );
        self.modify(|_, w|
            w.sdio1_ref_rst(false)
             .sdio1_cpu1x_rst(false)
        );
    }
}

register!(uart_rst_ctrl, UartRstCtrl, RW, u32);
register_bit!(uart_rst_ctrl, uart0_ref_rst, 3);
register_bit!(uart_rst_ctrl, uart1_ref_rst, 2);
register_bit!(uart_rst_ctrl, uart0_cpu1x_rst, 1);
register_bit!(uart_rst_ctrl, uart1_cpu1x_rst, 0);
register_at!(UartRstCtrl, 0xF8000228, new);
impl UartRstCtrl {
    pub fn reset_uart0(&mut self) {
        self.modify(|_, w|
            w.uart0_ref_rst(true)
             .uart0_cpu1x_rst(true)
        );
        self.modify(|_, w|
            w.uart0_ref_rst(false)
             .uart0_cpu1x_rst(false)
        );
    }

    pub fn reset_uart1(&mut self) {
        self.modify(|_, w|
            w.uart1_ref_rst(true)
             .uart1_cpu1x_rst(true)
        );
        self.modify(|_, w|
            w.uart1_ref_rst(false)
             .uart1_cpu1x_rst(false)
        );
    }
}

register!(gpio_rst_ctrl, GpioRstCtrl, RW, u32);
register_bit!(gpio_rst_ctrl, gpio_cpu1x_rst, 0);
register_at!(GpioRstCtrl, 0xF800022C, new);
impl GpioRstCtrl {
    pub fn reset_gpio(&mut self) {
        self.modify(|_, w|
            w.gpio_cpu1x_rst(true)
        );
        self.modify(|_, w|
            w.gpio_cpu1x_rst(false)
        );
    }
}

register!(lqspi_clk_ctrl, LqspiClkCtrl, RW, u32);
register_bit!(lqspi_clk_ctrl, clkact, 0);
register_bits_typed!(lqspi_clk_ctrl, src_sel, u8, PllSource, 4, 5);
register_bits!(lqspi_clk_ctrl, divisor, u8, 8, 13);

register!(lqspi_rst_ctrl, LqspiRstCtrl, RW, u32);
register_bit!(lqspi_rst_ctrl, ref_rst, 1);
register_bit!(lqspi_rst_ctrl, cpu1x_rst, 0);

register!(fpga0_clk_ctrl, Fpga0ClkCtrl, RW, u32);
register_bits!(fpga0_clk_ctrl, divisor1, u8, 20, 25);
register_bits!(fpga0_clk_ctrl, divisor0, u8, 8, 13);
register_bits_typed!(fpga0_clk_ctrl, src_sel, u8, PllSource, 4, 5);

register!(fpga1_clk_ctrl, Fpga1ClkCtrl, RW, u32);
register_bits!(fpga1_clk_ctrl, divisor1, u8, 20, 25);
register_bits!(fpga1_clk_ctrl, divisor0, u8, 8, 13);
register_bits_typed!(fpga1_clk_ctrl, src_sel, u8, PllSource, 4, 5);

register!(fpga2_clk_ctrl, Fpga2ClkCtrl, RW, u32);
register_bits!(fpga2_clk_ctrl, divisor1, u8, 20, 25);
register_bits!(fpga2_clk_ctrl, divisor0, u8, 8, 13);
register_bits_typed!(fpga2_clk_ctrl, src_sel, u8, PllSource, 4, 5);

register!(fpga3_clk_ctrl, Fpga3ClkCtrl, RW, u32);
register_bits!(fpga3_clk_ctrl, divisor1, u8, 20, 25);
register_bits!(fpga3_clk_ctrl, divisor0, u8, 8, 13);
register_bits_typed!(fpga3_clk_ctrl, src_sel, u8, PllSource, 4, 5);

register!(fpga_rst_ctrl, FpgaRstCtrl, RW, u32);
register_bit!(fpga_rst_ctrl, fpga0_out_rst, 0);
register_bit!(fpga_rst_ctrl, fpga1_out_rst, 1);
register_bit!(fpga_rst_ctrl, fpga2_out_rst, 2);
register_bit!(fpga_rst_ctrl, fpga3_out_rst, 3);

register!(a9_cpu_rst_ctrl, A9CpuRstCtrl, RW, u32);
register_bit!(a9_cpu_rst_ctrl, peri_rst, 8);
register_bit!(a9_cpu_rst_ctrl, a9_clkstop1, 5);
register_bit!(a9_cpu_rst_ctrl, a9_clkstop0, 4);
register_bit!(a9_cpu_rst_ctrl, a9_rst1, 1);
register_bit!(a9_cpu_rst_ctrl, a9_rst0, 0);

pub fn reboot() {
    RegisterBlock::unlocked(|slcr| {
        unsafe {
            let reboot = slcr.reboot_status.read();
            slcr.reboot_status.write(reboot & 0xF0FFFFFF);
            slcr.pss_rst_ctrl.modify(|_, w| w.soft_rst(true));
        }
    });
}


#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum BootModePins {
    // CAUTION!
    // The BOOT_MODE bits table 6-4 in UG585 are *out of order*.
    Jtag = 0b000,
    Nor = 0b010,
    Nand = 0b100,
    QuadSpi = 0b001,
    SdCard = 0b101,
}

register!(boot_mode, BootMode, RO, u32);
register_bit!(boot_mode, pll_bypass, 4);
register_bit!(boot_mode, jtag_routing, 3);
register_bits_typed!(boot_mode, boot_mode_pins, u8, BootModePins, 0, 2);

register!(pss_rst_ctrl, PssRstCtrl, RW, u32);
register_bit!(pss_rst_ctrl, soft_rst, 0);

/// Used for MioPin*.io_type
#[repr(u8)]
pub enum IoBufferType {
    Lvcmos18 = 0b001,
    Lvcmos25 = 0b010,
    Lvcmos33 = 0b011,
    Hstl     = 0b100,
}

macro_rules! mio_pin_register {
    ($mod_name: ident, $struct_name: ident) => (
        register!($mod_name, $struct_name, RW, u32);
        register_bit!($mod_name, disable_rcvr, 13);
        register_bit!($mod_name, pullup, 12);
        register_bits_typed!($mod_name, io_type, u8, IoBufferType, 9, 11);
        register_bit!($mod_name, speed, 8);
        register_bits!($mod_name, l3_sel, u8, 5, 7);
        register_bits!($mod_name, l2_sel, u8, 3, 4);
        register_bit!($mod_name, l1_sel, 2);
        register_bit!($mod_name, l0_sel, 1);
        register_bit!($mod_name, tri_enable, 0);
    );
}

mio_pin_register!(mio_pin_00, MioPin00);
mio_pin_register!(mio_pin_01, MioPin01);
mio_pin_register!(mio_pin_02, MioPin02);
mio_pin_register!(mio_pin_03, MioPin03);
mio_pin_register!(mio_pin_04, MioPin04);
mio_pin_register!(mio_pin_05, MioPin05);
mio_pin_register!(mio_pin_06, MioPin06);
mio_pin_register!(mio_pin_07, MioPin07);
mio_pin_register!(mio_pin_08, MioPin08);
mio_pin_register!(mio_pin_09, MioPin09);
mio_pin_register!(mio_pin_10, MioPin10);
mio_pin_register!(mio_pin_11, MioPin11);
mio_pin_register!(mio_pin_12, MioPin12);
mio_pin_register!(mio_pin_13, MioPin13);
mio_pin_register!(mio_pin_14, MioPin14);
mio_pin_register!(mio_pin_15, MioPin15);
mio_pin_register!(mio_pin_16, MioPin16);
mio_pin_register!(mio_pin_17, MioPin17);
mio_pin_register!(mio_pin_18, MioPin18);
mio_pin_register!(mio_pin_19, MioPin19);
mio_pin_register!(mio_pin_20, MioPin20);
mio_pin_register!(mio_pin_21, MioPin21);
mio_pin_register!(mio_pin_22, MioPin22);
mio_pin_register!(mio_pin_23, MioPin23);
mio_pin_register!(mio_pin_24, MioPin24);
mio_pin_register!(mio_pin_25, MioPin25);
mio_pin_register!(mio_pin_26, MioPin26);
mio_pin_register!(mio_pin_27, MioPin27);
mio_pin_register!(mio_pin_28, MioPin28);
mio_pin_register!(mio_pin_29, MioPin29);
mio_pin_register!(mio_pin_30, MioPin30);
mio_pin_register!(mio_pin_31, MioPin31);
mio_pin_register!(mio_pin_32, MioPin32);
mio_pin_register!(mio_pin_33, MioPin33);
mio_pin_register!(mio_pin_34, MioPin34);
mio_pin_register!(mio_pin_35, MioPin35);
mio_pin_register!(mio_pin_36, MioPin36);
mio_pin_register!(mio_pin_37, MioPin37);
mio_pin_register!(mio_pin_38, MioPin38);
mio_pin_register!(mio_pin_39, MioPin39);
mio_pin_register!(mio_pin_40, MioPin40);
mio_pin_register!(mio_pin_41, MioPin41);
mio_pin_register!(mio_pin_42, MioPin42);
mio_pin_register!(mio_pin_43, MioPin43);
mio_pin_register!(mio_pin_44, MioPin44);
mio_pin_register!(mio_pin_45, MioPin45);
mio_pin_register!(mio_pin_46, MioPin46);
mio_pin_register!(mio_pin_47, MioPin47);
mio_pin_register!(mio_pin_48, MioPin48);
mio_pin_register!(mio_pin_49, MioPin49);
mio_pin_register!(mio_pin_50, MioPin50);
mio_pin_register!(mio_pin_51, MioPin51);
mio_pin_register!(mio_pin_52, MioPin52);
mio_pin_register!(mio_pin_53, MioPin53);

register!(lvl_shftr, LvlShftr, RW, u32);
register_bits_typed!(lvl_shftr, user_lvl_shftr_en, u8, LevelShifterEnable, 0, 3);

register!(gpiob_ctrl, GpiobCtrl, RW, u32);
register_bit!(gpiob_ctrl, vref_en, 0);

register!(ddriob_config, DdriobConfig, RW, u32);
register_bits_typed!(ddriob_config, inp_type, u8, DdriobInputType, 1, 2);
register_bit!(ddriob_config, dci_update_b, 3);
register_bit!(ddriob_config, term_en, 4);
register_bits_typed!(ddriob_config, dci_type, u8, DdriobDciType, 5, 6);
register_bit!(ddriob_config, ibuf_disable_mode, 7);
register_bit!(ddriob_config, term_disable_mode, 8);
register_bits_typed!(ddriob_config, output_en, u8, DdriobOutputEn, 9, 10);
register_bit!(ddriob_config, pullup_en, 11);

register!(ddriob_ddr_ctrl, DdriobDdrCtrl, RW, u32);
register_bit!(ddriob_ddr_ctrl, vref_int_en, 0);
register_bits_typed!(ddriob_ddr_ctrl, vref_sel, u8, DdriobVrefSel, 1, 4);
register_bit!(ddriob_ddr_ctrl, vref_ext_en_lower, 5);
register_bit!(ddriob_ddr_ctrl, vref_ext_en_upper, 6);
register_bit!(ddriob_ddr_ctrl, refio_en, 9);

register!(ddriob_dci_ctrl, DdriobDciCtrl, RW, u32);
register_bit!(ddriob_dci_ctrl, reset, 0);
register_bit!(ddriob_dci_ctrl, enable, 1);
register_bits!(ddriob_dci_ctrl, nref_opt1, u8, 6, 7);
register_bits!(ddriob_dci_ctrl, nref_opt2, u8, 8, 10);
register_bits!(ddriob_dci_ctrl, nref_opt4, u8, 11, 13);
register_bits!(ddriob_dci_ctrl, pref_opt1, u8, 14, 15);
register_bits!(ddriob_dci_ctrl, pref_opt2, u8, 17, 19);
register_bit!(ddriob_dci_ctrl, update_control, 20);

register!(ddriob_dci_status, DdriobDciStatus, RW, u32);
register_bit!(ddriob_dci_status, done, 0);
register_bit!(ddriob_dci_status, lock, 13);
