//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::alloc::Global;
use alloc::collections::BTreeMap;
use core::alloc::{Allocator, Layout};
use core::ops::Bound;

use crate::{AbstractBounceBufferAllocator, Offset, Size};

const GRANULE_SIZE: usize = 2048;

pub struct Basic<A: Allocator + Clone = Global> {
    holes: BTreeMap<Offset, Size, A>,
}

impl Basic {
    pub fn new(size: Size) -> Self {
        Self::new_in(Global, size)
    }
}

impl<A: Allocator + Clone> Basic<A> {
    pub fn new_in(alloc: A, size: Size) -> Self {
        assert_eq!(size % GRANULE_SIZE, 0);
        let offset = 0;
        let mut holes = BTreeMap::new_in(alloc.clone());
        holes.insert(offset, size);
        Self { holes }
    }
}

impl<A: Allocator + Clone> AbstractBounceBufferAllocator for Basic<A> {
    type Error = ();

    fn allocate(&mut self, layout: Layout) -> Result<Offset, Self::Error> {
        let layout = Layout::from_size_align(
            layout.size().next_multiple_of(GRANULE_SIZE),
            layout.align().max(GRANULE_SIZE),
        )
        .unwrap();
        let mut cursor = self.holes.lower_bound_mut(Bound::Unbounded);
        loop {
            if let Some((&hole_offset, &hole_size)) = cursor.key_value() {
                let buffer_offset = hole_offset.next_multiple_of(layout.align());
                if buffer_offset + layout.size() <= hole_offset + hole_size {
                    cursor.remove_current();
                    if hole_offset < buffer_offset {
                        cursor.insert_before(hole_offset, buffer_offset - hole_offset);
                    }
                    if buffer_offset + layout.size() < hole_offset + hole_size {
                        cursor.insert_before(
                            buffer_offset + layout.size(),
                            (hole_offset + hole_size) - (buffer_offset + layout.size()),
                        );
                    }
                    return Ok(buffer_offset);
                } else {
                    cursor.move_next();
                }
            } else {
                return Err(());
            }
        }
    }

    fn deallocate(&mut self, offset: Offset, size: Size) {
        assert_eq!(offset % GRANULE_SIZE, 0);
        let size = size.next_multiple_of(GRANULE_SIZE);
        let mut cursor = self.holes.upper_bound_mut(Bound::Included(&offset));
        if let Some((&prev_hole_offset, prev_hole_size)) = cursor.key_value_mut() {
            assert!(prev_hole_offset + *prev_hole_size <= offset);
            if prev_hole_offset + *prev_hole_size == offset {
                *prev_hole_size += size;
                return;
            }
        }
        cursor.move_next();
        if let Some((&next_hole_offset, &next_hole_size)) = cursor.key_value() {
            assert!(offset + size <= next_hole_offset);
            if offset + size == next_hole_offset {
                cursor.remove_current();
                cursor.insert_before(offset, size + next_hole_size);
                return;
            }
        }
        cursor.insert_before(offset, size);
    }
}
