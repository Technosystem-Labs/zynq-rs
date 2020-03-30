use core::{
    cell::{RefCell, UnsafeCell},
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};
use alloc::vec;
use smoltcp::{
    iface::EthernetInterface,
    phy::Device,
    socket::{
        SocketSet, SocketHandle, SocketRef,
        TcpSocketBuffer, TcpSocket,
    },
    time::Instant,
};
use libboard_zynq::println;
use super::Sockets;

pub struct TcpStream {
    handle: SocketHandle,
}

impl TcpStream {
    fn new(rx_bufsize: usize, tx_bufsize: usize) -> Self {
        let rx_buffer = TcpSocketBuffer::new(vec![0u8; rx_bufsize]);
        let tx_buffer = TcpSocketBuffer::new(vec![0u8; tx_bufsize]);
        let socket = TcpSocket::new(rx_buffer, tx_buffer);
        let handle = Sockets::instance().sockets.borrow_mut()
            .add(socket);
        TcpStream { handle }
    }

    fn with_socket<F, R>(&self, f: F) -> R
    where
        F: FnOnce(SocketRef<TcpSocket>) -> R,
    {
        println!("with_socket");
        let mut sockets = Sockets::instance().sockets.borrow_mut();
        let socket_ref = sockets.get::<TcpSocket>(self.handle);
        f(socket_ref)
    }

    pub async fn listen(port: u16, rx_bufsize: usize, tx_bufsize: usize) -> Self {
        struct Accept {
            stream: Option<TcpStream>,
        }

        impl Future for Accept {
            type Output = TcpStream;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let is_active = self.stream.as_ref()
                    .map(|s| s.with_socket(|s| s.is_active()))
                    .unwrap_or(true);
                println!("is_active={:?}", is_active);
                if is_active {
                    Poll::Ready(self.stream.take().unwrap())
                } else {
                    println!("register_waker");
                    Sockets::register_waker(cx.waker().clone());

                    //asm::sev();
                    Poll::Pending
                }
            }
        }

        let stream = Self::new(rx_bufsize, tx_bufsize);
        stream.with_socket(|mut s| s.listen(port)).expect("listen");
        println!("listening on {}", port);
        Accept {
            stream: Some(stream),
        }.await
    }

    pub async fn recv(&self) {
        // self.socket();
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        // TODO: verify
        println!("tcpstream drop");
        Sockets::instance().sockets.borrow_mut()
            .remove(self.handle);
    }
}
