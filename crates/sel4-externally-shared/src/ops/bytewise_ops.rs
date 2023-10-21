//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![cfg_attr(not(feature = "unstable"), allow(unused_imports))]

use core::mem;

use volatile::ops::{Ops, UnitaryOps};
use zerocopy::{AsBytes, FromBytes};

#[cfg(feature = "unstable")]
use volatile::ops::BulkOps;

#[derive(Debug, Default, Copy, Clone)]
pub struct BytewiseOps<O>(O);

impl<O: Ops> Ops for BytewiseOps<O> {}

#[cfg(feature = "unstable")]
impl<O: BulkOps<u8>, T: FromBytes + AsBytes> UnitaryOps<T> for BytewiseOps<O> {
    unsafe fn read(src: *const T) -> T {
        let mut val = T::new_zeroed();
        let view = val.as_bytes_mut();
        unsafe { O::memcpy(view.as_mut_ptr(), src.cast(), mem::size_of::<T>()) };
        val
    }

    unsafe fn write(dst: *mut T, src: T) {
        let view = src.as_bytes();
        unsafe { O::memcpy(dst.cast(), view.as_ptr(), mem::size_of::<T>()) };
    }
}

#[cfg(feature = "unstable")]
impl<O: BulkOps<u8>, T: FromBytes + AsBytes> BulkOps<T> for BytewiseOps<O> {
    unsafe fn memmove(dst: *mut T, src: *const T, count: usize) {
        unsafe { O::memmove(dst.cast(), src.cast(), count * mem::size_of::<T>()) }
    }

    unsafe fn memcpy(dst: *mut T, src: *const T, count: usize) {
        unsafe { O::memcpy(dst.cast(), src.cast(), count * mem::size_of::<T>()) }
    }

    unsafe fn memset(dst: *mut T, val: u8, count: usize) {
        unsafe { O::memset(dst.cast(), val, count * mem::size_of::<T>()) }
    }
}
