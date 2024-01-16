//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_dlmalloc::StaticDlmallocGlobalAlloc;
use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking_env::abort;
use sel4_sync::{GenericRawMutex, IndirectNotificationMutexSyncOps};

pub use sel4_dlmalloc::StaticHeap;

pub fn set_global_allocator_mutex_notification(nfn: sel4::Notification) {
    GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION
        .set(nfn)
        .unwrap_or_else(|_| abort!("global allocator mutex notification already initialized"))
}

static GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION: ImmediateSyncOnceCell<sel4::Notification> =
    ImmediateSyncOnceCell::new();

#[doc(hidden)]
pub type GlobalAllocator<const N: usize> = StaticDlmallocGlobalAlloc<
    GenericRawMutex<IndirectNotificationMutexSyncOps<fn() -> sel4::Notification>>,
    &'static StaticHeap<N>,
>;

#[doc(hidden)]
pub const fn new_global_allocator<const N: usize>(
    bounds: &'static StaticHeap<N>,
) -> GlobalAllocator<N> {
    StaticDlmallocGlobalAlloc::new(
        GenericRawMutex::new(IndirectNotificationMutexSyncOps::new(|| {
            *GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION
                .get()
                .unwrap_or_else(|| {
                    abort!("global allocator contention before mutex notification initialization")
                })
        })),
        bounds,
    )
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
