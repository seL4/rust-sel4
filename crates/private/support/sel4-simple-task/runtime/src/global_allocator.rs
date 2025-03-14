//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_dlmalloc::{DeferredStaticDlmalloc, StaticHeapBounds};
use sel4_sync::{GenericRawMutex, MutexSyncOps};

use crate::get_static_heap_mutex_notification;

#[global_allocator]
#[allow(clippy::type_complexity)]
static GLOBAL_ALLOCATOR: DeferredStaticDlmalloc<GenericRawMutex<MutexSyncOpsImpl>> =
    DeferredStaticDlmalloc::new(GenericRawMutex::new(MutexSyncOpsImpl));

pub(crate) fn init(bounds: StaticHeapBounds) {
    let _ = GLOBAL_ALLOCATOR.set_bounds(bounds);
}

struct MutexSyncOpsImpl;

impl MutexSyncOps for MutexSyncOpsImpl {
    fn signal(&self) {
        get_static_heap_mutex_notification().signal();
    }

    fn wait(&self) {
        get_static_heap_mutex_notification().wait();
    }
}
