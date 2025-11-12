//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::collections::BTreeMap;
use core::alloc::Layout;
use core::ops::Range;

use crate::{AbstractAllocator, AbstractAllocatorAllocation};

pub struct ByRange<A: AbstractAllocator> {
    inner: A,
    allocations: BTreeMap<RangeKey<usize>, A::Allocation>,
}

impl<A: AbstractAllocator> ByRange<A> {
    pub const fn new(inner: A) -> Self {
        Self {
            inner,
            allocations: BTreeMap::new(),
        }
    }

    pub fn allocate(&mut self, layout: Layout) -> Result<Range<usize>, A::AllocationError> {
        let allocation = self.inner.allocate(layout)?;
        let range = allocation.range();
        self.allocations.insert(range.clone().into(), allocation);
        Ok(range)
    }

    pub fn deallocate(&mut self, range: Range<usize>) {
        let allocation = self.allocations.remove(&range.into()).unwrap();
        self.inner.deallocate(allocation)
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct RangeKey<T> {
    start: T,
    end: T,
}

impl<T> From<Range<T>> for RangeKey<T> {
    fn from(range: Range<T>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl<T> From<RangeKey<T>> for Range<T> {
    fn from(range: RangeKey<T>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}
