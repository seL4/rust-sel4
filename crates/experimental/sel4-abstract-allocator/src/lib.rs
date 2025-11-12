//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::alloc::Layout;
use core::fmt;
use core::ops::Range;

pub mod bump;

#[cfg(feature = "alloc")]
pub mod basic;

#[cfg(feature = "alloc")]
mod by_range;

#[cfg(feature = "alloc")]
pub use by_range::ByRange;

// TODO consider using u64 instead of usize, or abstract

pub trait AbstractAllocatorAllocation {
    fn range(&self) -> Range<usize>;
}

pub trait AbstractAllocator {
    type AllocationError: fmt::Debug;

    type Allocation: AbstractAllocatorAllocation;

    fn allocate(&mut self, layout: Layout) -> Result<Self::Allocation, Self::AllocationError>;

    fn deallocate(&mut self, allocation: Self::Allocation);
}

// // //

pub struct WithAlignmentBound<A> {
    inner: A,
    max_alignment: usize,
}

impl<A> WithAlignmentBound<A> {
    pub const fn new(inner: A, max_alignment: usize) -> Self {
        Self {
            inner,
            max_alignment,
        }
    }

    pub const fn max_alignment(&self) -> usize {
        self.max_alignment
    }

    #[must_use]
    pub fn is_suitably_aligned(&self, region: *mut u8) -> bool {
        region.cast::<()>().align_offset(self.max_alignment()) == 0
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum WithAlignmentBoundAllocationError<E> {
    InnerError(E),
    AlignmentExceedsBound,
}

impl<A: AbstractAllocator> AbstractAllocator for WithAlignmentBound<A> {
    type AllocationError = WithAlignmentBoundAllocationError<A::AllocationError>;

    type Allocation = A::Allocation;

    fn allocate(&mut self, layout: Layout) -> Result<Self::Allocation, Self::AllocationError> {
        if layout.align() > self.max_alignment() {
            return Err(WithAlignmentBoundAllocationError::AlignmentExceedsBound);
        }
        self.inner
            .allocate(layout)
            .map_err(WithAlignmentBoundAllocationError::InnerError)
    }

    fn deallocate(&mut self, allocation: Self::Allocation) {
        self.inner.deallocate(allocation)
    }
}
