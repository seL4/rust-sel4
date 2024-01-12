//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeapBounds};
use sel4_sync::{AbstractMutexSyncOps, GenericRawMutex};

use crate::{get_static_heap_bounds, get_static_heap_mutex_notification};

#[global_allocator]
#[allow(clippy::type_complexity)]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
    GenericRawMutex<AbstractMutexSyncOps<fn(), fn()>>,
    fn() -> StaticHeapBounds,
> = StaticDlmallocGlobalAlloc::new(
    GenericRawMutex::new(AbstractMutexSyncOps {
        signal: || {
            get_static_heap_mutex_notification().signal();
        },
        wait: || {
            get_static_heap_mutex_notification().wait();
        },
    }),
    get_static_heap_bounds,
);
