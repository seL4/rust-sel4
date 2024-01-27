//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;

use volatile::ops::{BulkOps, Ops, UnitaryOps};

#[derive(Default, Copy, Clone)]
pub struct NormalOps(());

impl Ops for NormalOps {}

impl<T> UnitaryOps<T> for NormalOps {
    unsafe fn read(src: *const T) -> T {
        unsafe { ptr::read(src) }
    }

    unsafe fn write(dst: *mut T, src: T) {
        unsafe { ptr::write(dst, src) }
    }
}

impl<T> BulkOps<T> for NormalOps {
    unsafe fn memmove(dst: *mut T, src: *const T, count: usize) {
        unsafe { ptr::copy(src, dst, count) }
    }

    unsafe fn memcpy(dst: *mut T, src: *const T, count: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, count) }
    }

    unsafe fn memset(dst: *mut T, val: u8, count: usize) {
        unsafe { ptr::write_bytes(dst, val, count) }
    }
}
