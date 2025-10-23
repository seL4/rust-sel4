//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::{RefCell, UnsafeCell};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

use dlmalloc::{Allocator, Dlmalloc};
use lock_api::{Mutex, RawMutex};

// TODO implement core::alloc::Allocator for StaticDlmalloc once stable

pub struct StaticDlmalloc<R>(
    SyncDlmalloc<R, SimpleDlmallocAllocatorWrapper<StaticDlmallocAllocator>>,
);

impl<R> StaticDlmalloc<R> {
    pub const fn new_with_raw_mutex(raw_mutex: R, bounds: StaticHeapBounds) -> Self {
        Self(SyncDlmalloc::new(
            raw_mutex,
            SimpleDlmallocAllocatorWrapper::new(StaticDlmallocAllocator::new(bounds)),
        ))
    }
}

impl<R: RawMutex> StaticDlmalloc<R> {
    pub const fn new(bounds: StaticHeapBounds) -> Self {
        Self::new_with_raw_mutex(R::INIT, bounds)
    }
}

impl<R: RawMutex> StaticDlmalloc<R> {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn raw_mutex(&self) -> &R {
        unsafe { self.0.raw_mutex() }
    }
}

unsafe impl<R: RawMutex> GlobalAlloc for StaticDlmalloc<R> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.0.alloc(layout) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { self.0.alloc_zeroed(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.0.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { self.0.realloc(ptr, layout, new_size) }
    }
}

pub struct DeferredStaticDlmalloc<R>(
    SyncDlmalloc<
        R,
        SimpleDlmallocAllocatorWrapper<DeferredStaticDlmallocAllocator<StaticDlmallocAllocator>>,
    >,
);

impl<R> DeferredStaticDlmalloc<R> {
    pub const fn new_with_raw_mutex(raw_mutex: R) -> Self {
        Self(SyncDlmalloc::new(
            raw_mutex,
            SimpleDlmallocAllocatorWrapper::new(DeferredStaticDlmallocAllocator::new()),
        ))
    }
}

impl<R: RawMutex> DeferredStaticDlmalloc<R> {
    pub const fn new() -> Self {
        Self::new_with_raw_mutex(R::INIT)
    }
}

impl<R: RawMutex> Default for DeferredStaticDlmalloc<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: RawMutex> DeferredStaticDlmalloc<R> {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn raw_mutex(&self) -> &R {
        unsafe { self.0.raw_mutex() }
    }

    pub fn set_bounds(&self, bounds: StaticHeapBounds) -> Result<(), BoundsAlreadySetError> {
        self.0
            .dlmalloc
            .lock()
            .allocator()
            .0
            .set(StaticDlmallocAllocator::new(bounds))
    }
}

unsafe impl<R: RawMutex> GlobalAlloc for DeferredStaticDlmalloc<R> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.0.alloc(layout) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { self.0.alloc_zeroed(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.0.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { self.0.realloc(ptr, layout, new_size) }
    }
}

// // //

struct StaticDlmallocAllocator {
    bounds: StaticHeapBounds,
    watermark: AtomicUsize,
}

impl StaticDlmallocAllocator {
    const fn new(bounds: StaticHeapBounds) -> Self {
        Self {
            bounds,
            watermark: AtomicUsize::new(0),
        }
    }
}

impl SimpleDlmallocAllocator for StaticDlmallocAllocator {
    fn alloc_simple(&self, size: usize) -> Option<*mut u8> {
        let old_watermark = self
            .watermark
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old_watermark| {
                let new_watermark = old_watermark.checked_add(size)?;
                if new_watermark > self.bounds.size() {
                    return None;
                }
                Some(new_watermark)
            })
            .ok()?;
        Some(
            self.bounds
                .start()
                .wrapping_offset(old_watermark.try_into().unwrap()),
        )
    }
}

// TODO remove RefCell once this is released:
// https://github.com/alexcrichton/dlmalloc-rs/pull/49
struct DeferredStaticDlmallocAllocator<T> {
    state: RefCell<Option<T>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BoundsAlreadySetError(());

impl<T> DeferredStaticDlmallocAllocator<T> {
    const fn new() -> Self {
        Self {
            state: RefCell::new(None),
        }
    }

    fn set(&self, state: T) -> Result<(), BoundsAlreadySetError> {
        let mut state_opt = self.state.borrow_mut();
        if state_opt.is_none() {
            *state_opt = Some(state);
            Ok(())
        } else {
            Err(BoundsAlreadySetError(()))
        }
    }
}

impl<T: SimpleDlmallocAllocator> SimpleDlmallocAllocator for DeferredStaticDlmallocAllocator<T> {
    fn alloc_simple(&self, size: usize) -> Option<*mut u8> {
        self.state
            .borrow()
            .as_ref()
            .and_then(|state| state.alloc_simple(size))
    }
}

// // //

struct SyncDlmalloc<R, T> {
    dlmalloc: Mutex<R, Dlmalloc<T>>,
}

impl<R, T> SyncDlmalloc<R, T> {
    const fn new(raw_mutex: R, state: T) -> Self {
        Self {
            dlmalloc: Mutex::from_raw(raw_mutex, Dlmalloc::new_with_allocator(state)),
        }
    }
}

impl<R: RawMutex, T> SyncDlmalloc<R, T> {
    #[allow(clippy::missing_safety_doc)]
    unsafe fn raw_mutex(&self) -> &R {
        unsafe { self.dlmalloc.raw() }
    }
}

unsafe impl<R: RawMutex, T: Allocator> GlobalAlloc for SyncDlmalloc<R, T> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.dlmalloc.lock().malloc(layout.size(), layout.align()) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { self.dlmalloc.lock().calloc(layout.size(), layout.align()) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.dlmalloc
                .lock()
                .free(ptr, layout.size(), layout.align())
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            self.dlmalloc
                .lock()
                .realloc(ptr, layout.size(), layout.align(), new_size)
        }
    }
}

trait SimpleDlmallocAllocator: Send {
    fn alloc_simple(&self, size: usize) -> Option<*mut u8>;
}

struct SimpleDlmallocAllocatorWrapper<T>(T);

impl<T> SimpleDlmallocAllocatorWrapper<T> {
    const fn new(inner: T) -> Self {
        Self(inner)
    }
}

unsafe impl<T: SimpleDlmallocAllocator> Allocator for SimpleDlmallocAllocatorWrapper<T> {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        match self.0.alloc_simple(size) {
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

// // //

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StaticHeapBounds {
    ptr: *mut u8,
    size: usize,
}

unsafe impl Send for StaticHeapBounds {}

impl StaticHeapBounds {
    pub const fn new(ptr: *mut u8, size: usize) -> Self {
        Self { ptr, size }
    }

    pub const fn start(&self) -> *mut u8 {
        self.ptr
    }

    pub fn end(&self) -> *mut u8 {
        self.start()
            .wrapping_offset(self.size().try_into().unwrap())
    }

    pub const fn size(&self) -> usize {
        self.size
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

    pub const fn bounds(&self) -> StaticHeapBounds {
        StaticHeapBounds::new(self.space.get().cast(), N)
    }
}

impl<const N: usize, A> Default for StaticHeap<N, A> {
    fn default() -> Self {
        Self::new()
    }
}
