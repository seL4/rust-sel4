#![no_std]
#![feature(c_size_t)]
#![feature(cfg_target_thread_local)]
#![feature(thread_local)]

extern crate alloc;

use alloc::borrow::Cow;
use alloc::format;
use core::cell::RefCell;
use core::ffi::{c_int, c_size_t as size_t, c_uchar};
use core::slice;
use core::task::{Context, Poll};

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use mbedtls::error::Result as TlsResult;
use mbedtls::ssl::async_io::AsyncIo;
use mbedtls::ssl::config::DbgCallback;

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

pub fn wrap_rng_callback<F>(f: F) -> impl Fn(*mut c_uchar, size_t) -> c_int + Send + Sync
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

pub fn seed_insecure_dummy_rng(seed: u64) {
    assert!(RNG.replace(Some(SmallRng::seed_from_u64(seed))).is_none());
}

pub fn insecure_dummy_rng() -> impl Fn(*mut c_uchar, size_t) -> c_int + Send + Sync {
    wrap_rng_callback(|buf| {
        RNG.borrow_mut().as_mut().unwrap().fill_bytes(buf);
        Ok(())
    })
}

pub struct DbgCallbackBuilder {
    forward_log_level: log::Level,
    local_debug_threshold: i32,
    include_locations: bool,
}

impl DbgCallbackBuilder {
    pub const fn const_default() -> Self {
        Self {
            forward_log_level: log::Level::Debug,
            local_debug_threshold: 5,
            include_locations: false,
        }
    }

    pub const fn forward_log_level(mut self, level: log::Level) -> Self {
        self.forward_log_level = level;
        self
    }

    pub const fn local_debug_threshold(mut self, threshold: i32) -> Self {
        self.local_debug_threshold = threshold;
        self
    }

    pub const fn include_locations(mut self, doit: bool) -> Self {
        self.include_locations = doit;
        self
    }

    pub const fn build(self) -> impl DbgCallback + Clone {
        move |level: i32, file: Cow<'_, str>, line: i32, message: Cow<'_, str>| {
            if level <= self.local_debug_threshold {
                let message = message.strip_suffix('\n').unwrap();
                let target = "mbedtls";
                // TODO remove allocations
                let location = if self.include_locations {
                    format!(" {file}:{line}")
                } else {
                    format!("")
                };
                log::log!(target: target, self.forward_log_level, "({level}) {location}{message}");
            }
        }
    }
}

impl Default for DbgCallbackBuilder {
    fn default() -> Self {
        Self::const_default()
    }
}
