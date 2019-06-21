use crate::regs::*;
use crate::slcr;

pub mod phy;
mod regs;
pub mod rx;
pub mod tx;

/// Size of all the buffers
pub const MTU: usize = 1536;

pub struct Eth<RX, TX> {
    regs: &'static mut regs::RegisterBlock,
    rx: RX,
    tx: TX,
}

impl Eth<(), ()> {
    pub fn default(macaddr: [u8; 6]) -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            // Manual example: 0x0000_1280
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
                    .l3_sel(0b100)
                    .io_type(slcr::IoBufferType::Lvcmos18)
                    .pullup(true)
            );
            // Manual example: 0x0000_3902
            // TX_CLK
            slcr.mio_pin_16.write(
                slcr::MioPin16::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TX_CTRL
            slcr.mio_pin_21.write(
                slcr::MioPin21::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD3
            slcr.mio_pin_20.write(
                slcr::MioPin20::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD2
            slcr.mio_pin_19.write(
                slcr::MioPin19::zeroed()
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD1
            slcr.mio_pin_18.write(
                slcr::MioPin18::zeroed()
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD0
            slcr.mio_pin_17.write(
                slcr::MioPin17::zeroed()
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // Manual example: 0x0000_1903
            // RX_CLK
            slcr.mio_pin_22.write(
                slcr::MioPin22::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
            );
            // RX_CTRL
            slcr.mio_pin_27.write(
                slcr::MioPin27::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
            );
            // RXD3
            slcr.mio_pin_26.write(
                slcr::MioPin26::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
            );
            // RXD2
            slcr.mio_pin_25.write(
                slcr::MioPin25::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
            );
            // RXD1
            slcr.mio_pin_24.write(
                slcr::MioPin24::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
            );
            // RXD0
            slcr.mio_pin_23.write(
                slcr::MioPin23::zeroed()
                    .tri_enable(true)
                    .l0_sel(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
            );
            // VREF internal generator
            slcr.gpiob_ctrl.write(
                slcr::GpiobCtrl::zeroed()
                    .vref_en(true)
            );
        });

        Self::gem0(macaddr)
    }

    pub fn gem0(macaddr: [u8; 6]) -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.gem0_clk_ctrl.write(
                // 0x0050_0801: 8, 5: 100 Mb/s
                slcr::ClkCtrl::zeroed()
                    .clkact(true)
                    .srcsel(slcr::PllSource::IoPll)
                    .divisor(8)
                    .divisor1(5)
            );
            // Enable gem0 ref clock
            slcr.gem0_rclk_ctrl.write(
                // 0x0000_0801
                slcr::RclkCtrl::zeroed()
                    .clkact(true)
            );
        });

        let regs = regs::RegisterBlock::gem0();
        Self::from_regs(regs, macaddr)
    }

    pub fn gem1(macaddr: [u8; 6]) -> Self {
        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.gem1_clk_ctrl.write(
                // 0x0050_0801: 8, 5: 100 Mb/s
                slcr::ClkCtrl::zeroed()
                    .clkact(true)
                    .srcsel(slcr::PllSource::IoPll)
                    .divisor(8)
                    .divisor1(5)
            );
            // Enable gem1 ref clock
            slcr.gem1_rclk_ctrl.write(
                slcr::RclkCtrl::zeroed()
                    .clkact(true)
            );
        });

        let regs = regs::RegisterBlock::gem1();
        Self::from_regs(regs, macaddr)
    }

    fn from_regs(regs: &'static mut regs::RegisterBlock, macaddr: [u8; 6]) -> Self {
        let mut eth = Eth {
            regs,
            rx: (),
            tx: (),
        }.init();
        eth.configure(macaddr);
        eth
    }
}

impl<RX, TX> Eth<RX, TX> {
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
                // Promiscuous mode (TODO?)
                .copy_all(true)
                // Remove 4-byte Frame CheckSum
                .fcs_remove(true)
                // One of the slower speeds
                // TODO: calculate properly
                .mdc_clk_div(0b110)
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
                // 1536 bytes
                .ahb_mem_rx_buf_size((MTU >> 6) as u8)
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
        );
    }

    pub fn start_rx<'rx>(self, rx_list: &'rx mut [rx::DescEntry], rx_buffers: &'rx mut [[u8; MTU]]) -> Eth<rx::DescList<'rx>, TX> {
        let new_self = Eth {
            regs: self.regs,
            rx: rx::DescList::new(rx_list, rx_buffers),
            tx: self.tx,
        };
        let list_addr = new_self.rx.list_addr();
        assert!(list_addr & 0b11 == 0);
        new_self.regs.rx_qbar.write(
            regs::RxQbar::zeroed()
                .rx_q_baseaddr(list_addr >> 2)
        );
        new_self.regs.net_ctrl.modify(|_, w|
            w.rx_en(true)
        );
        new_self
    }

    // pub fn start_tx<'tx>(self, tx_buffers: [&'tx [u8]; tx::DESCS]) -> Eth<RX, tx::DescList<'tx>> {
    //     let new_self = Eth {
    //         regs: self.regs,
    //         rx: self.rx,
    //         tx: tx::DescList::new(tx_buffers),
    //     };
    //     let list_addr = &new_self.tx as *const _ as u32;
    //     assert!(list_addr & 0b11 == 0);
    //     new_self.regs.tx_qbar.write(
    //         regs::TxQbar::zeroed()
    //             .tx_q_baseaddr(list_addr >> 2)
    //     );
    //     new_self.regs.net_ctrl.modify(|_, w|
    //         w.tx_en(true)
    //     );
    //     new_self
    // }

    fn wait_phy_idle(&self) {
        while !self.regs.net_status.read().phy_mgmt_idle() {}
    }
}

impl<'rx, TX> Eth<rx::DescList<'rx>, TX> {
    pub fn recv_next<'s: 'p, 'p>(&'s mut self) -> Result<Option<rx::PktRef<'p>>, rx::Error> {
        let status = self.regs.rx_status.read();
        if status.hresp_not_ok() {
            // Clear
            self.regs.rx_status.write(
                regs::RxStatus::zeroed()
                    .hresp_not_ok(true)
            );
            return Err(rx::Error::HrespNotOk);
        }
        if status.rx_overrun() {
            // Clear
            self.regs.rx_status.write(
                regs::RxStatus::zeroed()
                    .rx_overrun(true)
            );
            return Err(rx::Error::RxOverrun);
        }
        if status.buffer_not_avail() {
            // Clear
            self.regs.rx_status.write(
                regs::RxStatus::zeroed()
                    .buffer_not_avail(true)
            );
            return Err(rx::Error::BufferNotAvail);
        }

        if status.frame_recd() {
            let result = self.rx.recv_next();
            match result {
                Ok(None) => {
                    // No packet, clear status bit
                    self.regs.rx_status.write(
                        regs::RxStatus::zeroed()
                            .frame_recd(true)
                    );
                }
                _ => {}
            }
            result
        } else {
            Ok(None)
        }
    }
}

impl<RX, TX> phy::PhyAccess for Eth<RX, TX> {
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
