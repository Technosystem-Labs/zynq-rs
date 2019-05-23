///! Register definitions for System Level Control

use volatile_register::{RO, WO, RW};
use crate::{register, register_bit, register_bits, register_at, regs::RegisterW, regs::RegisterRW};

pub enum PllSource {
    IoPll  = 0b00,
    ArmPll = 0b10,
    DdrPll = 0b11,
}

#[repr(C)]
pub struct RegisterBlock {
    pub scl: RW<u32>,
    pub slcr_lock: SlcrLock,
    pub slcr_unlock: SlcrUnlock,
    pub slcr_locksta: RO<u32>,
    reserved0: [u32; 60],
    pub arm_pll_ctrl: RW<u32>,
    pub ddr_pll_ctrl: RW<u32>,
    pub io_pll_ctrl: RW<u32>,
    pub pll_status: RO<u32>,
    pub arm_pll_cfg: RW<u32>,
    pub ddr_pll_cfg: RW<u32>,
    pub io_pll_cfg: RW<u32>,
    reserved1: [u32; 1],
    pub arm_clk_ctrl: RW<u32>,
    pub ddr_clk_ctrl: RW<u32>,
    pub dci_clk_ctrl: RW<u32>,
    pub aper_clk_ctrl: AperClkCtrl,
    pub usb0_clk_ctrl: RW<u32>,
    pub usb1_clk_ctrl: RW<u32>,
    pub gem0_rclk_ctrl: RW<u32>,
    pub gem1_rclk_ctrl: RW<u32>,
    pub gem0_clk_ctrl: RW<u32>,
    pub gem1_clk_ctrl: RW<u32>,
    pub smc_clk_ctrl: RW<u32>,
    pub lqspi_clk_ctrl: RW<u32>,
    pub sdio_clk_ctrl: RW<u32>,
    pub uart_clk_ctrl: UartClkCtrl,
    pub spi_clk_ctrl: RW<u32>,
    pub can_clk_ctrl: RW<u32>,
    pub can_mioclk_ctrl: RW<u32>,
    pub dbg_clk_ctrl: RW<u32>,
    pub pcap_clk_ctrl: RW<u32>,
    pub topsw_clk_ctrl: RW<u32>,
    pub fpga0_clk_ctrl: RW<u32>,
    pub fpga0_thr_ctrl: RW<u32>,
    pub fpga0_thr_cnt: RW<u32>,
    pub fpga0_thr_sta: RO<u32>,
    pub fpga1_clk_ctrl: RW<u32>,
    pub fpga1_thr_ctrl: RW<u32>,
    pub fpga1_thr_cnt: RW<u32>,
    pub fpga1_thr_sta: RO<u32>,
    pub fpga2_clk_ctrl: RW<u32>,
    pub fpga2_thr_ctrl: RW<u32>,
    pub fpga2_thr_cnt: RW<u32>,
    pub fpga2_thr_sta: RO<u32>,
    pub fpga3_clk_ctrl: RW<u32>,
    pub fpga3_thr_ctrl: RW<u32>,
    pub fpga3_thr_cnt: RW<u32>,
    pub fpga3_thr_sta: RO<u32>,
    reserved2: [u32; 5],
    pub clk_621_true: RW<u32>,
    reserved3: [u32; 14],
    pub pss_rst_ctrl: RW<u32>,
    pub ddr_rst_ctrl: RW<u32>,
    pub topsw_rst_ctrl: RW<u32>,
    pub dmac_rst_ctrl: RW<u32>,
    pub usb_rst_ctrl: RW<u32>,
    pub gem_rst_ctrl: RW<u32>,
    pub sdio_rst_ctrl: RW<u32>,
    pub spi_rst_ctrl: RW<u32>,
    pub can_rst_ctrl: RW<u32>,
    pub i2c_rst_ctrl: RW<u32>,
    pub uart_rst_ctrl: UartRstCtrl,
    pub gpio_rst_ctrl: RW<u32>,
    pub lqspi_rst_ctrl: RW<u32>,
    pub smc_rst_ctrl: RW<u32>,
    pub ocm_rst_ctrl: RW<u32>,
    reserved4: [u32; 1],
    pub fpga_rst_ctrl: RW<u32>,
    pub a9_cpu_rst_ctrl: RW<u32>,
    reserved5: [u32; 1],
    pub rs_awdt_ctrl: RW<u32>,
    reserved6: [u32; 2],
    pub reboot_status: RW<u32>,
    pub boot_mode: RW<u32>,
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
    pub mio_pin_00: RW<u32>,
    pub mio_pin_01: RW<u32>,
    pub mio_pin_02: RW<u32>,
    pub mio_pin_03: RW<u32>,
    pub mio_pin_04: RW<u32>,
    pub mio_pin_05: RW<u32>,
    pub mio_pin_06: RW<u32>,
    pub mio_pin_07: RW<u32>,
    pub mio_pin_08: RW<u32>,
    pub mio_pin_09: RW<u32>,
    pub mio_pin_10: RW<u32>,
    pub mio_pin_11: RW<u32>,
    pub mio_pin_12: RW<u32>,
    pub mio_pin_13: RW<u32>,
    pub mio_pin_14: RW<u32>,
    pub mio_pin_15: RW<u32>,
    pub mio_pin_16: RW<u32>,
    pub mio_pin_17: RW<u32>,
    pub mio_pin_18: RW<u32>,
    pub mio_pin_19: RW<u32>,
    pub mio_pin_20: RW<u32>,
    pub mio_pin_21: RW<u32>,
    pub mio_pin_22: RW<u32>,
    pub mio_pin_23: RW<u32>,
    pub mio_pin_24: RW<u32>,
    pub mio_pin_25: RW<u32>,
    pub mio_pin_26: RW<u32>,
    pub mio_pin_27: RW<u32>,
    pub mio_pin_28: RW<u32>,
    pub mio_pin_29: RW<u32>,
    pub mio_pin_30: RW<u32>,
    pub mio_pin_31: RW<u32>,
    pub mio_pin_32: RW<u32>,
    pub mio_pin_33: RW<u32>,
    pub mio_pin_34: RW<u32>,
    pub mio_pin_35: RW<u32>,
    pub mio_pin_36: RW<u32>,
    pub mio_pin_37: RW<u32>,
    pub mio_pin_38: RW<u32>,
    pub mio_pin_39: RW<u32>,
    pub mio_pin_40: RW<u32>,
    pub mio_pin_41: RW<u32>,
    pub mio_pin_42: RW<u32>,
    pub mio_pin_43: RW<u32>,
    pub mio_pin_44: RW<u32>,
    pub mio_pin_45: RW<u32>,
    pub mio_pin_46: RW<u32>,
    pub mio_pin_47: RW<u32>,
    pub mio_pin_48: MioPin48,
    pub mio_pin_49: MioPin49,
    pub mio_pin_50: RW<u32>,
    pub mio_pin_51: RW<u32>,
    pub mio_pin_52: RW<u32>,
    pub mio_pin_53: RW<u32>,
    reserved14: [u32; 11],
    pub mio_loopback: RW<u32>,
    reserved15: [u32; 1],
    pub mio_mst_tri0: RW<u32>,
    pub mio_mst_tri1: RW<u32>,
    reserved16: [u32; 7],
    pub sd0_wp_cd_sel: RW<u32>,
    pub sd1_wp_cd_sel: RW<u32>,
    reserved17: [u32; 50],
    pub lvl_shftr_en: RW<u32>,
    reserved18: [u32; 3],
    pub ocm_cfg: RW<u32>,
    reserved19: [u32; 123],
    pub gpiob_ctrl: RW<u32>,
    pub gpiob_cfg_cmos18: RW<u32>,
    pub gpiob_cfg_cmos25: RW<u32>,
    pub gpiob_cfg_cmos33: RW<u32>,
    reserved20: [u32; 1],
    pub gpiob_cfg_hstl: RW<u32>,
    pub gpiob_drvr_bias_ctrl: RW<u32>,
    reserved21: [u32; 9],
    pub ddriob_addr1: RW<u32>,
    pub ddriob_data0: RW<u32>,
    pub ddriob_data1: RW<u32>,
    pub ddriob_diff0: RW<u32>,
    pub ddriob_diff1: RW<u32>,
    pub ddriob_clock: RW<u32>,
    pub w_addr: RW<u32>,
    pub w_data: RW<u32>,
    pub w_diff: RW<u32>,
    pub w_clock: RW<u32>,
    pub ddriob_ddr_ctrl: RW<u32>,
    pub ddriob_dci_ctrl: RW<u32>,
    pub ddriob_dci_status: RW<u32>,
}
register_at!(RegisterBlock, 0xF8000000, new);

