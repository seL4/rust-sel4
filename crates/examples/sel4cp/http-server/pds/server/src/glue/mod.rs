mod cpiofs_io_impl;
mod virtio_blk_hal_impl;

pub(crate) use cpiofs_io_impl::{CpiofsBlockIOImpl, BLOCK_SIZE};
pub(crate) use virtio_blk_hal_impl::VirtioBlkHalImpl;
