//! async TCP interface
//!
//! TODO: implement futures AsyncRead/AsyncWrite/Stream/Sink interfaces

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use alloc::vec::Vec;
use smoltcp::{
    socket::{
        SocketHandle, SocketRef,
        TcpSocketBuffer, TcpSocket,
    },
};
use crate::task;
use super::Sockets;

/// References a smoltcp TcpSocket
pub struct TcpStream {
    handle: SocketHandle,
}

/// Wait while letting `$f()` poll a stream's socket
macro_rules! poll_stream {
    ($stream: expr, $output: ty, $f: expr) => (async {
        struct Adhoc<'a> {
            stream: &'a TcpStream,
        }

        impl<'a> Future for Adhoc<'a> {
            type Output = $output;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let result = self.stream.with_socket($f);
                if !result.is_ready() {
                    Sockets::register_waker(cx.waker().clone());
                }
                result
            }
        }

        Adhoc { stream: $stream }.await
    })
}

impl TcpStream {
    /// Allocates sockets and its buffers, registers it in the
    /// SocketSet.
    ///
    /// Not `pub` as the result can not yet be used. Use `listen()` or
    /// `connect()` to obtain a valid TcpStream.
    fn new(rx_bufsize: usize, tx_bufsize: usize) -> Self {
        fn uninit_vec<T>(size: usize) -> Vec<T> {
            let mut result = Vec::with_capacity(size);
            unsafe {
                result.set_len(size);
            }
            result
        }
        let rx_buffer = TcpSocketBuffer::new(uninit_vec(rx_bufsize));
        let tx_buffer = TcpSocketBuffer::new(uninit_vec(tx_bufsize));
        let socket = TcpSocket::new(rx_buffer, tx_buffer);
        let handle = Sockets::instance().sockets.borrow_mut()
            .add(socket);
        TcpStream { handle }
    }

    /// Operate on the referenced TCP socket
    fn with_socket<F, R>(&self, f: F) -> R
    where
        F: FnOnce(SocketRef<TcpSocket>) -> R,
    {
        let mut sockets = Sockets::instance().sockets.borrow_mut();
        let socket_ref = sockets.get::<TcpSocket>(self.handle);
        f(socket_ref)
    }

    /// Spawns `backlog` tasks with listening sockets so that more
    /// connections can be accepted while some are still
    /// handshaking. Spawns additional tasks for each connection.
    pub fn listen<F, R, T>(port: u16, rx_bufsize: usize, tx_bufsize: usize, backlog: usize, f: F)
    where
        F: Fn(Self) -> R + Clone + 'static,
        R: Future<Output = T> + 'static,
    {
        for _ in 0..backlog {
            let f = f.clone();
            task::spawn(async move {
                loop {
                    // Wait for new connection
                    let stream = TcpStream::accept(port, rx_bufsize, tx_bufsize).await;
                    // Spawn async task for new connection
                    task::spawn(f(stream));
                }
            });
        }
    }

    /// Listen for the next incoming connection on a TCP
    /// port. Succeeds on connection attempt.
    ///
    /// Calling this serially in a loop will cause slow/botched
    /// connection attempts stall any more new connections. Use
    /// `listen()` with a backlog instead.
    pub async fn accept(port: u16, rx_bufsize: usize, tx_bufsize: usize) -> Self {
        let stream = Self::new(rx_bufsize, tx_bufsize);
        // Set socket to listen
        stream.with_socket(|mut s| s.listen(port))
            .expect("listen");
        // Wait for a connection
        poll_stream!(&stream, (), |socket| {
            if socket.is_active() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }).await;
        stream
    }

    /// Probe the receive buffer
    ///
    /// Instead of handing you the data on the heap all at once,
    /// smoltcp's read interface is wrapped so that your callback can
    /// just return `Poll::Pending` if there is not enough data
    /// yet. Likewise, return the amount of bytes consumed from the
    /// buffer in the `Poll::Ready` result.
    pub async fn recv<F, R>(&self, f: F) -> smoltcp::Result<R>
    where
        F: Fn(&[u8]) -> Poll<(usize, R)>,
    {
        struct Recv<'a, F: FnOnce(&[u8]) -> Poll<(usize, R)>, R> {
            stream: &'a TcpStream,
            f: F,
        }

        impl<'a, F: Fn(&[u8]) -> Poll<(usize, R)>, R> Future for Recv<'a, F, R> {
            type Output = smoltcp::Result<R>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let result = self.stream.with_socket(|mut socket| {
                    socket.recv(|buf| match (self.f)(buf) {
                        Poll::Ready((amount, result)) =>
                            (amount, Poll::Ready(Ok(result))),
                        Poll::Pending =>
                            // 0 bytes consumed
                            (0, Poll::Pending),
                    })
                });
                match result {
                    Ok(result) => {
                        if !result.is_ready() {
                            Sockets::register_waker(cx.waker().clone());
                        }
                        result
                    }
                    Err(e) =>
                        Poll::Ready(Err(e)),
                }
            }
        }

        Recv {
            stream: self,
            f,
        }.await
    }

    /// Wait until there is any space in the socket's send queue
    async fn wait_can_send(&self) -> smoltcp::Result<()> {
        poll_stream!(self, smoltcp::Result<()>, |socket| {
            if !socket.is_active() {
                Poll::Ready(Err(smoltcp::Error::Illegal))
            } else if socket.can_send() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }).await
    }

    /// Yields to wait for more buffer space
    pub async fn send<I: IntoIterator<Item = u8>>(&self, data: I) -> Result<(), smoltcp::Error> {
        let mut data = data.into_iter();
        let mut done = false;
        while !done {
            self.wait_can_send().await?;

            self.with_socket(|mut socket| {
                socket.send(|buf| {
                    for i in 0..buf.len() {
                        if let Some(byte) = data.next() {
                            buf[i] = byte;
                        } else {
                            done = true;
                            return (i, ())
                        }
                    }
                    (buf.len(), ())
                })
            })?;
        }

        Ok(())
    }

    /// Wait for all queued data to be sent and ACKed
    ///
    /// **Warning:** this may not work as immediately as expected! The
    /// other side may wait until it sends packets to you for
    /// piggybacking the ACKs.
    pub async fn flush(&self) {
        poll_stream!(self, (), |socket| {
            if socket.may_send() && socket.send_queue() > 0 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }).await
    }
}

impl Drop for TcpStream {
    /// Free item in the socket set, which leads to deallocation of
    /// the rx/tx buffers associated with this socket.
    fn drop(&mut self) {
        Sockets::instance().sockets.borrow_mut()
            .remove(self.handle);
    }
}
