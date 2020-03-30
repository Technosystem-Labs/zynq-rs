use core::{
    cell::RefCell,
    task::Waker,
};
use alloc::{vec, vec::Vec};
use smoltcp::{
    iface::EthernetInterface,
    phy::Device,
    socket::{
        SocketSet, SocketHandle,
        TcpSocketBuffer, TcpSocket,
    },
    time::Instant,
};
use libboard_zynq::println;
use crate::task;

mod tcp_stream;
pub use tcp_stream::TcpStream;

static mut SOCKETS: Option<Sockets> = None;

pub struct Sockets {
    sockets: RefCell<SocketSet<'static, 'static, 'static>>,
    wakers: RefCell<Vec<Waker>>,
}

impl Sockets {
    pub fn init(max_sockets: usize) {
        println!("initializing {} sockets", max_sockets);
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
        println!("sockets initialized");
        unsafe { SOCKETS = Some(instance); }
    }

    /// Block and run executor indefinitely while polling the smoltcp
    /// iface
    pub fn run<'b, 'c, 'e, D: for<'d> Device<'d>>(iface: &mut EthernetInterface<'b, 'c, 'e, D>) {
        task::block_on(async {
            loop {
                Self::instance().poll(iface);
                task::r#yield().await;
            }
        });
    }

    pub(crate) fn instance() -> &'static Self {
        unsafe { SOCKETS.as_ref().expect("Sockets") }
    }
    
    fn poll<'b, 'c, 'e, D: for<'d> Device<'d>>(&self, iface: &mut EthernetInterface<'b, 'c, 'e, D>) {
        // TODO:
        let instant = Instant::from_millis(0);
        let processed = {
            let mut sockets = self.sockets.borrow_mut();
            let r = iface.poll(&mut sockets, instant);
            if r != Ok(false) { println!("poll: {:?}", r); }
            match r {
                Ok(processed) => processed,
                Err(_) => true,
            }
        };
        if processed {
            let mut wakers = self.wakers.borrow_mut();
            println!("wakeup of {}", wakers.len());
            for waker in wakers.drain(..) {
                waker.wake();
            }
        }
    }

    /// TODO: this was called through eg. TcpStream, another poll()
    /// might want to send packets before sleeping for an interrupt.
    pub(crate) fn register_waker(waker: Waker) {
        println!("register_waker");
        Self::instance().wakers.borrow_mut()
            .push(waker);
    }
}
