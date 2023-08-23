#![no_std]
#![feature(async_fn_in_trait)]
#![feature(int_roundings)]
#![feature(slice_as_chunks)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod when_alloc;

#[cfg(feature = "alloc")]
pub use when_alloc::{BytesIOAdapter, CachedBlockIO};

// NOTE: type gymnastics due to current limitations of generic_const_exprs

pub type BlockId = usize;

pub trait BlockIO<const BLOCK_SIZE: usize> {
    async fn read_block(&self, block_id: usize, buf: &mut [u8; BLOCK_SIZE]);
}

pub trait BytesIO {
    async fn read(&self, offset: usize, buf: &mut [u8]);
}
