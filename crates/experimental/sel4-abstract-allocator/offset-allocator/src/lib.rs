//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::alloc::Layout;
use core::ops::Range;

use offset_allocator::NodeIndex;

use sel4_abstract_allocator::{AbstractAllocator, AbstractAllocatorAllocation};

pub struct OffsetAllocator<NI: NodeIndex = u16> {
    inner: offset_allocator::Allocator<NI>,
}

impl<NI: NodeIndex> OffsetAllocator<NI> {
    pub fn new(size: usize) -> Self {
        Self {
            inner: offset_allocator::Allocator::new(
                size.try_into()
                    .unwrap_or_else(|_| panic_with_u32_limitation()),
            ),
        }
    }

    pub fn with_max_allocs(size: usize, max_allocs: u32) -> Self {
        Self {
            inner: offset_allocator::Allocator::with_max_allocs(
                size.try_into()
                    .unwrap_or_else(|_| panic_with_u32_limitation()),
                max_allocs,
            ),
        }
    }
}

impl<NI: NodeIndex> AbstractAllocator for OffsetAllocator<NI> {
    type AllocationError = InsufficientResources;

    type Allocation = Allocation<NI>;

    fn allocate(&mut self, layout: Layout) -> Result<Self::Allocation, Self::AllocationError> {
        let req_size = layout.size() + layout.align() - 1;
        let req_size = req_size
            .try_into()
            .unwrap_or_else(|_| panic_with_u32_limitation());
        match self.inner.allocate(req_size) {
            Some(allocation) => Ok(Allocation::new(allocation, layout)),
            None => Err(InsufficientResources::new()),
        }
    }

    fn deallocate(&mut self, allocation: Self::Allocation) {
        self.inner.free(allocation.inner)
    }
}

pub struct Allocation<NI: NodeIndex> {
    inner: offset_allocator::Allocation<NI>,
    range: Range<usize>,
}

impl<NI: NodeIndex> Allocation<NI> {
    fn new(inner: offset_allocator::Allocation<NI>, layout: Layout) -> Self {
        let allocation_start: usize = inner.offset.to_usize();
        let start = allocation_start.next_multiple_of(layout.align());
        let end = start + layout.size();
        Self {
            inner,
            range: start..end,
        }
    }
}

impl<NI: NodeIndex> AbstractAllocatorAllocation for Allocation<NI> {
    fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct InsufficientResources(());

impl InsufficientResources {
    fn new() -> Self {
        Self(())
    }
}

fn panic_with_u32_limitation() -> ! {
    panic!("offset-allocator only supports u32 sizes")
}
