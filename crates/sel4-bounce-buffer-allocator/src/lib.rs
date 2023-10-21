//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(allocator_api)]
#![feature(btree_cursors)]
#![feature(btreemap_alloc)]
#![feature(int_roundings)]
#![feature(pointer_is_aligned)]

extern crate alloc;

use core::alloc::Layout;
use core::fmt;
use core::ops::Range;

type Offset = usize;
type Size = usize;
type Align = usize;

mod basic;
mod bump;

pub use basic::Basic;
pub use bump::Bump;

const MIN_ALLOCATION_SIZE: Size = 1;

pub trait AbstractBounceBufferAllocator {
    type Error: fmt::Debug;

    fn allocate(&mut self, layout: Layout) -> Result<Offset, Self::Error>;

    fn deallocate(&mut self, offset: Offset, size: Size);
}

pub struct BounceBufferAllocator<T> {
    abstract_allocator: T,
    max_alignment: Align,
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
        assert!(region.is_aligned_to(self.max_alignment()));
    }
}

impl<T: AbstractBounceBufferAllocator> BounceBufferAllocator<T> {
    pub fn allocate(&mut self, layout: Layout) -> Result<Range<Offset>, T::Error> {
        assert!(layout.align() <= self.max_alignment);
        assert!(layout.size() >= MIN_ALLOCATION_SIZE);
        let start = self.abstract_allocator.allocate(layout)?;
        let end = start + layout.size();
        Ok(start..end)
    }

    pub fn deallocate(&mut self, buffer: Range<Offset>) {
        self.abstract_allocator
            .deallocate(buffer.start, buffer.len())
    }
}
