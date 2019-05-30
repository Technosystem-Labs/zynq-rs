use crate::regs::*;
use crate::slcr;

mod regs;

pub struct Eth {
    regs: &'static mut regs::RegisterBlock,
}

impl Eth {
    pub fn default() -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            // MDIO
            slcr.mio_pin_53.write(
                slcr::MioPin53::zeroed()
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // MDC
            slcr.mio_pin_52.write(
                slcr::MioPin52::zeroed()
                    .tri_enable(true)
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // TX_CLK
            slcr.mio_pin_16.write(
                slcr::MioPin16::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // TX_CTRL
            slcr.mio_pin_21.write(
                slcr::MioPin21::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // TXD3
            slcr.mio_pin_20.write(
                slcr::MioPin20::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // TXD2
            slcr.mio_pin_19.write(
                slcr::MioPin19::zeroed()
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // TXD1
            slcr.mio_pin_18.write(
                slcr::MioPin18::zeroed()
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // TXD0
            slcr.mio_pin_17.write(
                slcr::MioPin17::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // RX_CLK
            slcr.mio_pin_22.write(
                slcr::MioPin22::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // RX_CTRL
            slcr.mio_pin_27.write(
                slcr::MioPin27::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // RXD3
            slcr.mio_pin_26.write(
                slcr::MioPin26::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // RXD2
            slcr.mio_pin_25.write(
                slcr::MioPin25::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // RXD1
            slcr.mio_pin_24.write(
                slcr::MioPin24::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // RXD0
            slcr.mio_pin_23.write(
                slcr::MioPin23::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
        });

        Self::gem0()
    }

    pub fn gem0() -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            // Enable gem0 ref clock
            slcr.gem0_rclk_ctrl.write(
                slcr::RclkCtrl::zeroed()
                    .clkact(true)
            );
            slcr.gem0_clk_ctrl.write(
                slcr::ClkCtrl::zeroed()
                    .clkact(true)
                    .srcsel(slcr::PllSource::IoPll)
                    .divisor(10)
            );
        });

        let regs = regs::RegisterBlock::gem0();
        Eth { regs }.init()
    }

    pub fn gem1() -> Self {
        let regs = regs::RegisterBlock::gem1();
        Eth { regs }.init()
    }

    fn init(self) -> Self {
        // Clear the Network Control register.
        self.regs.net_ctrl.write(regs::NetCtrl::zeroed());
        self.regs.net_ctrl.write(regs::NetCtrl::zeroed().clear_stat_regs(true));
        // Clear the Status registers.
        self.regs.rx_status.write(
            regs::RxStatus::zeroed()
                .buffer_not_avail(true)
                .frame_recd(true)
                .rx_overrun(true)
                .hresp_not_ok(true)
        );
        self.regs.tx_status.write(
            regs::TxStatus::zeroed()
                .used_bit_read(true)
                .collision(true)
                .retry_limit_exceeded(true)
                .tx_go(true)
                .tx_corr_ahb_err(true)
                .tx_complete(true)
                .tx_under_run(true)
                .late_collision(true)
                // not in the manual:
                .hresp_not_ok(true)
        );
        // Disable all interrupts.
        self.regs.intr_dis.write(
            regs::IntrDis::zeroed()
                .mgmt_done(true)
                .rx_complete(true)
                .rx_used_read(true)
                .tx_used_read(true)
                .tx_underrun(true)
                .retry_ex_late_collisn(true)
                .tx_corrupt_ahb_err(true)
                .tx_complete(true)
                .link_chng(true)
                .rx_overrun(true)
                .hresp_not_ok(true)
                .pause_nonzeroq(true)
                .pause_zero(true)
                .pause_tx(true)
                .ex_intr(true)
                .autoneg_complete(true)
                .partner_pg_rx(true)
                .delay_req_rx(true)
                .sync_rx(true)
                .delay_req_tx(true)
                .sync_tx(true)
                .pdelay_req_rx(true)
                .pdelay_resp_rx(true)
                .pdelay_req_tx(true)
                .pdelay_resp_tx(true)
                .tsu_sec_incr(true)
        );
        // Clear the buffer queues.
        self.regs.rx_qbar.write(
            regs::RxQbar::zeroed()
        );
        self.regs.tx_qbar.write(
            regs::TxQbar::zeroed()
        );

        self
    }

    fn configure(&mut self) {
        self.regs.net_cfg.write(
            regs::NetCfg::zeroed()
                .full_duplex(true)
                .gige_en(true)
                .speed(true)
                .no_broadcast(false)
                .multi_hash_en(true)
                // Promiscuous mode
                .copy_all(true)
                .mdc_clk_div(0b111)
        );
    }
}
