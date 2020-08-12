use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use log::{debug, info, warn, error};
use libregister::*;
use super::slcr;
use super::clocks::Clocks;

pub mod phy;
use phy::{Phy, PhyAccess};
mod regs;
pub mod rx;
pub mod tx;

/// Size of all the buffers
pub const MTU: usize = 1536;
/// Maximum MDC clock
const MAX_MDC: u32 = 2_500_000;
const TX_10: u32 = 10_000_000;
const TX_100: u32 = 25_000_000;
/// Clock for GbE
const TX_1000: u32 = 125_000_000;

#[derive(Clone)]
#[repr(C, align(0x20))]
pub struct Buffer(pub [u8; MTU]);

impl Buffer {
    pub fn new() -> Self {
        Buffer([0; MTU])
    }
}

impl Deref for Buffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

/// Gigabit Ethernet Peripheral
pub trait Gem {
    fn setup_clock(tx_clock: u32);
    fn regs() -> &'static mut regs::RegisterBlock;
}

/// first Gigabit Ethernet peripheral
pub struct Gem0;

impl Gem for Gem0 {
    fn setup_clock(tx_clock: u32) {
        let (divisor0, divisor1) = calculate_tx_divisors(tx_clock);

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.gem0_clk_ctrl.write(
                // 0x0050_0801: 8, 5: 100 Mb/s
                // ...: 8, 1: 1000 Mb/s
                slcr::GemClkCtrl::zeroed()
                    .clkact(true)
                    .srcsel(slcr::PllSource::IoPll)
                    .divisor(divisor0 as u8)
                    .divisor1(divisor1 as u8)
            );
            // Enable gem0 recv clock
            slcr.gem0_rclk_ctrl.write(
                // 0x0000_0801
                slcr::RclkCtrl::zeroed()
                    .clkact(true)
            );
        });
    }

    fn regs() -> &'static mut regs::RegisterBlock {
        regs::RegisterBlock::gem0()
    }
}

/// second Gigabit Ethernet peripheal
pub struct Gem1;

impl Gem for Gem1 {
    fn setup_clock(tx_clock: u32) {
        let (divisor0, divisor1) = calculate_tx_divisors(tx_clock);

        slcr::RegisterBlock::unlocked(|slcr| {
            slcr.gem1_clk_ctrl.write(
                slcr::GemClkCtrl::zeroed()
                    .clkact(true)
                    .srcsel(slcr::PllSource::IoPll)
                    .divisor(divisor0 as u8)
                    .divisor1(divisor1 as u8)
            );
            // Enable gem1 recv clock
            slcr.gem1_rclk_ctrl.write(
                // 0x0000_0801
                slcr::RclkCtrl::zeroed()
                    .clkact(true)
            );
        });
    }

    fn regs() -> &'static mut regs::RegisterBlock {
        regs::RegisterBlock::gem1()
    }
}

fn calculate_tx_divisors(tx_clock: u32) -> (u8, u8) {
    let io_pll = Clocks::get().io;
    let target = (tx_clock - 1 + io_pll) / tx_clock;

    let mut best = None;
    let mut best_error = 0;
    for divisor0 in 1..63 {
        for divisor1 in 1..63 {
            let current = (divisor0 as u32) * (divisor1 as u32);
            let error = if current > target {
                current - target
            } else {
                target - current
            };
            if best.is_none() || best_error > error {
                best = Some((divisor0, divisor1));
                best_error = error;
            }
        }
    }
    let result = best.unwrap();
    debug!("Eth TX clock for {}: {} / {} / {} = {}",
           tx_clock, io_pll,
           result.0, result.1,
           io_pll / result.0 as u32 / result.1 as u32
    );
    result
}

pub struct Eth<GEM: Gem, RX, TX> {
    rx: RX,
    tx: TX,
    inner: EthInner<GEM>,
    phy: Phy,
}

