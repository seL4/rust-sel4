use core::alloc::Layout;
use core::ops::Range;

use sel4_dlmalloc::StaticDlmallocGlobalAlloc;
use sel4_runtime_building_blocks_abort::{abort, debug_println};
use sel4_sync::PanickingMutexSyncOps;

#[cfg(all(feature = "unwinding", feature = "postcard"))]
use crate::backtrace;

const STATIC_HEAP_SIZE: usize = include!(concat!(env!("OUT_DIR"), "/heap_size.fragment.rs"));

// TODO(nspin) does dlmalloc assume align(PAGE_SIZE)?
#[repr(C, align(16))]
struct StaticHeap([u8; STATIC_HEAP_SIZE]);

static mut STATIC_HEAP: StaticHeap = StaticHeap([0; STATIC_HEAP_SIZE]);

fn get_static_heap_bounds() -> Range<usize> {
    unsafe {
        let ptr = &mut STATIC_HEAP as *mut StaticHeap;
        let start = ptr.to_bits();
        let end = ptr.offset(1).to_bits();
        start..end
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<PanickingMutexSyncOps, fn() -> Range<usize>> =
    StaticDlmallocGlobalAlloc::new(PanickingMutexSyncOps, get_static_heap_bounds);

// TODO make recoverable, e.g. by using begin_panic in a way that doesn't use the allocator
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    debug_println!("alloc error with layout: {:?}", layout);

    #[cfg(all(feature = "unwinding", feature = "postcard"))]
    backtrace::collect_and_send();

    abort()
}
