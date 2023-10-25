//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub mod channels {
    use sel4_microkit::Channel;

    pub const DEVICE: Channel = Channel::new(0);
    pub const CLIENT: Channel = Channel::new(1);
}

pub const VIRTIO_BLK_MMIO_OFFSET: usize = 0xc00;
pub const VIRTIO_BLK_DRIVER_DMA_SIZE: usize = 0x200_000;
pub const VIRTIO_BLK_CLIENT_DMA_SIZE: usize = 0x200_000;
