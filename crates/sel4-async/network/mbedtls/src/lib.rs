#![no_std]
#![feature(c_size_t)]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]

use core::cell::RefCell;
use core::slice;
use core::task::{Context, Poll};

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use mbedtls::error::Result as TlsResult;
use mbedtls::rng::EntropyCallback;
use mbedtls::ssl::async_io::AsyncIo;

use sel4_async_network::{TcpSocket, TcpSocketError};

// re-export
pub use mbedtls;

pub struct TcpSocketWrapper {
    inner: TcpSocket,
}

impl TcpSocketWrapper {
    pub fn new(inner: TcpSocket) -> Self {
        Self { inner }
    }

    pub fn inner_mut(&mut self) -> &mut TcpSocket {
        &mut self.inner
    }

    pub fn into_inner(self) -> TcpSocket {
        self.inner
    }
}

impl AsyncIo for TcpSocketWrapper {
    type Error = TcpSocketError;

    fn poll_recv(
        &mut self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        self.inner_mut().poll_recv(cx, buf)
    }

    fn poll_send(&mut self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, Self::Error>> {
        self.inner_mut().poll_send(cx, buf)
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
