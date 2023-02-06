#![no_std]

pub mod elf;

#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "unwinding")]
pub mod unwinding;

#[cfg(feature = "embedded-phdrs")]
pub mod embedded;

#[cfg(feature = "injected-phdrs")]
pub mod injected;
