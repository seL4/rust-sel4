#![no_std]
#![feature(strict_provenance)]

use core::ops::Range;

use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeap};
use sel4_sync::DeferredNotificationMutexSyncOps;

const STATIC_HEAP_SIZE: usize = include!(concat!(env!("OUT_DIR"), "/heap_size.fragment.rs"));

static mut STATIC_HEAP: StaticHeap<STATIC_HEAP_SIZE> = StaticHeap::new();

#[global_allocator]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
    DeferredNotificationMutexSyncOps,
    fn() -> Range<*mut u8>,
> = StaticDlmallocGlobalAlloc::new(DeferredNotificationMutexSyncOps::new(), || unsafe {
    STATIC_HEAP.bounds()
});
