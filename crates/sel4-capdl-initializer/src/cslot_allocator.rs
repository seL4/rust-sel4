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

    pub(crate) fn alloc_many(&mut self, n: usize) -> Result<Range<Slot>, CSlotAllocatorError> {
        let alloc_start = self.free.start;
        let alloc_end = alloc_start.checked_add(n).unwrap();
        if alloc_end > self.free.end {
            return Err(CSlotAllocatorError::OutOfSlots);
        }
        self.free.start = alloc_end;
        Ok(Slot::from_index(alloc_start)..Slot::from_index(alloc_end))
    }
}
