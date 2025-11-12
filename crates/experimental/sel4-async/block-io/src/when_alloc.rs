//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::num::NonZeroUsize;
use core::ops::Deref;

use futures::future;
use lru::LruCache;

use crate::{Access, BlockIO, BlockIOLayout, BlockSize, Operation, wrapper_methods};

pub struct DynamicBlockSize {
    bits: usize,
}

impl DynamicBlockSize {
    pub fn new(bits: usize) -> Self {
        Self { bits }
    }
}

impl BlockSize for DynamicBlockSize {
    type Block = Vec<u8>;

    fn bytes(&self) -> usize {
        1 << self.bits
    }

    fn zeroed_block(&self) -> Self::Block {
        vec![0; self.bytes()]
    }
}

impl<T: BlockIOLayout> BlockIOLayout for Rc<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.deref().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.deref().num_blocks()
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for Rc<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        self.deref()
            .read_or_write_blocks(start_block_idx, operation)
            .await
    }
}

#[derive(Debug)]
pub struct CachedBlockIO<T: BlockIOLayout> {
    inner: T,
    lru: RefCell<LruCache<u64, <T::BlockSize as BlockSize>::Block>>,
}

impl<T: BlockIOLayout> CachedBlockIO<T> {
    pub fn new(inner: T, cache_size_in_blocks: usize) -> Self {
        Self {
            inner,
            lru: RefCell::new(LruCache::new(
                NonZeroUsize::new(cache_size_in_blocks).unwrap(),
            )),
        }
    }

    wrapper_methods!(T);
}

impl<T: BlockIOLayout> BlockIOLayout for CachedBlockIO<T> {
    type Error = <T as BlockIOLayout>::Error;

    type BlockSize = <T as BlockIOLayout>::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        <T as BlockIOLayout>::block_size(self.inner())
    }

    fn num_blocks(&self) -> u64 {
        <T as BlockIOLayout>::num_blocks(self.inner())
    }
}

impl<T: BlockIO<A>, A: Access> BlockIO<A> for CachedBlockIO<T> {
    async fn read_or_write_blocks(
        &self,
        start_block_idx: u64,
        mut operation: Operation<'_, A>,
    ) -> Result<(), Self::Error> {
        assert_eq!(operation.len() % self.block_size().bytes(), 0);
        future::try_join_all(operation.chunks(self.block_size().bytes()).enumerate().map(
            |(i, block_operation)| async move {
                let block_idx = start_block_idx.checked_add(i.try_into().unwrap()).unwrap();
                match block_operation {
                    Operation::Read { buf, witness } => {
                        // NOTE: odd control flow to avoid holding core::cell::RefMut across await
                        let cached = self
                            .lru
                            .borrow_mut()
                            .get(&block_idx)
                            .map(|block| {
                                buf.copy_from_slice(block.as_ref());
                            })
                            .is_some();
                        if !cached {
                            let mut block = self.block_size().zeroed_block();
                            self.inner
                                .read_or_write_blocks(
                                    block_idx,
                                    Operation::Read {
                                        buf: block.as_mut(),
                                        witness,
                                    },
                                )
                                .await?;
                            buf.copy_from_slice(block.as_ref());
                            let _ = self.lru.borrow_mut().put(block_idx, block);
                        }
                    }
                    Operation::Write { buf, witness } => {
                        self.inner
                            .read_or_write_blocks(block_idx, Operation::Write { buf, witness })
                            .await?;
                        let mut block = self.block_size().zeroed_block();
                        block.as_mut().copy_from_slice(buf);
                        let _ = self.lru.borrow_mut().put(block_idx, block);
                    }
                }
                Ok(())
            },
        ))
        .await?;
        Ok(())
    }
}
