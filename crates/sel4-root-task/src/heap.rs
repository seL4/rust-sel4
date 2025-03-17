//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_immediate_sync_once_cell::ImmediateSyncOnceCell;
use sel4_panicking_env::abort;

static GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION: ImmediateSyncOnceCell<sel4::cap::Notification> =
    ImmediateSyncOnceCell::new();

/// Provides the global allocator with a [`sel4::cap::Notification`] to use as a mutex..
///
/// Until this function is used, contention in the global allocator will result in a panic. This is
/// only useful for multi-threaded root tasks.
pub fn set_global_allocator_mutex_notification(nfn: sel4::cap::Notification) {
    GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION
        .set(nfn)
        .unwrap_or_else(|_| abort!("global allocator mutex notification already initialized"))
}

#[doc(hidden)]
pub fn get_global_allocator_mutex_notification() -> sel4::cap::Notification {
    *GLOBAL_ALLOCATOR_MUTEX_NOTIFICATION
        .get()
        .unwrap_or_else(|| {
            abort!("global allocator contention before mutex notification initialization")
        })
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_heap {
    ($size:expr) => {
        const _: () = {
            mod _scope {
                mod size_scope {
                    use super::super::*;
                    pub(super) const SIZE: usize = $size;
                }

                use $crate::_private::heap::*;

                static STATIC_HEAP: StaticHeap<{ size_scope::SIZE }> = StaticHeap::new();

                #[global_allocator]
                static GLOBAL_ALLOCATOR: StaticDlmalloc<LazyRawNotificationMutex> =
                    StaticDlmalloc::new_with(
                        LazyRawNotificationMutex::new(get_global_allocator_mutex_notification),
                        STATIC_HEAP.bounds(),
                    );
            }
        };
    };
}

pub mod _private {
    pub use sel4_dlmalloc::{StaticDlmalloc, StaticHeap};
    pub use sel4_sync::LazyRawNotificationMutex;

    pub use super::get_global_allocator_mutex_notification;
}
