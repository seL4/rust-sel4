//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::UnsafeCell;

use crate::Region;

#[repr(C)]
pub struct StaticTlsAllocation<const N: usize, A = ()> {
    _alignment: [A; 0],
    space: UnsafeCell<[u8; N]>,
}

unsafe impl<const N: usize, A> Sync for StaticTlsAllocation<N, A> {}

impl<const N: usize, A> StaticTlsAllocation<N, A> {
    pub const fn new() -> Self {
        Self {
            _alignment: [],
            space: UnsafeCell::new([0; N]),
        }
    }

    const fn size(&self) -> usize {
        N
    }

    const fn start(&self) -> *mut u8 {
        self.space.get().cast()
    }

    pub const fn region(&self) -> Region {
        Region::new(self.start(), self.size())
    }
}

impl<const N: usize, A> Default for StaticTlsAllocation<N, A> {
    fn default() -> Self {
        Self::new()
    }
}
