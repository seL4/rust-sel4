#![no_std]
#![feature(associated_type_bounds)]
#![feature(async_fn_in_trait)]
#![feature(const_option)]
#![feature(int_roundings)]
#![feature(never_type)]
#![feature(slice_as_chunks)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;

use futures::future;

pub mod disk;

#[cfg(feature = "alloc")]
mod when_alloc;

#[cfg(feature = "alloc")]
pub use when_alloc::CachedBlockIO;

pub trait BlockIO {
    type Error: fmt::Debug;

    type BlockSize: BlockSize;

    fn num_blocks(&self) -> u64;

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error>;
}

pub trait BlockSize {
    const BYTES: usize;

    type Block: AsRef<[u8]> + AsMut<[u8]>;

    fn zeroed_block() -> Self::Block;
}

pub trait HasNextBlockSize: BlockSize {
    type NextBlockSize: BlockSize;
}

pub trait HasPrevBlockSize: BlockSize {
    type PrevBlockSize: BlockSize;
}

pub mod block_sizes {
    use super::{BlockSize, HasNextBlockSize, HasPrevBlockSize};

    macro_rules! declare_block_size {
        ($ident:ident, $n:literal) => {
            pub struct $ident;

            impl BlockSize for $ident {
                const BYTES: usize = $n;

                type Block = [u8; $n];

                fn zeroed_block() -> Self::Block {
                    [0; $n]
                }
            }
        };
    }

    macro_rules! declare_next_block_size {
        ($cur:ident, $next:ident) => {
            impl HasNextBlockSize for $cur {
                type NextBlockSize = $next;
            }

            impl HasPrevBlockSize for $next {
                type PrevBlockSize = $cur;
            }
        };
    }

    declare_block_size!(BlockSize512, 512);
    declare_block_size!(BlockSize1024, 1024);
    declare_block_size!(BlockSize2048, 2048);
    declare_block_size!(BlockSize4096, 4096);
    declare_block_size!(BlockSize8192, 8192);

    declare_next_block_size!(BlockSize512, BlockSize1024);
    declare_next_block_size!(BlockSize1024, BlockSize2048);
    declare_next_block_size!(BlockSize2048, BlockSize4096);
    declare_next_block_size!(BlockSize4096, BlockSize8192);
}

macro_rules! wrapper_methods {
    ($inner:path) => {
        pub fn into_inner(self) -> $inner {
            self.inner
        }

        pub const fn inner(&self) -> &$inner {
            &self.inner
        }

        pub fn inner_mut(&mut self) -> &mut $inner {
            &mut self.inner
        }
    };
}

use wrapper_methods;

#[derive(Clone, Debug)]
pub struct NextBlockSizeAdapter<T> {
    inner: T,
}

impl<T> NextBlockSizeAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    wrapper_methods!(T);
}

impl<T: BlockIO<BlockSize: HasNextBlockSize>> BlockIO for NextBlockSizeAdapter<T> {
    type Error = T::Error;

    type BlockSize = <T::BlockSize as HasNextBlockSize>::NextBlockSize;

    fn num_blocks(&self) -> u64 {
        let inner_num_blocks = self.inner().num_blocks();
        assert_eq!(inner_num_blocks % 2, 0);
        inner_num_blocks / 2
    }

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        let inner_start_block_idx = start_block_idx.checked_mul(2).unwrap();
        self.inner().read_blocks(inner_start_block_idx, buf).await
    }
}

#[derive(Clone, Debug)]
pub struct PrevBlockSizeAdapter<T> {
    inner: T,
}

impl<T> PrevBlockSizeAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    wrapper_methods!(T);
}

impl<T: BlockIO<BlockSize: HasPrevBlockSize>> BlockIO for PrevBlockSizeAdapter<T> {
    type Error = T::Error;

    type BlockSize = <T::BlockSize as HasPrevBlockSize>::PrevBlockSize;

    fn num_blocks(&self) -> u64 {
        self.inner().num_blocks().checked_mul(2).unwrap()
    }

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        let block_size = Self::BlockSize::BYTES.try_into().unwrap();
        let start_byte_idx = start_block_idx.checked_mul(block_size).unwrap();
        read_bytes(self.inner(), start_byte_idx, buf).await
    }
}

#[derive(Clone, Debug)]
pub struct Partition<T> {
    inner: T,
    range: Range<u64>,
}

impl<T: BlockIO> Partition<T> {
    pub fn new(inner: T, range: Range<u64>) -> Self {
        assert!(range.start <= range.end);
        assert!(range.end <= inner.num_blocks());
        Self { inner, range }
    }

    wrapper_methods!(T);
}

