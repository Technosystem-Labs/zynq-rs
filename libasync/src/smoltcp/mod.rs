use core::{
    cell::RefCell,
    task::Waker,
};
use alloc::vec::Vec;
use smoltcp::{
    iface::EthernetInterface,
    phy::Device,
    socket::SocketSet,
    time::Instant,
};

mod tcp_stream;
pub use tcp_stream::TcpStream;

static mut SOCKETS: Option<Sockets> = None;

pub struct Sockets {
    sockets: RefCell<SocketSet<'static>>,
    wakers: RefCell<Vec<Waker>>,
}

impl Sockets {
    pub fn init(max_sockets: usize) {
        let mut sockets_storage = Vec::with_capacity(max_sockets);
        for _ in 0..max_sockets {
            sockets_storage.push(None);
        }
        let sockets = RefCell::new(SocketSet::new(sockets_storage));

        let wakers = RefCell::new(Vec::new());

        let instance = Sockets {
            sockets,
            wakers,
        };
        unsafe { SOCKETS = Some(instance); }
    }

    #[allow(static_mut_refs)]
    pub fn instance() -> &'static Self {
        unsafe { SOCKETS.as_ref().expect("Sockets") }
    }

    pub fn poll<'b, D: for<'d> Device<'d>>(
        &self,
        iface: &mut EthernetInterface<'b, D>,
        instant: Instant
    ) {
        let processed = {
            let mut sockets = self.sockets.borrow_mut();
            match iface.poll(&mut sockets, instant) {
                Ok(processed) => processed,
                Err(_) => true,
            }
        };
        if processed {
            let mut wakers = self.wakers.borrow_mut();
            for waker in wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// TODO: this was called through eg. TcpStream, another poll()
    /// might want to send packets before sleeping for an interrupt.
    pub(crate) fn register_waker(waker: Waker) {
        let mut wakers = Self::instance().wakers.borrow_mut();
        for (i, w) in wakers.iter().enumerate() {
            if w.will_wake(&waker) {
                let last = wakers.len() - 1;
                wakers.swap(i, last);
                return;
            }
        }
        wakers.push(waker);
    }
}
