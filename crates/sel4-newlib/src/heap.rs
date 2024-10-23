//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::{c_int, c_void};
use core::sync::atomic::{AtomicIsize, Ordering};

use sel4_panicking_env::abort;
use sel4_static_heap::StaticHeap;

use crate::errno;

#[repr(align(4096))] // no real reason for this
struct Align;

#[doc(hidden)]
pub struct StaticHeapWithWatermark<const N: usize> {
    memory: StaticHeap<N, Align>,
    watermark: AtomicIsize,
}

impl<const N: usize> StaticHeapWithWatermark<N> {
    pub const fn new() -> Self {
        Self {
            memory: StaticHeap::new(),
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
        if new > self.memory.size().try_into().unwrap_or_else(|_| abort!()) {
            self.watermark.fetch_sub(incr, Ordering::SeqCst);
            errno::set_errno(errno::values::ENOMEM);
            return usize::MAX as *mut c_void;
        }
        self.memory.start().wrapping_offset(old).cast::<c_void>()
    }
}

impl<const N: usize> Default for StaticHeapWithWatermark<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! declare_sbrk_with_static_heap {
    ($n:expr) => {
        #[no_mangle]
        extern "C" fn _sbrk(incr: core::ffi::c_int) -> *mut core::ffi::c_void {
            static HEAP: $crate::StaticHeapWithWatermark<{ $n }> =
                $crate::StaticHeapWithWatermark::new();
            HEAP.sbrk(incr)
        }
    };
}
