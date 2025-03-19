//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::collections::BTreeMap;
use core::alloc::Layout;
use core::ops::Range;

use crate::{AbstractAllocator, AbstractAllocatorAllocation};

type Offset = usize;
type Size = usize;

const GRANULE_SIZE: usize = 2048;

// TODO
// This is just a temporary implementation to serve as a stand-in.

// NOTE
// Once #![feature(allocator_api)] and #![feature(btreemap_alloc)] land, Should be parameterized
// with an allocator A, to enable this type to be used without a global allocator.

// NOTE
// #![feature(btree_cursors)] would make this stand-in implementation simpler and more efficient.
// See git history.

pub struct BasicAllocator {
    holes: BTreeMap<Offset, Size>,
}

impl BasicAllocator {
    pub fn new(size: Size) -> Self {
        assert_eq!(size % GRANULE_SIZE, 0);
        let offset = 0;
        let mut holes = BTreeMap::new();
        holes.insert(offset, size);
        Self { holes }
    }
}

impl AbstractAllocator for BasicAllocator {
    type AllocationError = InsufficientResources;

    type Allocation = Allocation;

    fn allocate(&mut self, orig_layout: Layout) -> Result<Self::Allocation, Self::AllocationError> {
        let layout = Layout::from_size_align(
            orig_layout.size().next_multiple_of(GRANULE_SIZE),
            orig_layout.align().max(GRANULE_SIZE),
        )
        .unwrap();

        let (buffer_offset, hole_offset, hole_size) = self
            .holes
            .iter()
            .find_map(|(&hole_offset, &hole_size)| {
                let buffer_offset = hole_offset.next_multiple_of(layout.align());
                if buffer_offset + layout.size() <= hole_offset + hole_size {
                    Some((buffer_offset, hole_offset, hole_size))
                } else {
                    None
                }
            })
            .ok_or(InsufficientResources::new())?;

        self.holes.remove(&hole_offset).unwrap();

        if hole_offset < buffer_offset {
            self.holes.insert(hole_offset, buffer_offset - hole_offset);
        }

        let hole_end_offset = hole_offset + hole_size;
        let buffer_end_offset = buffer_offset + layout.size();

        if buffer_end_offset < hole_end_offset {
            self.holes
                .insert(buffer_end_offset, hole_end_offset - buffer_end_offset);
        }

        Ok(Allocation::new(
            buffer_offset..(buffer_offset + orig_layout.size()),
        ))
    }

    fn deallocate(&mut self, allocation: Self::Allocation) {
        let range = allocation.range();
        let offset = range.start;
        let size = range.len();

        assert_eq!(offset % GRANULE_SIZE, 0);
        let size = size.next_multiple_of(GRANULE_SIZE);

        let holes = self
            .holes
            .range(..&offset)
            .next_back()
            .map(copy_typle_fields)
            .map(|prev_hole| {
                (
                    prev_hole,
                    self.holes.range(&offset..).next().map(copy_typle_fields),
                )
            });

        let mut island = true;

        if let Some(((prev_hole_offset, prev_hole_size), next_hole)) = holes {
            assert!(prev_hole_offset + prev_hole_size <= offset);
            let adjacent_to_prev = prev_hole_offset + prev_hole_size == offset;
            if adjacent_to_prev {
                island = false;
                *self.holes.get_mut(&prev_hole_offset).unwrap() += size;
            }
            if let Some((next_hole_offset, next_hole_size)) = next_hole {
                assert!(offset + size <= next_hole_offset);
                let adjacent_to_next = offset + size == next_hole_offset;
                if adjacent_to_next {
                    island = false;
                    self.holes.remove(&next_hole_offset).unwrap();
                    if adjacent_to_prev {
                        *self.holes.get_mut(&prev_hole_offset).unwrap() += next_hole_size;
                    } else {
                        self.holes.insert(offset, size + next_hole_size);
                    }
                }
            }
        }

        if island {
            self.holes.insert(offset, size);
        }
    }
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

fn copy_typle_fields<T: Copy, U: Copy>((&t, &u): (&T, &U)) -> (T, U) {
    (t, u)
}
