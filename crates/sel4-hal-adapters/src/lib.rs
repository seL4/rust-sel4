#![no_std]
#![feature(never_type)]
#![feature(strict_provenance)]
#![feature(let_chains)]

#[cfg(feature = "smoltcp-hal")]
pub mod smoltcp;
