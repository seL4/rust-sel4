use futures::future;

use sel4_async_block_io::{constant_block_sizes, BlockIO};

pub use embedded_fat as fat;

pub struct BlockIOWrapper<T> {
    inner: T,
}

impl<T> BlockIOWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: BlockIO<BlockSize = constant_block_sizes::BlockSize512>> fat::BlockDevice
    for BlockIOWrapper<T>
{
    type Error = !;

    async fn read(
        &self,
        blocks: &mut [fat::Block],
        start_block_idx: fat::BlockIdx,
        _reason: &str,
    ) -> Result<(), Self::Error> {
        future::join_all(blocks.iter_mut().enumerate().map(|(i, block)| async move {
            let block_idx = u64::try_from(start_block_idx.0)
                .unwrap()
                .checked_add(i.try_into().unwrap())
                .unwrap();
            self.inner.read_blocks(block_idx, &mut block.contents).await
        }))
        .await;
        Ok(())
    }

    async fn write(
        &self,
        _blocks: &[fat::Block],
        _start_block_idx: fat::BlockIdx,
    ) -> Result<(), Self::Error> {
        panic!()
    }

    async fn num_blocks(&self) -> Result<fat::BlockCount, Self::Error> {
        Ok(fat::BlockCount(self.inner.num_blocks().try_into().unwrap()))
    }
}
