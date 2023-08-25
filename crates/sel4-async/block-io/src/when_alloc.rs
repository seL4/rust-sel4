use core::cell::RefCell;
use core::marker::PhantomData;
use core::num::NonZeroUsize;

use futures::future;
use lru::LruCache;

use crate::{BlockIO, BlockId, BytesIO};

#[derive(Clone, Debug)]
pub struct BytesIOAdapter<T, const BLOCK_SIZE: usize> {
    inner: T,
    _phantom: PhantomData<[(); BLOCK_SIZE]>,
}

impl<T, const BLOCK_SIZE: usize> BytesIOAdapter<T, BLOCK_SIZE> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<const BLOCK_SIZE: usize, T: BlockIO<BLOCK_SIZE>> BytesIOAdapter<T, BLOCK_SIZE> {
    async fn read_partial_block(&self, block_id: usize, offset_into_block: usize, buf: &mut [u8]) {
        assert!(offset_into_block + buf.len() <= BLOCK_SIZE);
        let mut block_buf = [0; BLOCK_SIZE];
        self.inner().read_block(block_id, &mut block_buf).await;
        buf.copy_from_slice(&block_buf[offset_into_block..][..buf.len()]);
    }
}

impl<const BLOCK_SIZE: usize, T: BlockIO<BLOCK_SIZE>> BytesIO for BytesIOAdapter<T, BLOCK_SIZE> {
    async fn read(&self, offset: usize, buf: &mut [u8]) {
        let offset_of_first_full_chunk = offset.next_multiple_of(BLOCK_SIZE);
        let block_id_of_first_full_chunk = offset_of_first_full_chunk / BLOCK_SIZE;
        if offset_of_first_full_chunk > offset + buf.len() {
            let block_id = block_id_of_first_full_chunk - 1;
            let offset_into_block = offset - block_id * BLOCK_SIZE;
            self.read_partial_block(block_id, offset_into_block, buf)
                .await;
        } else {
            let (left_partial_chunk, rest) = buf.split_at_mut(offset_of_first_full_chunk - offset);
            let (mid_chunks, right_partial_chunk) = rest.as_chunks_mut::<BLOCK_SIZE>();
            let num_mid_chunks = mid_chunks.len();
            future::join3(
                future::join_all(mid_chunks.iter_mut().enumerate().map(|(i, chunk)| {
                    let block_id = block_id_of_first_full_chunk + i;
                    self.inner().read_block(block_id, chunk)
                })),
                async {
                    if !left_partial_chunk.is_empty() {
                        let block_id = block_id_of_first_full_chunk - 1;
                        let offset_into_block = BLOCK_SIZE - left_partial_chunk.len();
                        self.read_partial_block(block_id, offset_into_block, left_partial_chunk)
                            .await;
                    }
                },
                async {
                    if !right_partial_chunk.is_empty() {
                        let block_id = block_id_of_first_full_chunk + num_mid_chunks;
                        let offset_into_block = 0;
                        self.read_partial_block(block_id, offset_into_block, right_partial_chunk)
                            .await;
                    }
                },
            )
            .await;
        }
    }
}

#[derive(Debug)]
pub struct CachedBlockIO<T, const BLOCK_SIZE: usize> {
    inner: T,
    lru: RefCell<LruCache<BlockId, [u8; BLOCK_SIZE]>>,
}

impl<T, const BLOCK_SIZE: usize> CachedBlockIO<T, BLOCK_SIZE> {
    pub fn new(inner: T, cache_size_in_blocks: usize) -> Self {
        Self {
            inner,
            lru: RefCell::new(LruCache::new(
                NonZeroUsize::new(cache_size_in_blocks).unwrap(),
            )),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: BlockIO<BLOCK_SIZE>, const BLOCK_SIZE: usize> BlockIO<BLOCK_SIZE>
    for CachedBlockIO<T, BLOCK_SIZE>
{
    async fn read_block(&self, block_id: usize, buf: &mut [u8; BLOCK_SIZE]) {
        // NOTE: odd control flow to avoid holding core::cell::RefMut across await
        if let Some(block) = self.lru.borrow_mut().get(&block_id) {
            *buf = *block;
            return;
        }
        self.inner().read_block(block_id, buf).await;
        let _ = self.lru.borrow_mut().put(block_id, *buf);
    }
}