impl<T: BlockIO> BlockIO for Partition<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn num_blocks(&self) -> u64 {
        self.range.end - self.range.start
    }

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        assert!(
            start_block_idx + u64::try_from(buf.len() / Self::BlockSize::BYTES).unwrap()
                <= self.num_blocks()
        );
        let inner_block_idx = self.range.start + start_block_idx;
        self.inner().read_blocks(inner_block_idx, buf).await
    }
}

pub trait ByteIO {
    type Error: fmt::Debug;

    fn size(&self) -> u64;

    async fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug)]
pub struct ByteIOAdapter<T> {
    inner: T,
}

impl<T> ByteIOAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    wrapper_methods!(T);
}

impl<T: BlockIO> ByteIO for ByteIOAdapter<T> {
    type Error = T::Error;

    fn size(&self) -> u64 {
        self.inner().num_blocks() * u64::try_from(T::BlockSize::BYTES).unwrap()
    }

    async fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        read_bytes(self.inner(), offset, buf).await
    }
}

#[derive(Clone, Debug)]
pub struct BlockIOAdapter<T, N: BlockSize> {
    inner: T,
    _phantom: PhantomData<N>,
}

impl<T, N: BlockSize> BlockIOAdapter<T, N> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    wrapper_methods!(T);
}

impl<T: ByteIO, N: BlockSize> BlockIO for BlockIOAdapter<T, N> {
    type Error = T::Error;

    type BlockSize = N;

    fn num_blocks(&self) -> u64 {
        self.inner().size() / u64::try_from(Self::BlockSize::BYTES).unwrap()
    }

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        let block_size = Self::BlockSize::BYTES.try_into().unwrap();
        let start_byte_idx = start_block_idx.checked_mul(block_size).unwrap();
        self.inner().read(start_byte_idx, buf).await
    }
}

pub struct SliceByteIO<T> {
    inner: T,
}

impl<T> SliceByteIO<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    wrapper_methods!(T);
}

impl<T: AsRef<[u8]>> ByteIO for SliceByteIO<T> {
    type Error = !;

    fn size(&self) -> u64 {
        self.inner().as_ref().len().try_into().unwrap()
    }

    async fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset.try_into().unwrap();
        buf.copy_from_slice(&self.inner().as_ref()[offset..][..buf.len()]);
        Ok(())
    }
}

async fn read_partial_block<T: BlockIO>(
    io: &T,
    block_idx: u64,
    offset_into_block: usize,
    buf: &mut [u8],
) -> Result<(), T::Error> {
    assert!(offset_into_block + buf.len() <= T::BlockSize::BYTES);
    let mut block_buf = T::BlockSize::zeroed_block();
    io.read_blocks(block_idx, block_buf.as_mut()).await?;
    buf.copy_from_slice(&block_buf.as_ref()[offset_into_block..][..buf.len()]);
    Ok(())
}

async fn read_bytes<T: BlockIO>(io: &T, offset: u64, buf: &mut [u8]) -> Result<(), T::Error> {
    let block_size = T::BlockSize::BYTES.try_into().unwrap();
    let byte_offset_of_first_full_block = offset.next_multiple_of(block_size);
    let byte_offset_of_first_full_block_in_buf =
        usize::try_from(byte_offset_of_first_full_block - offset).unwrap();
    let first_full_block_idx = byte_offset_of_first_full_block / block_size;
    let num_full_blocks =
        (buf.len() - byte_offset_of_first_full_block_in_buf) / T::BlockSize::BYTES;
    if byte_offset_of_first_full_block > offset + u64::try_from(buf.len()).unwrap() {
        let block_idx = first_full_block_idx - 1;
        let offset_into_block = offset - block_idx * block_size;
        read_partial_block(io, block_idx, offset_into_block.try_into().unwrap(), buf).await?;
    } else {
        let (left_partial_block, rest) = buf.split_at_mut(byte_offset_of_first_full_block_in_buf);
        let (full_blocks, right_partial_block) =
            rest.split_at_mut(num_full_blocks * T::BlockSize::BYTES);
        future::try_join3(
            async { io.read_blocks(first_full_block_idx, full_blocks).await },
            async {
                if !left_partial_block.is_empty() {
                    let block_idx = first_full_block_idx - 1;
                    let offset_into_block = T::BlockSize::BYTES - left_partial_block.len();
                    read_partial_block(io, block_idx, offset_into_block, left_partial_block)
                        .await?;
                }
                Ok(())
            },
            async {
                if !right_partial_block.is_empty() {
                    let block_idx = first_full_block_idx + u64::try_from(num_full_blocks).unwrap();
                    let offset_into_block = 0;
                    read_partial_block(io, block_idx, offset_into_block, right_partial_block)
                        .await?;
                }
                Ok(())
            },
        )
        .await?;
    }
    Ok(())
}
