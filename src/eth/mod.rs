use crate::regs::*;
use crate::slcr;

pub mod phy;
mod regs;
mod rx;
mod tx;

pub struct Eth<'rx> {
    regs: &'static mut regs::RegisterBlock,
    rx: Option<rx::DescList<'rx>>,
}

impl<'rx> Eth<'rx> {
    pub fn default(macaddr: [u8; 6]) -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            // MDIO
            slcr.mio_pin_53.write(
                slcr::MioPin53::zeroed()
                    .tri_enable(true)
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // MDC
            slcr.mio_pin_52.write(
                slcr::MioPin52::zeroed()
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

        Self::gem0(macaddr)
    }

    pub fn gem0(macaddr: [u8; 6]) -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            // Enable gem0 ref clock
            slcr.gem0_rclk_ctrl.write(
                slcr::RclkCtrl::zeroed()
                    .clkact(true)
            );
            // 0x0050_0801: 8, 5: 100 Mb/s
            slcr.gem0_clk_ctrl.write(
                slcr::ClkCtrl::zeroed()
                    .clkact(true)
                    .srcsel(slcr::PllSource::IoPll)
                    .divisor(8)
                    .divisor1(5)
            );
        });

        let regs = regs::RegisterBlock::gem0();
        let rx = None;
        let mut eth = Eth { regs, rx }.init();
        eth.configure(macaddr);
        eth
    }

    pub fn gem1(macaddr: [u8; 6]) -> Self {
        let regs = regs::RegisterBlock::gem1();
        let rx = None;
        let mut eth = Eth { regs, rx }.init();
        eth.configure(macaddr);
        eth
    }

    fn init(mut self) -> Self {
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

    fn configure(&mut self, macaddr: [u8; 6]) {
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

        let macaddr_msbs =
            (u16::from(macaddr[0]) << 8) |
            u16::from(macaddr[1]);
        let macaddr_lsbs =
            (u32::from(macaddr[2]) << 24) |
            (u32::from(macaddr[3]) << 16) |
            (u32::from(macaddr[4]) << 8) |
            u32::from(macaddr[5]);
        self.regs.spec_addr1_top.write(
            regs::SpecAddrTop::zeroed()
                .addr_msbs(macaddr_msbs)
        );
        self.regs.spec_addr1_bot.write(
            regs::SpecAddrBot::zeroed()
                .addr_lsbs(macaddr_lsbs)
        );


        self.regs.dma_cfg.write(
            regs::DmaCfg::zeroed()
                // 1600 bytes
                .ahb_mem_rx_buf_size(0x19)
                // 8 KB
                .rx_pktbuf_memsz_sel(0x3)
                // 4 KB
                .tx_pktbuf_memsz_sel(true)
                // .csum_gen_offload_en(true)
                // Little-endian
                .ahb_endian_swp_mgmt_en(false)
                // INCR16 AHB burst
                .ahb_fixed_burst_len(0x10)
        );

        self.regs.net_ctrl.write(
            regs::NetCtrl::zeroed()
                .mgmt_port_en(true)
                .tx_en(true)
                .rx_en(true)
        );
    }

    pub fn start_rx(&mut self, rx_buffers: [&'rx mut [u8]; rx::DESCS]) {
        self.rx = Some(rx::DescList::new(rx_buffers));
        let list_addr = self.rx.as_ref().unwrap() as *const _ as u32;
        self.regs.rx_qbar.write(
            regs::RxQbar::zeroed()
                .rx_q_baseaddr(list_addr >> 2)
        );
    }

    fn wait_phy_idle(&self) {
        while !self.regs.net_status.read().phy_mgmt_idle() {}
    }
}

impl<'rx> phy::PhyAccess for Eth<'rx> {
    fn read_phy(&mut self, addr: u8, reg: u8) -> u16 {
        self.wait_phy_idle();
        self.regs.phy_maint.write(
            regs::PhyMaint::zeroed()
                .clause_22(true)
                .operation(regs::PhyOperation::Read)
                .phy_addr(addr)
                .reg_addr(reg)
                .must_10(0b10)
        );
        self.wait_phy_idle();
        self.regs.phy_maint.read().data()
    }

    fn write_phy(&mut self, addr: u8, reg: u8, data: u16) {
        self.wait_phy_idle();
        self.regs.phy_maint.write(
            regs::PhyMaint::zeroed()
                .clause_22(true)
                .operation(regs::PhyOperation::Write)
                .phy_addr(addr)
                .reg_addr(reg)
                .must_10(0b10)
                .data(data)
        );
        self.wait_phy_idle();
    }
}
