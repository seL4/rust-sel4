//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::collections::BTreeMap;
use core::alloc::Layout;

use crate::{AbstractBounceBufferAllocator, Offset, Size};

const GRANULE_SIZE: usize = 2048;

// TODO
// This is just a temporary implementation to serve as a stand-in.

// NOTE(rustc_wishlist)
//
// #![feature(allocator_api)] and #![feature(btreemap_alloc)]
//
// Should be parameterized with an allocator A, to enable this type to be used without a global
// allocator.

// NOTE(rustc_wishlist)
//
// #![feature(btree_cursors)] would make this stand-in implementation simpler and more efficient.
// See git history.

pub struct Basic {
    holes: BTreeMap<Offset, Size>,
}

impl Basic {
    pub fn new(size: Size) -> Self {
        assert_eq!(size % GRANULE_SIZE, 0);
        let offset = 0;
        let mut holes = BTreeMap::new();
        holes.insert(offset, size);
        Self { holes }
    }
}

impl AbstractBounceBufferAllocator for Basic {
    type Error = ();

    fn allocate(&mut self, layout: Layout) -> Result<Offset, Self::Error> {
        let layout = Layout::from_size_align(
            layout.size().next_multiple_of(GRANULE_SIZE),
            layout.align().max(GRANULE_SIZE),
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
            .ok_or(())?;

        self.holes.remove(&hole_offset).unwrap();

        if hole_offset < buffer_offset {
            self.holes.insert(hole_offset, buffer_offset - hole_offset);
        }

        if buffer_offset + layout.size() < hole_offset + hole_size {
            self.holes.insert(
                buffer_offset + layout.size(),
                (hole_offset + hole_size) - (buffer_offset + layout.size()),
            );
        }

        Ok(buffer_offset)
    }

    fn deallocate(&mut self, offset: Offset, size: Size) {
        assert_eq!(offset % GRANULE_SIZE, 0);
        let size = size.next_multiple_of(GRANULE_SIZE);

        let holes = self
            .holes
            .range(..&offset)
            .rev()
            .next()
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

fn copy_typle_fields<T: Copy, U: Copy>((&t, &u): (&T, &U)) -> (T, U) {
    (t, u)
}
