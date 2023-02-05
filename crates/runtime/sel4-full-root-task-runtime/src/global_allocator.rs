use core::ops::Range;

use sel4_dlmalloc::StaticDlmallocGlobalAlloc;
use sel4_sync::PanickingMutexSyncOps;

const STATIC_HEAP_SIZE: usize = include!(concat!(env!("OUT_DIR"), "/heap_size.fragment.rs"));

// TODO(nspin) does dlmalloc assume align(PAGE_SIZE)?
#[repr(C, align(16))]
struct StaticHeap([u8; STATIC_HEAP_SIZE]);

static mut STATIC_HEAP: StaticHeap = StaticHeap([0; STATIC_HEAP_SIZE]);

fn get_static_heap_bounds() -> Range<usize> {
    unsafe {
        let ptr = &mut STATIC_HEAP as *mut StaticHeap;
        let start = ptr.expose_addr();
        let end = ptr.offset(1).expose_addr();
        start..end
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: StaticDlmallocGlobalAlloc<PanickingMutexSyncOps, fn() -> Range<usize>> =
    StaticDlmallocGlobalAlloc::new(PanickingMutexSyncOps, get_static_heap_bounds);
