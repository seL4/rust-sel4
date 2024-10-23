//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::cell::UnsafeCell;

// NOTE(rustc_wishlist) use SyncUnsafeCell once #![feature(sync_unsafe_cell)] stabilizes
#[repr(C)]
pub struct StaticHeap<const N: usize, A = ()> {
    _alignment: [A; 0],
    space: UnsafeCell<[u8; N]>,
}

unsafe impl<const N: usize, A> Sync for StaticHeap<N, A> {}

impl<const N: usize, A> StaticHeap<N, A> {
    pub const fn new() -> Self {
        Self {
            _alignment: [],
            space: UnsafeCell::new([0; N]),
        }
    }

    pub const fn size(&self) -> usize {
        N
    }

    pub const fn start(&self) -> *mut u8 {
        self.space.get().cast()
    }

    pub const fn end(&self) -> *mut u8 {
        self.space.get().cast::<u8>().wrapping_add(self.size())
    }
}

impl<const N: usize, A> Default for StaticHeap<N, A> {
    fn default() -> Self {
        Self::new()
    }
}
