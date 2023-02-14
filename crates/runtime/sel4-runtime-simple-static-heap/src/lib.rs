#![no_std]
#![feature(const_option_ext)]

use core::ops::Range;

use sel4_dlmalloc::{StaticDlmallocGlobalAlloc, StaticHeap};
use sel4_env_literal_helper::env_literal;
use sel4_sync::DeferredNotificationMutexSyncOps;

const STATIC_HEAP_SIZE: usize = env_literal!("SEL4_RUNTIME_HEAP_SIZE").unwrap_or(0);

static mut STATIC_HEAP: StaticHeap<STATIC_HEAP_SIZE> = StaticHeap::new();

#[global_allocator]
pub static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<
    DeferredNotificationMutexSyncOps,
    fn() -> Range<*mut u8>,
> = StaticDlmallocGlobalAlloc::new(DeferredNotificationMutexSyncOps::new(), || unsafe {
    STATIC_HEAP.bounds()
});
