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
    Error, Result,
    socket::{
        SocketHandle, SocketRef,
        TcpSocketBuffer, TcpSocket, TcpState,
    },
    time::Duration,
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

    /// Listen for the next incoming connection on a TCP
    /// port. Succeeds on connection attempt.
    ///
    /// Calling this serially in a loop will cause slow/botched
    /// connection attempts stall any more new connections. Use
    /// `listen()` with a backlog instead.
    pub async fn accept(port: u16, rx_bufsize: usize, tx_bufsize: usize) -> Result<Self> {
        let stream = Self::new(rx_bufsize, tx_bufsize);
        // Set socket to listen
        stream.with_socket(|mut s| s.listen(port))?;
        // Wait for a connection
        poll_stream!(&stream, (), |socket| {
            if socket.state() != TcpState::Listen {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }).await;

        Ok(stream)
    }

    /// Probe the receive buffer
    ///
    /// Your callback will only be called when there is some data available,
    /// and it must consume at least one byte. It returns a tuple with the
    /// number of bytes it consumed, and a user-defined return value of type R.
    pub async fn recv<F, R>(&self, f: F) -> Result<R>
    where
        F: Fn(&[u8]) -> (usize, R),
    {
        struct Recv<'a, F: FnOnce(&[u8]) -> (usize, R), R> {
            stream: &'a TcpStream,
            f: F,
        }

        impl<'a, F: Fn(&[u8]) -> (usize, R), R> Future for Recv<'a, F, R> {
            type Output = Result<R>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let result = self.stream.with_socket(|mut socket| {
                    if socket_is_handhshaking(&socket) {
                        return Ok(Poll::Pending);
                    }

                    socket.recv(|buf| {
                        if buf.len() > 0 {
                            let (amount, result) = (self.f)(buf);
                            assert!(amount > 0);
                            (amount, Poll::Ready(Ok(result)))
                        } else {
                            (0, Poll::Pending)
                        }
                    })
                });
                match result {
                    Ok(Poll::Pending) => {
                        Sockets::register_waker(cx.waker().clone());
                        Poll::Pending
                    }
                    Ok(result) => {
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
    async fn wait_can_send(&self) -> Result<()> {
        poll_stream!(self, Result<()>, |socket| {
            if socket_is_handhshaking(&socket) {
                Poll::Pending
            } else if socket.can_send() {
                Poll::Ready(Ok(()))
            } else if ! socket.may_send() {
                Poll::Ready(Err(Error::Truncated))
            } else {
                Poll::Pending
            }
        }).await
    }

    /// Yields to wait for more buffer space
    pub async fn send<I: IntoIterator<Item = u8>>(&self, data: I) -> Result<()> {
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

    /// Yields to wait for more buffer space
    pub async fn send_slice(&self, mut data: &'_ [u8]) -> Result<()> {
        while data.len() > 0 {
            self.wait_can_send().await?;

            data = self.with_socket(|mut socket| {
                socket.send(|buf| {
                    let len = buf.len().min(data.len());
                    buf[..len].copy_from_slice(&data[..len]);
                    data = &data[len..];
                    (len, data)
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
    pub async fn flush(&self) -> Result<()> {
        poll_stream!(self, Result<()>, |socket| {
            if socket_is_handhshaking(&socket) {
                Poll::Pending
            } else if socket.may_send() && socket.send_queue() > 0 {
                Poll::Pending
            } else if socket.may_send() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Ready(Err(Error::Truncated))
            }
        }).await
    }

    /// Close the transmit half of the connection
    pub async fn close(&self) {
        self.with_socket(|mut socket| socket.close());

        // Yield for one iface.poll() to send the packet
        task::r#yield().await;
    }

    /// Destroy the socket, sending the RST
    pub async fn abort(self) {
        self.with_socket(|mut socket| socket.abort());

        // Yield for one iface.poll() to send the packet
        task::r#yield().await;
    }

    pub fn keep_alive(&self) -> Option<Duration> {
        self.with_socket(|socket| socket.keep_alive())
    }

    pub fn set_keep_alive(&mut self, interval: Option<Duration>) {
        self.with_socket(|mut socket| socket.set_keep_alive(interval));
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.with_socket(|socket| socket.timeout())
    }

    pub fn set_timeout(&mut self, duration: Option<Duration>) {
        self.with_socket(|mut socket| socket.set_timeout(duration));
    }

    pub fn ack_delay(&self) -> Option<Duration> {
        self.with_socket(|socket| socket.ack_delay())
    }

    pub fn set_ack_delay(&mut self, duration: Option<Duration>) {
        self.with_socket(|mut socket| socket.set_ack_delay(duration));
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

fn socket_is_handhshaking(socket: &SocketRef<TcpSocket>) -> bool {
    match socket.state() {
        TcpState::SynSent | TcpState::SynReceived =>
            true,
        _ =>
            false,
    }
}
