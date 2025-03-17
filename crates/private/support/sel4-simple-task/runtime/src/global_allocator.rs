//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_dlmalloc::{DeferredStaticDlmalloc, StaticHeapBounds};
use sel4_sync::DeferredRawNotificationMutex;

#[global_allocator]
#[allow(clippy::type_complexity)]
static GLOBAL_ALLOCATOR: DeferredStaticDlmalloc<DeferredRawNotificationMutex> =
    DeferredStaticDlmalloc::new();

pub(crate) fn init(nfn: sel4::cap::Notification, bounds: StaticHeapBounds) {
    let _ = unsafe { GLOBAL_ALLOCATOR.raw_mutex().set_notification(nfn) };
    let _ = GLOBAL_ALLOCATOR.set_bounds(bounds);
}
