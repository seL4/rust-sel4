//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(never_type)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::cell::RefCell;
use core::fmt;
use core::ops::Range;

use futures::future;

pub mod access;
pub mod disk;

mod operation;

pub use operation::{Operation, OperationType};

use access::{Access, ReadAccess, ReadOnly, ReadWrite, WriteAccess};

#[cfg(feature = "alloc")]
mod when_alloc;

#[cfg(feature = "alloc")]
pub use when_alloc::{CachedBlockIO, DynamicBlockSize};

pub trait BlockIOLayout {
    type Error: fmt::Debug;

    type BlockSize: BlockSize;

    fn block_size(&self) -> Self::BlockSize;

    fn num_blocks(&self) -> u64;
}

pub trait BlockIO<A: Access>: BlockIOLayout {
    #[allow(async_fn_in_trait)]
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error>;

    #[allow(async_fn_in_trait)]
    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error>
    where
        A: ReadAccess,
    {
        self.read_or_write_blocks(
            start_block_idx,
            Operation::Read {
                buf,
                witness: A::READ_WITNESS,
            },
        )
        .await
    }

    #[allow(async_fn_in_trait)]
    async fn write_blocks(&self, start_block_idx: u64, buf: &[u8]) -> Result<(), Self::Error>
    where
        A: WriteAccess,
    {
        self.read_or_write_blocks(
            start_block_idx,
            Operation::Write {
                buf,
                witness: A::WRITE_WITNESS,
            },
        )
        .await
    }
}

pub trait BlockSize {
    type Block: AsRef<[u8]> + AsMut<[u8]>;

    fn bytes(&self) -> usize;

    fn bytes_u64(&self) -> u64 {
        self.bytes().try_into().unwrap()
    }

    fn zeroed_block(&self) -> Self::Block;
}

pub trait ConstantBlockSize: BlockSize {
    const BLOCK_SIZE: Self;

    const BYTES: usize;
}

pub trait HasNextBlockSize: ConstantBlockSize {
    type NextBlockSize: ConstantBlockSize;
}

pub trait HasPrevBlockSize: ConstantBlockSize {
    type PrevBlockSize: ConstantBlockSize;
}

pub mod constant_block_sizes {
    use super::{BlockSize, ConstantBlockSize, HasNextBlockSize, HasPrevBlockSize};

