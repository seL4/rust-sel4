use sel4_async_block_io_cpiofs::BlockIO as CpiofsBlockIO;
use sel4_shared_ring_buffer_block_io::BlockIO as SharedRingBufferBlockIO;

pub use sel4_shared_ring_buffer_block_io::BLOCK_SIZE;

#[derive(Clone)]
pub(crate) struct CpiofsBlockIOImpl(SharedRingBufferBlockIO);

impl CpiofsBlockIOImpl {
    pub(crate) fn new(inner: SharedRingBufferBlockIO) -> Self {
        Self(inner)
    }

    pub(crate) fn poll(&self) -> bool {
        self.0.poll()
    }
}

impl CpiofsBlockIO<BLOCK_SIZE> for CpiofsBlockIOImpl {
    async fn read_block(&self, block_id: usize, buf: &mut [u8; BLOCK_SIZE]) {
        self.0.read_block(block_id, buf).await
    }
}
