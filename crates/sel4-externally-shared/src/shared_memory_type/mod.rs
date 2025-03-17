//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::ptr;

use zerocopy::{FromBytes, IntoBytes};

use sel4_abstract_ptr::memory_type::*;

#[cfg(feature = "atomics")]
mod atomics;

pub struct SharedMemory(());

impl MemoryType for SharedMemory {}

impl<T: FromBytes + IntoBytes> UnitaryOps<T> for SharedMemory {
    unsafe fn read(src: *const T) -> T {
        unsafe { ptr::read(src) }
    }

    unsafe fn write(dst: *mut T, src: T) {
        unsafe { ptr::write(dst, src) }
    }
}

impl<T: FromBytes + IntoBytes> BulkOps<T> for SharedMemory {
    unsafe fn memmove(dst: *mut T, src: *const T, count: usize) {
        unsafe { ptr::copy(src, dst, count) }
    }

    unsafe fn memcpy_into(dst: *mut T, src: *const T, count: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, count) }
    }

    unsafe fn memcpy_from(dst: *mut T, src: *const T, count: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, count) }
    }

    unsafe fn memset(dst: *mut T, val: u8, count: usize) {
        unsafe { ptr::write_bytes(dst, val, count) }
    }
}
