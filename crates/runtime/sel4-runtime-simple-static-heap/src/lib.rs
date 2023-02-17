#![no_std]

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
        $vis:vis $ident:ident: $size:expr;
    } => {
        #[global_allocator]
        $vis static $ident: $crate::GlobalAllocator = {
            static mut STATIC_HEAP: $crate::StaticHeap<{ $size }> = $crate::StaticHeap::new();
            $crate::new_global_allocator(unsafe { STATIC_HEAP.bounds() })
        };
    }
}
