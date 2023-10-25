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

pub const VIRTIO_NET_MMIO_OFFSET: usize = 0xe00;
pub const VIRTIO_NET_DRIVER_DMA_SIZE: usize = 0x200_000;
pub const VIRTIO_NET_CLIENT_DMA_SIZE: usize = 0x200_000;
