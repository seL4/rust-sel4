#![no_std]
#![feature(c_size_t)]
#![feature(cfg_target_thread_local)]
#![feature(lazy_cell)]
#![feature(thread_local)]

use core::cell::RefCell;
use core::slice;
use core::task;
use core::task::Poll;

use futures::future;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use mbedtls::error::{codes, Result as TlsResult};
use mbedtls::rng::EntropyCallback;
use mbedtls::ssl;

use sel4_async_network::TcpSocket;

// re-export
pub use mbedtls;

pub struct ContextWrapper {
    context: ssl::Context<TcpSocketWrapper>,
}

impl ContextWrapper {
    pub fn new(context: ssl::Context<TcpSocketWrapper>) -> Self {
        Self { context }
    }

    pub fn context_mut(&mut self) -> &mut ssl::Context<TcpSocketWrapper> {
        &mut self.context
    }

    pub fn into_context(self) -> ssl::Context<TcpSocketWrapper> {
        self.context
    }

    fn result_to_poll<T>(&mut self, r: TlsResult<T>, cx: &task::Context) -> Poll<TlsResult<T>> {
        match r {
            Ok(ok) => Poll::Ready(Ok(ok)),
            Err(e) if e.high_level() == Some(codes::SslWantRead) => {
                self.context_mut()
                    .io_mut()
                    .unwrap()
                    .socket_mut()
                    .with_mut(|socket| socket.register_recv_waker(cx.waker()));
                Poll::Pending
            }
            Err(e) if e.high_level() == Some(codes::SslWantWrite) => {
                self.context_mut()
                    .io_mut()
                    .unwrap()
                    .socket_mut()
                    .with_mut(|socket| socket.register_send_waker(cx.waker()));
                Poll::Pending
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }

    pub async fn establish(
        &mut self,
        io: TcpSocketWrapper,
        hostname: Option<&str>,
    ) -> TlsResult<()> {
        self.context_mut().prepare_handshake(io, hostname)?;
        future::poll_fn(|cx| {
            let r = self.context_mut().handshake();
            self.result_to_poll(r, cx)
        })
        .await
    }

    pub async fn recv(&mut self, buf: &mut [u8]) -> TlsResult<usize> {
        future::poll_fn(|cx| {
            let r = self.context_mut().read_impl_inner(buf);
            self.result_to_poll(r, cx)
        })
        .await
    }

    pub async fn send(&mut self, buf: &[u8]) -> TlsResult<usize> {
        future::poll_fn(|cx| {
            let r = self.context_mut().async_write(buf);
            self.result_to_poll(r, cx)
        })
        .await
    }

    pub async fn send_all(&mut self, buffer: &[u8]) -> TlsResult<()> {
        let mut pos = 0;
        while pos < buffer.len() {
            let n = self.send(&buffer[pos..]).await?;
            assert!(n > 0); // check assumption about mbedtls API
            pos += n;
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> TlsResult<()> {
        future::poll_fn(|cx| {
            let r = self.context_mut().flush_output();
            self.result_to_poll(r, cx)
        })
        .await
    }

    pub async fn close(&mut self) -> TlsResult<()> {
        future::poll_fn(|cx| {
            let r = self.context_mut().close_notify();
            self.result_to_poll(r, cx)
        })
        .await?;
        self.context_mut()
            .io_mut()
            .unwrap()
            .socket_mut()
            .close()
            .await
            .map_err(|_| codes::NetSocketFailed)?;
        self.context_mut().drop_io();
        Ok(())
    }
}

pub struct TcpSocketWrapper {
    socket: TcpSocket,
}

impl TcpSocketWrapper {
    pub fn new(socket: TcpSocket) -> Self {
        Self { socket }
    }

    pub fn socket_mut(&mut self) -> &mut TcpSocket {
        &mut self.socket
    }

    pub fn into_socket(self) -> TcpSocket {
        self.socket
    }
}

impl ssl::Io for TcpSocketWrapper {
    fn recv(&mut self, buf: &mut [u8]) -> TlsResult<usize> {
        match self.socket_mut().recv_poll_fn(buf) {
            Poll::Ready(Ok(ok)) => Ok(ok),
            Poll::Ready(Err(_)) => Err(codes::NetRecvFailed.into()),
            Poll::Pending => Err(codes::SslWantRead.into()),
        }
    }

    fn send(&mut self, buf: &[u8]) -> TlsResult<usize> {
        match self.socket_mut().send_poll_fn(buf) {
            Poll::Ready(Ok(ok)) => Ok(ok),
            Poll::Ready(Err(_)) => Err(codes::NetSendFailed.into()),
            Poll::Pending => Err(codes::SslWantWrite.into()),
        }
    }
}

pub fn wrap_entropy_callback<F>(f: F) -> impl EntropyCallback
where
    F: Fn(&mut [u8]) -> TlsResult<()> + Send + Sync,
{
    move |data, len| {
        let buf = unsafe { slice::from_raw_parts_mut(data, len) };
        match f(buf) {
            Ok(_) => 0,
            Err(err) => err.to_int(),
        }
    }
}

#[cfg(not(target_thread_local))]
compile_error!("");

#[thread_local]
static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);

pub fn seed_insecure_dummy_entropy(seed: u64) {
    assert!(RNG.replace(Some(SmallRng::seed_from_u64(seed))).is_none());
}

pub fn insecure_dummy_entropy() -> impl EntropyCallback {
    wrap_entropy_callback(|buf| {
        RNG.borrow_mut().as_mut().unwrap().fill_bytes(buf);
        Ok(())
    })
}
