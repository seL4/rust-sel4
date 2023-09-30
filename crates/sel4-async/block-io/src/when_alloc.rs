#![allow(dead_code)]
#![allow(unused_variables)]

use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::num::NonZeroUsize;
use core::ops::Deref;

use futures::future;
use lru::LruCache;

use crate::{wrapper_methods, BlockIO, BlockSize};

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

impl<T: BlockIO> BlockIO for Rc<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.deref().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.deref().num_blocks()
    }

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.deref().read_blocks(start_block_idx, buf).await
    }
}

#[derive(Debug)]
pub struct CachedBlockIO<T: BlockIO> {
    inner: T,
    lru: RefCell<LruCache<u64, <T::BlockSize as BlockSize>::Block>>,
}

impl<T: BlockIO> CachedBlockIO<T> {
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

impl<T: BlockIO> BlockIO for CachedBlockIO<T> {
    type Error = T::Error;

    type BlockSize = T::BlockSize;

    fn block_size(&self) -> Self::BlockSize {
        self.inner().block_size()
    }

    fn num_blocks(&self) -> u64 {
        self.inner().num_blocks()
    }

    async fn read_blocks(&self, start_block_idx: u64, buf: &mut [u8]) -> Result<(), Self::Error> {
        assert_eq!(buf.len() % self.block_size().bytes(), 0);
        future::try_join_all(buf.chunks_mut(self.block_size().bytes()).enumerate().map(
            |(i, block_buf)| async move {
                let block_idx = start_block_idx.checked_add(i.try_into().unwrap()).unwrap();
                // NOTE: odd control flow to avoid holding core::cell::RefMut across await
                if let Some(block) = self.lru.borrow_mut().get(&block_idx) {
                    block_buf.copy_from_slice(block.as_ref());
                    return Ok(());
                }
                let mut block = self.block_size().zeroed_block();
                self.inner.read_blocks(block_idx, block.as_mut()).await?;
                block_buf.copy_from_slice(block.as_ref());
                let _ = self.lru.borrow_mut().put(block_idx, block);
                Ok(())
            },
        ))
        .await?;
        Ok(())
    }
}
