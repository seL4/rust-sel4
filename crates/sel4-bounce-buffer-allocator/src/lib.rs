//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use core::alloc::Layout;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Range;

type Offset = usize;
type Size = usize;
type Align = usize;

mod basic;
mod bump;

pub use basic::Basic;
pub use bump::Bump;

const MIN_ALLOCATION_SIZE: Size = 1;

// TODO use u64 instead of usize, or abstract

pub trait AbstractBounceBufferAllocator {
    type Error: fmt::Debug;

    fn allocate(&mut self, layout: Layout) -> Result<Offset, Self::Error>;

    fn deallocate(&mut self, offset: Offset, size: Size);
}

pub trait AbstractBounceBufferAllocatorWithRangeTracking: AbstractBounceBufferAllocator {
    // fn deallocate_by_range(&mut self, offset: Offset, size: Size);
}

pub struct BounceBufferAllocator<T> {
    abstract_allocator: T,
    max_alignment: Align,
}

pub struct BounceBufferAllocation<T> {
    range: Range<Offset>,
    _phantom: PhantomData<T>,
}

impl<T> BounceBufferAllocation<T> {
    const fn new(range: Range<Offset>) -> Self {
        Self {
            range,
            _phantom: PhantomData,
        }
    }

    pub fn range(&self) -> Range<Offset> {
        self.range.clone()
    }
}

impl<T> BounceBufferAllocator<T> {
    pub fn new(abstract_allocator: T, max_alignment: Align) -> Self {
        assert!(max_alignment.is_power_of_two());
        Self {
            abstract_allocator,
            max_alignment,
        }
    }

    pub fn max_alignment(&self) -> Align {
        self.max_alignment
    }

    pub fn check_alignment(&self, region: *mut u8) {
        assert_eq!(region.cast::<()>().align_offset(self.max_alignment()), 0); // sanity check
    }
}

impl<T: AbstractBounceBufferAllocator> BounceBufferAllocator<T> {
    pub fn allocate(&mut self, layout: Layout) -> Result<BounceBufferAllocation<T>, T::Error> {
        assert!(layout.align() <= self.max_alignment);
        assert!(layout.size() >= MIN_ALLOCATION_SIZE);
        let start = self.abstract_allocator.allocate(layout)?;
        let end = start + layout.size();
        Ok(BounceBufferAllocation::new(start..end))
    }

    pub fn deallocate(&mut self, allocation: BounceBufferAllocation<T>) {
        self.abstract_allocator
            .deallocate(allocation.range.start, allocation.range.len())
    }
}

impl<T: AbstractBounceBufferAllocatorWithRangeTracking> BounceBufferAllocator<T> {
    pub fn deallocate_by_range(&mut self, allocation_range: Range<usize>) {
        self.abstract_allocator
            .deallocate(allocation_range.start, allocation_range.len())
    }
}
