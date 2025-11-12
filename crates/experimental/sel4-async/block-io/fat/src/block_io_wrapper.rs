//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::convert::Infallible;
use core::marker::PhantomData;

use futures::future;

use sel4_async_block_io::{
    BlockIO, Operation,
    access::{Access, Witness},
    constant_block_sizes,
};

pub use embedded_fat as fat;

pub struct BlockIOWrapper<T, A> {
    inner: T,
    _phantom: PhantomData<A>,
}

impl<T, A> BlockIOWrapper<T, A> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<T: BlockIO<A, BlockSize = constant_block_sizes::BlockSize512>, A: Access> fat::BlockDevice
    for BlockIOWrapper<T, A>
{
    type Error = Infallible;

    async fn read(
        &self,
        blocks: &mut [fat::Block],
        start_block_idx: fat::BlockIdx,
        _reason: &str,
    ) -> Result<(), Self::Error> {
        future::join_all(blocks.iter_mut().enumerate().map(|(i, block)| async move {
            let block_idx = u64::from(start_block_idx.0)
                .checked_add(i.try_into().unwrap())
                .unwrap();
            self.inner
                .read_or_write_blocks(
                    block_idx,
                    Operation::Read {
                        buf: &mut block.contents,
                        witness: A::ReadWitness::TRY_WITNESS.unwrap(),
                    },
                )
                .await
        }))
        .await;
        Ok(())
    }

    async fn write(
        &self,
        blocks: &[fat::Block],
        start_block_idx: fat::BlockIdx,
    ) -> Result<(), Self::Error> {
        future::join_all(blocks.iter().enumerate().map(|(i, block)| async move {
            let block_idx = u64::from(start_block_idx.0)
                .checked_add(i.try_into().unwrap())
                .unwrap();
            self.inner
                .read_or_write_blocks(
                    block_idx,
                    Operation::Write {
                        buf: &block.contents,
                        witness: A::WriteWitness::TRY_WITNESS.unwrap(),
                    },
                )
                .await
        }))
        .await;
        Ok(())
    }

    async fn num_blocks(&self) -> Result<fat::BlockCount, Self::Error> {
        Ok(fat::BlockCount(self.inner.num_blocks().try_into().unwrap()))
    }
}
