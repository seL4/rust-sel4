use sel4_dlmalloc::StaticDlmallocGlobalAlloc;
use sel4_sync::AbstractMutexSyncOps;

use crate::{get_static_heap_bounds, get_static_heap_mutex_notification};

#[global_allocator]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
    AbstractMutexSyncOps<fn(), fn()>,
    fn() -> *mut [u8],
> = StaticDlmallocGlobalAlloc::new(
    AbstractMutexSyncOps {
        signal: || {
            get_static_heap_mutex_notification().signal();
        },
        wait: || {
            get_static_heap_mutex_notification().wait();
        },
    },
    get_static_heap_bounds,
);
