#![no_std]
#![feature(async_fn_in_trait)]
#![feature(never_type)]

pub use embedded_fat::*;

mod block_io_wrapper;
mod dummy_time_source;

pub use block_io_wrapper::BlockIOWrapper;
pub use dummy_time_source::DummyTimeSource;
