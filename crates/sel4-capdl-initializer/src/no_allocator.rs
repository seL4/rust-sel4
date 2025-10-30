//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static A: NoAllocator = NoAllocator;

struct NoAllocator;

unsafe impl GlobalAlloc for NoAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unsafe { sel4_capdl_initializer__no_allocator_nonexistent() }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unsafe { sel4_capdl_initializer__no_allocator_nonexistent() }
    }
}

unsafe extern "Rust" {
    fn sel4_capdl_initializer__no_allocator_nonexistent() -> !;
}