impl RegisterBlock {
    pub fn unlocked<F: FnMut(&Self) -> R, R>(mut f: F) -> R {
        let self_ = Self::new();
        self_.slcr_unlock.unlock();
        let r = f(&self_);
        self_.slcr_lock.lock();
        r
    }
}

register!(slcr_lock, SlcrLock, WO, u32);
register_bits!(slcr_lock, lock_key, u16, 0, 15);
impl SlcrLock {
    pub fn lock(&self) {
        self.write(
            Self::zeroed()
                .lock_key(0x767B)
        );
    }
}

register!(slcr_unlock, SlcrUnlock, WO, u32);
register_bits!(slcr_unlock, unlock_key, u16, 0, 15);
impl SlcrUnlock {
    pub fn unlock(&self) {
        self.write(
            Self::zeroed()
                .unlock_key(0xDF0D)
        );
    }
}

register!(aper_clk_ctrl, AperClkCtrl, RW, u32);
register_bit!(aper_clk_ctrl, uart1_cpu_1xclkact, 21);
register_bit!(aper_clk_ctrl, uart0_cpu_1xclkact, 20);
impl AperClkCtrl {
    pub fn enable_uart0(&self) {
        self.modify(|_, w| w.uart0_cpu_1xclkact(true));
    }

    pub fn enable_uart1(&self) {
        self.modify(|_, w| w.uart1_cpu_1xclkact(true));
    }
}


