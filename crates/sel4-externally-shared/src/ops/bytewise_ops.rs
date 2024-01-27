//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::marker::PhantomData;
use core::mem;

use volatile::ops::{BulkOps, Ops, UnitaryOps};
use zerocopy::{AsBytes, FromBytes};

#[derive(Default, Copy, Clone)]
pub struct BytewiseOps<O> {
    _phantom: PhantomData<O>,
}

impl<O: Ops> Ops for BytewiseOps<O> {}

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
