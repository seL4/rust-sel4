//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use sel4::init_thread::Slot;

pub(crate) struct CSlotAllocator {
    free: Range<usize>,
}

#[derive(Debug)]
pub enum CSlotAllocatorError {
    OutOfSlots,
}

impl CSlotAllocator {
    pub(crate) fn new(free: Range<usize>) -> Self {
        Self { free }
    }

    pub(crate) fn alloc(&mut self) -> Result<Slot, CSlotAllocatorError> {
        self.free
            .next()
            .map(Slot::from_index)
            .ok_or(CSlotAllocatorError::OutOfSlots)
    }

    pub(crate) fn alloc_or_panic(&mut self) -> Slot {
        self.alloc().unwrap()
    }
}
