//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::alloc::Layout;
use core::ops::Range;

use crate::{AbstractAllocator, AbstractAllocatorAllocation};

pub struct BumpAllocator {
    watermark: usize,
    size: usize,
}

impl BumpAllocator {
    pub fn new(size: usize) -> Self {
        Self { watermark: 0, size }
    }
}

impl AbstractAllocator for BumpAllocator {
    type AllocationError = InsufficientResources;

    type Allocation = Allocation;

    fn allocate(&mut self, layout: Layout) -> Result<Self::Allocation, Self::AllocationError> {
        let start = self.watermark.next_multiple_of(layout.align());
        let end = start + layout.size();
        if end > self.size {
            return Err(InsufficientResources::new());
        }
        self.watermark = end;
        Ok(Allocation::new(start..end))
    }

    fn deallocate(&mut self, _allocation: Self::Allocation) {}
}

pub struct Allocation(Range<usize>);

impl Allocation {
    fn new(range: Range<usize>) -> Self {
        Self(range)
    }
}

impl AbstractAllocatorAllocation for Allocation {
    fn range(&self) -> Range<usize> {
        self.0.clone()
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct InsufficientResources(());

impl InsufficientResources {
    fn new() -> Self {
        Self(())
    }
}