    macro_rules! declare_block_size {
        ($ident:ident, $n:literal) => {
            #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
            pub struct $ident;

            impl BlockSize for $ident {
                type Block = [u8; $n];

                fn bytes(&self) -> usize {
                    Self::BYTES
                }

                fn zeroed_block(&self) -> Self::Block {
                    [0; $n]
                }
            }

            impl ConstantBlockSize for $ident {
                const BLOCK_SIZE: Self = $ident;

                const BYTES: usize = $n;
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

impl<T: BlockIOLayout> BlockIOLayout for &T {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        T::block_size(self)
    }

    fn num_blocks(&self) -> u64 {
        T::num_blocks(self)
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for &T {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), <&T as BlockIOLayout>::Error> {
        T::read_or_write_blocks(self, start_block_idx, operation).await
    }
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

impl<T: BlockIOLayout> BlockIOLayout for NextBlockSizeAdapter<T>
where
    T::BlockSize: HasNextBlockSize,
{
    type Error = T::Error;

    type BlockSize = <T::BlockSize as HasNextBlockSize>::NextBlockSize;

    fn block_size(&self) -> Self::BlockSize {
        Self::BlockSize::BLOCK_SIZE
    }

    fn num_blocks(&self) -> u64 {
        let inner_num_blocks = self.inner().num_blocks();
        assert_eq!(inner_num_blocks % 2, 0);
        inner_num_blocks / 2
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for NextBlockSizeAdapter<T>
where
    T::BlockSize: HasNextBlockSize,
{
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        let inner_start_block_idx = start_block_idx.checked_mul(2).unwrap();
        self.inner()
            .read_or_write_blocks(inner_start_block_idx, operation)
            .await
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

impl<T: BlockIOLayout> BlockIOLayout for PrevBlockSizeAdapter<T>
where
    T::BlockSize: HasPrevBlockSize,
{
    type Error = T::Error;

    type BlockSize = <T::BlockSize as HasPrevBlockSize>::PrevBlockSize;

    fn block_size(&self) -> Self::BlockSize {
        Self::BlockSize::BLOCK_SIZE
    }

    fn num_blocks(&self) -> u64 {
        self.inner().num_blocks().checked_mul(2).unwrap()
    }
}

impl<T: BlockIO<A>, A: ReadAccess> BlockIO<A> for PrevBlockSizeAdapter<T>
where
    T::BlockSize: HasPrevBlockSize,
{
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        let block_size = Self::BlockSize::BYTES.try_into().unwrap();
        let start_byte_idx = start_block_idx.checked_mul(block_size).unwrap();
        read_or_write_bytes(self.inner(), start_byte_idx, operation).await
    }
}

#[derive(Clone, Debug)]
pub struct Partition<T> {
    inner: T,
    range: Range<u64>,
}

impl<T: BlockIO<ReadOnly>> Partition<T> {
    pub fn new(inner: T, range: Range<u64>) -> Self {
        assert!(range.start <= range.end);
        assert!(range.end <= inner.num_blocks());
        Self { inner, range }
    }
}

impl<T> Partition<T> {
    wrapper_methods!(T);
}

impl<T: BlockIOLayout> BlockIOLayout for Partition<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.inner().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.range.end - self.range.start
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for Partition<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        assert!(
            start_block_idx
                + u64::try_from(operation.len()).unwrap() / self.block_size().bytes_u64()
                <= self.num_blocks()
        );
        let inner_block_idx = self.range.start + start_block_idx;
        self.inner()
            .read_or_write_blocks(inner_block_idx, operation)
            .await
    }
}

pub trait ByteIOLayout {
    type Error: fmt::Debug;

    fn size(&self) -> u64;
}

pub trait ByteIO<A: Access>: ByteIOLayout {
    #[allow(async_fn_in_trait)]
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error>;

    #[allow(async_fn_in_trait)]
    async fn read(&self, offset: u64, buf: &mut [u8]) -> Result<(), Self::Error>
    where
        A: ReadAccess,
    {
        self.read_or_write(offset, Operation::read(buf)).await
    }

    #[allow(async_fn_in_trait)]
    async fn write(&self, offset: u64, buf: &[u8]) -> Result<(), Self::Error>
    where
        A: WriteAccess,
    {
        self.read_or_write(offset, Operation::write(buf)).await
    }
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

impl<T: BlockIOLayout> ByteIOLayout for ByteIOAdapter<T> {
    type Error = T::Error;

    fn size(&self) -> u64 {
        self.inner().num_blocks() * self.inner().block_size().bytes_u64()
    }
}

impl<T: BlockIO<A>, A: ReadAccess> ByteIO<A> for ByteIOAdapter<T> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        read_or_write_bytes(self.inner(), offset, operation).await
    }
}

#[derive(Clone, Debug)]
pub struct BlockIOAdapter<T, N> {
    inner: T,
    block_size: N,
}

impl<T, N> BlockIOAdapter<T, N> {
    pub fn new(inner: T, block_size: N) -> Self {
        Self { inner, block_size }
    }

    wrapper_methods!(T);
}

impl<T: ByteIOLayout, N: BlockSize + Copy> BlockIOLayout for BlockIOAdapter<T, N> {
    type Error = T::Error;

    type BlockSize = N;

    fn block_size(&self) -> Self::BlockSize {
        self.block_size
    }

    fn num_blocks(&self) -> u64 {
        self.inner().size() / self.block_size().bytes_u64()
    }
}

impl<T: ByteIO<A>, A: Access, N: BlockSize + Copy> BlockIO<A> for BlockIOAdapter<T, N> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        let start_byte_idx = start_block_idx
            .checked_mul(self.block_size().bytes_u64())
            .unwrap();
        self.inner().read_or_write(start_byte_idx, operation).await
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

impl<T: AsRef<[u8]>> ByteIOLayout for SliceByteIO<T> {
    type Error = !;

    fn size(&self) -> u64 {
        self.inner().as_ref().len().try_into().unwrap()
    }
}

impl<T: AsRef<[u8]>> ByteIO<ReadOnly> for SliceByteIO<T> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, ReadOnly>,
    ) -> Result<(), Self::Error> {
        let offset = offset.try_into().unwrap();
        match operation {
            Operation::Read { buf, .. } => {
                buf.copy_from_slice(&self.inner().as_ref()[offset..][..buf.len()]);
            }
            Operation::Write { witness, .. } => witness,
        }
        Ok(())
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ByteIOLayout for RefCell<SliceByteIO<T>> {
    type Error = !;

    fn size(&self) -> u64 {
        self.borrow().size()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ByteIO<ReadWrite> for RefCell<SliceByteIO<T>> {
    async fn read_or_write(
        &self,
        offset: u64,
        operation: Operation<'_, ReadWrite>,
    ) -> Result<(), Self::Error> {
        let offset = offset.try_into().unwrap();
        match operation {
            Operation::Read { buf, .. } => {
                buf.copy_from_slice(&self.borrow().inner().as_ref()[offset..][..buf.len()]);
            }
            Operation::Write { buf, .. } => {
                self.borrow_mut().inner_mut().as_mut()[offset..][..buf.len()].copy_from_slice(buf);
            }
        }
        Ok(())
    }
}

async fn read_or_write_partial_block<T: BlockIO<A>, A: ReadAccess>(
    io: &T,
    block_idx: u64,
    offset_into_block: usize,
    operation: Operation<'_, A>,
) -> Result<(), T::Error> {
    assert!(offset_into_block + operation.len() <= io.block_size().bytes());
    let mut block_buf = io.block_size().zeroed_block();
    io.read_blocks(block_idx, block_buf.as_mut()).await?;
    match operation {
        Operation::Read { buf, .. } => {
            buf.copy_from_slice(&block_buf.as_ref()[offset_into_block..][..buf.len()]);
        }
        Operation::Write { buf, witness } => {
            block_buf.as_mut()[offset_into_block..][..buf.len()].copy_from_slice(buf);
            io.read_or_write_blocks(
                block_idx,
                Operation::Write {
                    buf: block_buf.as_ref(),
                    witness,
                },
            )
            .await?;
        }
    }
    Ok(())
}

async fn read_or_write_bytes<T: BlockIO<A>, A: ReadAccess>(
    io: &T,
    offset: u64,
    mut operation: Operation<'_, A>,
) -> Result<(), T::Error> {
    let block_size = io.block_size().bytes();
    let block_size_u64 = io.block_size().bytes_u64();
    let byte_offset_of_first_full_block = offset.next_multiple_of(block_size_u64);
    let byte_offset_of_first_full_block_in_buf =
        usize::try_from(byte_offset_of_first_full_block - offset).unwrap();
    let first_full_block_idx = byte_offset_of_first_full_block / block_size_u64;
    let num_full_blocks = (operation.len() - byte_offset_of_first_full_block_in_buf) / block_size;
    if byte_offset_of_first_full_block > offset + u64::try_from(operation.len()).unwrap() {
        let block_idx = first_full_block_idx - 1;
        let offset_into_block = offset - block_idx * block_size_u64;
        read_or_write_partial_block(
            io,
            block_idx,
            offset_into_block.try_into().unwrap(),
            operation,
        )
        .await?;
    } else {
        let (left_partial_block, mut rest) =
            operation.split_at(byte_offset_of_first_full_block_in_buf);
        let (full_blocks, right_partial_block) = rest.split_at(num_full_blocks * block_size);
        future::try_join3(
            async {
                io.read_or_write_blocks(first_full_block_idx, full_blocks)
                    .await
            },
            async {
                if !left_partial_block.is_empty() {
                    let block_idx = first_full_block_idx - 1;
                    let offset_into_block = block_size - left_partial_block.len();
                    read_or_write_partial_block(
                        io,
                        block_idx,
                        offset_into_block,
                        left_partial_block,
                    )
                    .await?;
                }
                Ok(())
            },
            async {
                if !right_partial_block.is_empty() {
                    let block_idx = first_full_block_idx + u64::try_from(num_full_blocks).unwrap();
                    let offset_into_block = 0;
                    read_or_write_partial_block(
                        io,
                        block_idx,
                        offset_into_block,
                        right_partial_block,
                    )
                    .await?;
                }
                Ok(())
            },
        )
        .await?;
    }
    Ok(())
}

pub async fn read_bytes<T: BlockIO<A>, A: ReadAccess>(
    io: &T,
    offset: u64,
    buf: &mut [u8],
) -> Result<(), T::Error> {
    read_or_write_bytes(io, offset, Operation::read(buf)).await
}

pub async fn write_bytes<T: BlockIO<A>, A: ReadAccess + WriteAccess>(
    io: &T,
    offset: u64,
    buf: &[u8],
) -> Result<(), T::Error> {
    read_or_write_bytes(io, offset, Operation::write(buf)).await
}
