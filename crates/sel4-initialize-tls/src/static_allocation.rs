//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::alloc::Layout;
use core::cell::UnsafeCell;

use crate::TlsImage;

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
}

impl<const N: usize, A> Default for StaticTlsAllocation<N, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsImage {
    pub fn initialize_static_allocation<const N: usize, A>(
        &self,
        allocation: &StaticTlsAllocation<N, A>,
    ) -> Result<usize, InitializeStaticTlsAllocationError> {
        let layout = self.reservation_layout();
        let footprint = layout.footprint();
        let align_offset = allocation.start().align_offset(layout.footprint().align());
        if align_offset + footprint.size() > allocation.size() {
            return Err(InitializeStaticTlsAllocationError::AllocationTooSmall {
                requested_footprint: footprint,
                align_offset,
            });
        }
        let start = allocation.start().wrapping_byte_add(align_offset);
        unsafe {
            self.initialize_tls_reservation(start);
        };
        Ok((allocation.start() as usize) + layout.thread_pointer_offset())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InitializeStaticTlsAllocationError {
    AllocationTooSmall {
        requested_footprint: Layout,
        align_offset: usize,
    },
}
