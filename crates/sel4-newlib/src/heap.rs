//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::*;

use core::cell::SyncUnsafeCell;
use core::ffi::{c_int, c_void};
use core::ptr;
use core::sync::atomic::{AtomicIsize, Ordering};

use sel4_panicking_env::abort;

#[repr(align(4096))] // no real reason for this
struct BackingMemory<const N: usize>(SyncUnsafeCell<[u8; N]>);

impl<const N: usize> BackingMemory<N> {
    const fn new() -> Self {
        Self(SyncUnsafeCell::new([0; N]))
    }

    fn bounds(&self) -> *mut [u8] {
        ptr::slice_from_raw_parts_mut(self.0.get().cast(), N)
    }
}

#[doc(hidden)]
pub struct StaticHeap<const N: usize> {
    memory: BackingMemory<N>,
    watermark: AtomicIsize,
}

unsafe impl<const N: usize> Sync for StaticHeap<N> {}

impl<const N: usize> StaticHeap<N> {
    pub const fn new() -> Self {
        Self {
            memory: BackingMemory::new(),
            watermark: AtomicIsize::new(0),
        }
    }

    // TODO handle overflowing atomic
    pub fn sbrk(&self, incr: c_int) -> *mut c_void {
        #[cfg(feature = "log")]
        {
            log::trace!("_sbrk({})", incr);
        }
        let incr = incr.try_into().unwrap_or_else(|_| abort!());
        let old = self.watermark.fetch_add(incr, Ordering::SeqCst);
        let new = old + incr;
        if new < 0 {
            abort!("program break below data segment start")
        }
        if new > N.try_into().unwrap_or_else(|_| abort!()) {
            self.watermark.fetch_sub(incr, Ordering::SeqCst);
            errno::set_errno(errno::values::ENOMEM);
            return usize::MAX as *mut c_void;
        }
        unsafe { self.memory.bounds().as_mut_ptr().offset(old).cast() }
    }
}

#[macro_export]
macro_rules! declare_sbrk_with_static_heap {
    ($n:expr) => {
        #[no_mangle]
        extern "C" fn _sbrk(incr: core::ffi::c_int) -> *mut core::ffi::c_void {
            static HEAP: $crate::StaticHeap<{ $n }> = $crate::StaticHeap::new();
            HEAP.sbrk(incr)
        }
    };
}