impl Eth<Gem0, (), ()> {
    pub fn eth0(macaddr: [u8; 6]) -> Self {
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
                    .speed(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TX_CTRL
            slcr.mio_pin_21.write(
                slcr::MioPin21::zeroed()
                    .l0_sel(true)
                    .speed(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD3
            slcr.mio_pin_20.write(
                slcr::MioPin20::zeroed()
                    .l0_sel(true)
                    .speed(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD2
            slcr.mio_pin_19.write(
                slcr::MioPin19::zeroed()
                    .l0_sel(true)
                    .speed(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD1
            slcr.mio_pin_18.write(
                slcr::MioPin18::zeroed()
                    .l0_sel(true)
                    .speed(true)
                    .io_type(slcr::IoBufferType::Hstl)
                    .pullup(true)
                    .disable_rcvr(true)
            );
            // TXD0
            slcr.mio_pin_17.write(
                slcr::MioPin17::zeroed()
                    .l0_sel(true)
                    .speed(true)
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
        Self::gem_common(macaddr)
    }
}


impl Eth<Gem1, (), ()> {
    // TODO: Add a `eth1()`

    pub fn gem1(macaddr: [u8; 6]) -> Self {
        Self::gem_common(macaddr)
    }
}


impl<GEM: Gem> Eth<GEM, (), ()> {
    fn gem_common(macaddr: [u8; 6]) -> Self {
        GEM::setup_clock(TX_1000);

        let mut inner = EthInner {
            gem: PhantomData,
            link: None,
        };
        inner.init();
        inner.configure(macaddr);

        let phy = Phy::find(&mut inner).expect("phy");
        phy.reset(&mut inner);
        phy.restart_autoneg(&mut inner);

        Eth {
            rx: (),
            tx: (),
            inner,
            phy,
        }
    }
}

impl<GEM: Gem, RX, TX> Eth<GEM, RX, TX> {
    pub fn start_rx(self, rx_size: usize) -> Eth<GEM, rx::DescList, TX> {
        let new_self = Eth {
            rx: rx::DescList::new(rx_size),
            tx: self.tx,
            inner: self.inner,
            phy: self.phy,
        };
        let list_addr = new_self.rx.list_addr();
        assert!(list_addr & 0b11 == 0);
        GEM::regs().rx_qbar.write(
            regs::RxQbar::zeroed()
                .rx_q_baseaddr(list_addr >> 2)
        );
        GEM::regs().net_ctrl.modify(|_, w|
            w.rx_en(true)
        );
        new_self
    }

    pub fn start_tx(self, tx_size: usize) -> Eth<GEM, RX, tx::DescList> {
        let new_self = Eth {
            rx: self.rx,
            tx: tx::DescList::new(tx_size),
            inner: self.inner,
            phy: self.phy,
        };
        let list_addr = &new_self.tx.list_addr();
        assert!(list_addr & 0b11 == 0);
        GEM::regs().tx_qbar.write(
            regs::TxQbar::zeroed()
                .tx_q_baseaddr(list_addr >> 2)
        );
        GEM::regs().net_ctrl.modify(|_, w|
            w.tx_en(true)
        );
        new_self
    }
}

impl<GEM: Gem, TX> Eth<GEM, rx::DescList, TX> {
    pub fn recv_next<'s: 'p, 'p>(&'s mut self) -> Result<Option<rx::PktRef<'p>>, rx::Error> {
        let status = GEM::regs().rx_status.read();
        if status.hresp_not_ok() {
            // Clear
            GEM::regs().rx_status.write(
                regs::RxStatus::zeroed()
                    .hresp_not_ok(true)
            );
            return Err(rx::Error::HrespNotOk);
        }
        if status.rx_overrun() {
            // Clear
            GEM::regs().rx_status.write(
                regs::RxStatus::zeroed()
                    .rx_overrun(true)
            );
            return Err(rx::Error::RxOverrun);
        }
        if status.buffer_not_avail() {
            // Clear
            GEM::regs().rx_status.write(
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
                    GEM::regs().rx_status.write(
                        regs::RxStatus::zeroed()
                            .frame_recd(true)
                    );
                }
                _ => {}
            }
            result
        } else {
            self.inner.check_link_change(&self.phy);
            Ok(None)
        }
    }
}

impl<GEM: Gem, RX> Eth<GEM, RX, tx::DescList> {
    pub fn send<'s: 'p, 'p>(&'s mut self, length: usize) -> Option<tx::PktRef<'p>> {
        self.tx.send(GEM::regs(), length)
    }
}

impl<'a, GEM: Gem> smoltcp::phy::Device<'a> for &mut Eth<GEM, rx::DescList, tx::DescList> {
    type RxToken = rx::PktRef<'a>;
    type TxToken = tx::Token<'a>;

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        use smoltcp::phy::{DeviceCapabilities, ChecksumCapabilities, Checksum};

        let mut checksum_caps = ChecksumCapabilities::default();
        checksum_caps.ipv4 = Checksum::Both;
        checksum_caps.tcp = Checksum::Both;
        checksum_caps.udp = Checksum::Both;

        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(self.rx.len().min(self.tx.len()));
        caps.checksum = checksum_caps;

        caps
    }

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        match self.rx.recv_next() {
            Ok(Some(pktref)) => {
                let tx_token = tx::Token {
                    regs: GEM::regs(),
                    desc_list: &mut self.tx,
                };
                Some((pktref, tx_token))
            }
            Ok(None) => {
                self.inner.check_link_change(&self.phy);
                None
            }
            Err(e) => {
                error!("eth recv error: {:?}", e);
                None
            }
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(tx::Token {
            regs: GEM::regs(),
            desc_list: &mut self.tx,
        })
    }
}


struct EthInner<GEM: Gem> {
    gem: PhantomData<GEM>,
    link: Option<phy::Link>,
}

impl<GEM: Gem> EthInner<GEM> {
    fn init(&mut self) {
        // Clear the Network Control register.
        GEM::regs().net_ctrl.write(regs::NetCtrl::zeroed());
        GEM::regs().net_ctrl.write(regs::NetCtrl::zeroed().clear_stat_regs(true));
        // Clear the Status registers.
        GEM::regs().rx_status.write(
            regs::RxStatus::zeroed()
                .buffer_not_avail(true)
                .frame_recd(true)
                .rx_overrun(true)
                .hresp_not_ok(true)
        );
        GEM::regs().tx_status.write(
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
        GEM::regs().intr_dis.write(
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
        GEM::regs().rx_qbar.write(
            regs::RxQbar::zeroed()
        );
        GEM::regs().tx_qbar.write(
            regs::TxQbar::zeroed()
        );
    }

    fn configure(&mut self, macaddr: [u8; 6]) {
        let clocks = Clocks::get();
        let mdc_clk_div = (clocks.cpu_1x() / MAX_MDC) + 1;

        GEM::regs().net_cfg.write(
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
                // RX checksum offload
                .rx_chksum_offld_en(true)
                // One of the slower speeds
                .mdc_clk_div((mdc_clk_div >> 4).min(0b111) as u8)
        );

        let macaddr_msbs =
            (u16::from(macaddr[0]) << 8) |
            u16::from(macaddr[1]);
        let macaddr_lsbs =
            (u32::from(macaddr[2]) << 24) |
            (u32::from(macaddr[3]) << 16) |
            (u32::from(macaddr[4]) << 8) |
            u32::from(macaddr[5]);
        GEM::regs().spec_addr1_top.write(
            regs::SpecAddrTop::zeroed()
                .addr_msbs(macaddr_msbs)
        );
        GEM::regs().spec_addr1_bot.write(
            regs::SpecAddrBot::zeroed()
                .addr_lsbs(macaddr_lsbs)
        );


        GEM::regs().dma_cfg.write(
            regs::DmaCfg::zeroed()
                // 1536 bytes
                .ahb_mem_rx_buf_size((MTU >> 6) as u8)
                // 8 KB
                .rx_pktbuf_memsz_sel(0x3)
                // 4 KB
                .tx_pktbuf_memsz_sel(true)
                // TX checksum offload
                .csum_gen_offload_en(true)
                // Little-endian
                .ahb_endian_swp_mgmt_en(false)
                // INCR16 AHB burst
                .ahb_fixed_burst_len(0x10)
        );

        GEM::regs().net_ctrl.write(
            regs::NetCtrl::zeroed()
                .mgmt_port_en(true)
        );
    }


    fn wait_phy_idle(&self) {
        while !GEM::regs().net_status.read().phy_mgmt_idle() {}
    }


    fn check_link_change(&mut self, phy: &Phy) {
        // As the PHY access takes some time, exit early if there was
        // already a link. TODO: check once per second.
        if self.link.is_some() {
            return
        }

        let link = phy.get_link(self);

        // Check link state transition
        if self.link != link {
            match &link {
                Some(link) => {
                    info!("eth: got {:?}", link);

                    use phy::{LinkDuplex::Full, LinkSpeed::*};
                    let txclock = match link.speed {
                        S10 => TX_10,
                        S100 => TX_100,
                        S1000 => TX_1000,
                    };
                    GEM::setup_clock(txclock);
                    GEM::regs().net_cfg.modify(|_, w| w
                        .full_duplex(link.duplex == Full)
                        .gige_en(link.speed == S1000)
                        .speed(link.speed != S10)
                    );
                }
                None => {
                    warn!("eth: link lost");
                    phy.modify_control(self, |control|
                                       control.set_autoneg_enable(true)
                                       .set_restart_autoneg(true)
                    );
                }
            }

            self.link = link;
        }
    }
}

impl<GEM: Gem> PhyAccess for EthInner<GEM> {
    fn read_phy(&mut self, addr: u8, reg: u8) -> u16 {
        self.wait_phy_idle();
        GEM::regs().phy_maint.write(
            regs::PhyMaint::zeroed()
                .clause_22(true)
                .operation(regs::PhyOperation::Read)
                .phy_addr(addr)
                .reg_addr(reg)
                .must_10(0b10)
        );
        self.wait_phy_idle();
        GEM::regs().phy_maint.read().data()
    }

    fn write_phy(&mut self, addr: u8, reg: u8, data: u16) {
        self.wait_phy_idle();
        GEM::regs().phy_maint.write(
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


