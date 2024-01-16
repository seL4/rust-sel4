//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_dlmalloc::StaticDlmallocGlobalAlloc;
use sel4_sync::{GenericRawMutex, PanickingMutexSyncOps};

pub use sel4_dlmalloc::StaticHeap;

#[doc(hidden)]
pub type GlobalAllocator<const N: usize> =
    StaticDlmallocGlobalAlloc<GenericRawMutex<PanickingMutexSyncOps>, &'static StaticHeap<N>>;

#[doc(hidden)]
pub const fn new_global_allocator<const N: usize>(
    bounds: &'static StaticHeap<N>,
) -> GlobalAllocator<N> {
    StaticDlmallocGlobalAlloc::new(GenericRawMutex::new(PanickingMutexSyncOps::new()), bounds)
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_heap {
    ($size:expr) => {
        const _: () = {
            #[global_allocator]
            static GLOBAL_ALLOCATOR: $crate::_private::heap::GlobalAllocator<{ $size }> = {
                static STATIC_HEAP: $crate::_private::heap::StaticHeap<{ $size }> =
                    $crate::_private::heap::StaticHeap::new();
                $crate::_private::heap::new_global_allocator(&STATIC_HEAP)
            };
        };
    };
}

pub mod _private {
    pub use super::{new_global_allocator, GlobalAllocator, StaticHeap};
}
