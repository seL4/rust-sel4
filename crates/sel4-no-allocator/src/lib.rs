//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static A: NoAllocator = NoAllocator;

struct NoAllocator;

unsafe impl GlobalAlloc for NoAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        __sel4_no_allocator__undefined()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        __sel4_no_allocator__undefined()
    }
}

unsafe extern "Rust" {
    safe fn __sel4_no_allocator__undefined() -> !;
}
