#![no_std]
#![feature(ptr_to_from_bits)]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::RefCell;
use core::ops::Range;
use core::ptr;

use dlmalloc::{Allocator as DlmallocAllocator, Dlmalloc};

use sel4_sync::{GenericMutex, MutexSyncOps};

pub type StaticDlmallocGlobalAlloc<O, T> = DlmallocGlobalAlloc<O, StaticDlmallocAllocator<T>>;

impl<O, T> StaticDlmallocGlobalAlloc<O, T> {
    pub const fn new(mutex_sync_ops: O, get_bounds: T) -> Self {
        Self {
            dlmalloc: GenericMutex::new(
                mutex_sync_ops,
                Dlmalloc::new_with_allocator(StaticDlmallocAllocator::new(get_bounds)),
            ),
        }
    }
}

pub struct DlmallocGlobalAlloc<O, T> {
    dlmalloc: GenericMutex<O, Dlmalloc<T>>,
}

unsafe impl<O: MutexSyncOps, T: DlmallocAllocator> GlobalAlloc for DlmallocGlobalAlloc<O, T> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.dlmalloc.lock().malloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.dlmalloc.lock().calloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.dlmalloc
            .lock()
            .free(ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.dlmalloc
            .lock()
            .realloc(ptr, layout.size(), layout.align(), new_size)
    }
}

pub struct StaticDlmallocAllocator<T> {
    state: RefCell<StaticDlmallocAllocatorState<T>>,
}

enum StaticDlmallocAllocatorState<T> {
    Uninitialized { get_initial_bounds: T },
    Initialized { free: Range<usize> },
}

impl<T> StaticDlmallocAllocator<T> {
    const fn new(get_initial_bounds: T) -> Self {
        Self {
            state: RefCell::new(StaticDlmallocAllocatorState::Uninitialized { get_initial_bounds }),
        }
    }
}

impl<T: Fn() -> Range<usize>> StaticDlmallocAllocatorState<T> {
    fn as_free(&mut self) -> &mut Range<usize> {
        if let StaticDlmallocAllocatorState::Uninitialized { get_initial_bounds } = self {
            *self = StaticDlmallocAllocatorState::Initialized {
                free: (get_initial_bounds)(),
            };
        }
        if let StaticDlmallocAllocatorState::Initialized { free } = self {
            free
        } else {
            unreachable!()
        }
    }
}

unsafe impl<T: Fn() -> Range<usize> + Send> DlmallocAllocator for StaticDlmallocAllocator<T> {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        let mut state = self.state.borrow_mut();
        let free = state.as_free();
        let start = free.start;
        let end = start + size;
        if end > free.end {
            (ptr::null_mut(), 0, 0)
        } else {
            free.start = end;
            (<*mut u8>::from_bits(start), size, 0)
        }
    }

    fn remap(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        ptr::null_mut()
    }

    fn free_part(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        false
    }

    fn free(&self, _ptr: *mut u8, _size: usize) -> bool {
        false
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        false
    }

    fn allocates_zeros(&self) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        4096
    }
}
