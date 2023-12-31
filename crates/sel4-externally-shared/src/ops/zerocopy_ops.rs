//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use volatile::ops::{Ops, UnitaryOps};
use zerocopy::{AsBytes, FromBytes};

#[cfg(feature = "unstable")]
use volatile::ops::BulkOps;

#[derive(Debug, Default, Copy, Clone)]
pub struct ZerocopyOps<O>(O);

impl<O: Ops> Ops for ZerocopyOps<O> {}

impl<O: UnitaryOps<T>, T: FromBytes + AsBytes> UnitaryOps<T> for ZerocopyOps<O> {
    unsafe fn read(src: *const T) -> T {
        unsafe { O::read(src) }
    }

    unsafe fn write(dst: *mut T, src: T) {
        unsafe { O::write(dst, src) }
    }
}

#[cfg(feature = "unstable")]
impl<O: BulkOps<T>, T: FromBytes + AsBytes> BulkOps<T> for ZerocopyOps<O> {
    unsafe fn memmove(dst: *mut T, src: *const T, count: usize) {
        unsafe { O::memmove(dst, src, count) }
    }

    unsafe fn memcpy(dst: *mut T, src: *const T, count: usize) {
        unsafe { O::memcpy(dst, src, count) }
    }

    unsafe fn memset(dst: *mut T, val: u8, count: usize) {
        unsafe { O::memset(dst, val, count) }
    }
}
