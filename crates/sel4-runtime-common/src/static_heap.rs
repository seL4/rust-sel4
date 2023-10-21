//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_dlmalloc::{ConstantStaticHeapBounds, StaticDlmallocGlobalAlloc};
use sel4_sync::DeferredNotificationMutexSyncOps;

pub use sel4_dlmalloc::StaticHeap;

pub type GlobalAllocator =
    StaticDlmallocGlobalAlloc<DeferredNotificationMutexSyncOps, ConstantStaticHeapBounds>;

pub const fn new_global_allocator(bounds: ConstantStaticHeapBounds) -> GlobalAllocator {
    StaticDlmallocGlobalAlloc::new(DeferredNotificationMutexSyncOps::new(), bounds)
}

#[macro_export]
macro_rules! declare_static_heap {
    {
        $(#[$attrs:meta])*
        $vis:vis $ident:ident: $size:expr;
    } => {
        #[global_allocator]
        $(#[$attrs])*
        $vis static $ident: $crate::_private::static_heap::GlobalAllocator = {
            static mut STATIC_HEAP: $crate::_private::static_heap::StaticHeap<{ $size }> =
                $crate::_private::static_heap::StaticHeap::new();
            $crate::_private::static_heap::new_global_allocator(unsafe { STATIC_HEAP.bounds() })
        };
    }
}

pub mod _private {
    pub use super::{new_global_allocator, GlobalAllocator, StaticHeap};
}
