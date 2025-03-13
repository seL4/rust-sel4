//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::{RefCell, UnsafeCell};
use core::mem;
use core::ptr;

use dlmalloc::{Allocator, Dlmalloc};
use lock_api::{Mutex, RawMutex};

pub struct StaticDlmallocGlobalAlloc<R, T> {
    dlmalloc: Mutex<R, Dlmalloc<StaticDlmallocAllocator<T>>>,
}

impl<R, T> StaticDlmallocGlobalAlloc<R, T> {
    pub const fn new(raw_mutex: R, get_bounds: T) -> Self {
        Self {
            dlmalloc: Mutex::from_raw(
                raw_mutex,
                Dlmalloc::new_with_allocator(StaticDlmallocAllocator::new(get_bounds)),
            ),
        }
    }
}

impl<R: RawMutex, T> StaticDlmallocGlobalAlloc<R, T> {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn raw_mutex(&self) -> &R {
        self.dlmalloc.raw()
    }
}

unsafe impl<R: RawMutex, T: GetStaticHeapBounds + Send> GlobalAlloc
    for StaticDlmallocGlobalAlloc<R, T>
{
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

struct StaticDlmallocAllocator<T> {
    state: RefCell<StaticDlmallocAllocatorState<T>>,
}

unsafe impl<T: Send> Send for StaticDlmallocAllocatorState<T> {}

enum StaticDlmallocAllocatorState<T> {
    Uninitialized { get_initial_bounds: T },
    Initializing,
    Initialized { free: Free },
}

// TODO: ptr, watermark: usize, size: usize
struct Free {
    watermark: *mut u8,
    end: *mut u8,
}

impl Free {
    fn new(bounds: StaticHeapBounds) -> Self {
        let end = bounds.ptr.wrapping_add(bounds.size);
        Self {
            watermark: bounds.ptr,
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
    pub const fn new(get_initial_bounds: T) -> Self {
        Self {
            state: RefCell::new(StaticDlmallocAllocatorState::Uninitialized { get_initial_bounds }),
        }
    }
}

impl<T: GetStaticHeapBounds> StaticDlmallocAllocatorState<T> {
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

unsafe impl<T: GetStaticHeapBounds + Send> Allocator for StaticDlmallocAllocator<T> {
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

pub trait GetStaticHeapBounds {
    fn bounds(self) -> StaticHeapBounds;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StaticHeapBounds {
    ptr: *mut u8,
    size: usize,
}

impl StaticHeapBounds {
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        Self { ptr, size }
    }
}

impl<T: FnOnce() -> StaticHeapBounds> GetStaticHeapBounds for T {
    fn bounds(self) -> StaticHeapBounds {
        (self)()
    }
}

#[repr(C)]
pub struct StaticHeap<const N: usize, A = ()> {
    _alignment: [A; 0],
    space: UnsafeCell<[u8; N]>,
}

unsafe impl<const N: usize, A> Sync for StaticHeap<N, A> {}

impl<const N: usize, A> StaticHeap<N, A> {
    pub const fn new() -> Self {
        Self {
            _alignment: [],
            space: UnsafeCell::new([0; N]),
        }
    }
}

impl<const N: usize, A> Default for StaticHeap<N, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> GetStaticHeapBounds for &StaticHeap<N> {
    fn bounds(self) -> StaticHeapBounds {
        StaticHeapBounds::new(self.space.get().cast(), N)
    }
}
