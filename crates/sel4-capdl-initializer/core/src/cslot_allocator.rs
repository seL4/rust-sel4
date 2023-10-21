//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;

use sel4::InitCSpaceSlot;

pub(crate) struct CSlotAllocator {
    free: Range<InitCSpaceSlot>,
}

#[derive(Debug)]
pub enum CSlotAllocatorError {
    OutOfSlots,
}

impl CSlotAllocator {
    pub(crate) fn new(free: Range<InitCSpaceSlot>) -> Self {
        Self { free }
    }

    pub(crate) fn alloc(&mut self) -> Result<InitCSpaceSlot, CSlotAllocatorError> {
        self.free.next().ok_or(CSlotAllocatorError::OutOfSlots)
    }

    pub(crate) fn alloc_or_panic(&mut self) -> InitCSpaceSlot {
        self.alloc().unwrap()
    }
}