register!(uart_clk_ctrl, UartClkCtrl, RW, u32);
register_bit!(uart_clk_ctrl, clkact0, 0);
register_bit!(uart_clk_ctrl, clkact1, 1);
register_bits!(uart_clk_ctrl, divisor, u8, 8, 13);
register_bits!(uart_clk_ctrl, srcsel, u8, 4, 5);
register_at!(UartClkCtrl, 0xF8000154, new);
impl UartClkCtrl {
    pub fn enable_uart0(&self) {
        self.modify(|_, w| {
            // a. Clock divisor, slcr.UART_CLK_CTRL[DIVISOR] = 0x14.
            // b. Select the IO PLL, slcr.UART_CLK_CTRL[SRCSEL] = 0.
            // c. Enable the UART 0 Reference clock, slcr.UART_CLK_CTRL [CLKACT0] = 1.
            w.divisor(0x14)
             .srcsel(PllSource::IoPll as u8)
             .clkact0(true)
        })
    }

    pub fn enable_uart1(&self) {
        self.modify(|_, w| {
            // a. Clock divisor, slcr.UART_CLK_CTRL[DIVISOR] = 0x14.
            // b. Select the IO PLL, slcr.UART_CLK_CTRL[SRCSEL] = 0.
            // c. Enable the UART 1 Reference clock, slcr.UART_CLK_CTRL [CLKACT1] = 1.
            w.divisor(0x14)
             .srcsel(PllSource::IoPll as u8)
             .clkact1(true)
        })
    }
}

register!(uart_rst_ctrl, UartRstCtrl, RW, u32);
register_bit!(uart_rst_ctrl, uart0_ref_rst, 3);
register_bit!(uart_rst_ctrl, uart1_ref_rst, 2);
register_bit!(uart_rst_ctrl, uart0_cpu1x_rst, 1);
register_bit!(uart_rst_ctrl, uart1_cpu1x_rst, 0);
register_at!(UartRstCtrl, 0xF8000228, new);
impl UartRstCtrl {
    pub fn reset_uart0(&self) {
        self.modify(|_, w|
            w.uart0_ref_rst(true)
             .uart0_cpu1x_rst(true)
        );
        self.modify(|_, w|
            w.uart0_ref_rst(false)
             .uart0_cpu1x_rst(false)
        );
    }

    pub fn reset_uart1(&self) {
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

/// Used for MioPin*.io_type
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
        register_bits!($mod_name, io_type, u8, 9, 11);
        register_bit!($mod_name, speed, 8);
        register_bits!($mod_name, l3_sel, u8, 5, 7);
        register_bits!($mod_name, l2_sel, u8, 3, 4);
        register_bit!($mod_name, l1_sel, 2);
        register_bit!($mod_name, l0_sel, 1);
        register_bit!($mod_name, tri_enable, 0);
    );
}

mio_pin_register!(mio_pin_48, MioPin48);
mio_pin_register!(mio_pin_49, MioPin49);
