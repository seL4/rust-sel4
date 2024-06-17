//
// Copyright 2023, Colias Group, LLC
// Copyright (c) 2019-2020 rCore Developers
//
// SPDX-License-Identifier: MIT
//

#![no_std]

use core::ops::Deref;

use sel4_driver_interfaces::block::GetBlockLayout;
use virtio_drivers::device::blk::{VirtIOBlk, SECTOR_SIZE};
use virtio_drivers::{transport::Transport, Hal};

pub struct GetBlockLayoutWrapper<T>(T);

impl<H: Hal, T: Transport, U: Deref<Target = VirtIOBlk<H, T>>> GetBlockLayout
    for GetBlockLayoutWrapper<U>
{
    fn get_block_size(&mut self) -> usize {
        SECTOR_SIZE
    }

    fn get_num_blocks(&mut self) -> u64 {
        self.0.deref().capacity()
    }
}
