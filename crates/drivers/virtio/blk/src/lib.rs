//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::convert::Infallible;
use core::ops::Deref;

use sel4_driver_interfaces::block::GetBlockDeviceLayout;
use virtio_drivers::device::blk::{SECTOR_SIZE, VirtIOBlk};
use virtio_drivers::{Hal, transport::Transport};

pub struct GetBlockDeviceLayoutWrapper<T>(pub T);

impl<H: Hal, T: Transport, U: Deref<Target = VirtIOBlk<H, T>>> GetBlockDeviceLayout
    for GetBlockDeviceLayoutWrapper<U>
{
    type Error = Infallible;

    fn get_block_size(&mut self) -> Result<usize, Self::Error> {
        Ok(SECTOR_SIZE)
    }

    fn get_num_blocks(&mut self) -> Result<u64, Self::Error> {
        Ok(self.0.deref().capacity())
    }
}
