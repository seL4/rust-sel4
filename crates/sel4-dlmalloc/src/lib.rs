//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(sync_unsafe_cell)]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::{RefCell, SyncUnsafeCell};
use core::mem;
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

    pub const fn mutex(&self) -> &GenericMutex<O, Dlmalloc<StaticDlmallocAllocator<T>>> {
        &self.dlmalloc
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

unsafe impl<T: Send> Send for StaticDlmallocAllocatorState<T> {}

enum StaticDlmallocAllocatorState<T> {
    Uninitialized { get_initial_bounds: T },
    Initializing,
    Initialized { free: Free },
}

struct Free {
    watermark: *mut u8,
    end: *mut u8,
}

impl Free {
    fn new(bounds: *mut [u8]) -> Self {
        let start = bounds.as_mut_ptr();
        let end = start.wrapping_add(bounds.len());
        Self {
            watermark: start,
            end,
        }
    }

    fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let start = self.watermark;
        let end = start.wrapping_offset(size.try_into().unwrap());
        if end < start || end > self.end {
            None
        } else {
            self.watermark = end;
            Some(start)
        }
    }
}

impl<T> StaticDlmallocAllocator<T> {
    const fn new(get_initial_bounds: T) -> Self {
        Self {
            state: RefCell::new(StaticDlmallocAllocatorState::Uninitialized { get_initial_bounds }),
        }
    }
}

impl<T: StaticHeapBounds> StaticDlmallocAllocatorState<T> {
    fn as_free(&mut self) -> &mut Free {
        if matches!(self, Self::Uninitialized { .. }) {
            if let Self::Uninitialized { get_initial_bounds } =
                mem::replace(self, Self::Initializing)
            {
                *self = Self::Initialized {
                    free: Free::new(get_initial_bounds.bounds()),
                };
            } else {
                unreachable!()
            }
        }
        if let Self::Initialized { free } = self {
            free
        } else {
            unreachable!()
        }
    }
}

unsafe impl<T: StaticHeapBounds + Send> DlmallocAllocator for StaticDlmallocAllocator<T> {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        match self.state.borrow_mut().as_free().alloc(size) {
            Some(start) => (start, size, 0),
            None => (ptr::null_mut(), 0, 0),
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
        // TODO should depend on configuration
        4096
    }
}

pub trait StaticHeapBounds {
    fn bounds(self) -> *mut [u8];
}

impl<T: FnOnce() -> *mut [u8]> StaticHeapBounds for T {
    fn bounds(self) -> *mut [u8] {
        (self)()
    }
}

// TODO alignment should depend on configuration
// TODO does this alignment provide any benefit?
#[repr(C, align(4096))]
pub struct StaticHeap<const N: usize>(SyncUnsafeCell<[u8; N]>);

impl<const N: usize> StaticHeap<N> {
    pub const fn new() -> Self {
        Self(SyncUnsafeCell::new([0; N]))
    }
}

impl<const N: usize> StaticHeapBounds for &StaticHeap<N> {
    fn bounds(self) -> *mut [u8] {
        ptr::slice_from_raw_parts_mut(self.0.get().cast(), N)
    }
}
