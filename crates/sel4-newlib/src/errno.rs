//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_int;

#[cfg(feature = "errno")]
#[no_mangle]
static mut errno: c_int = 0;

#[cfg(not(feature = "errno"))]
extern "C" {
    static mut errno: c_int;
}

pub(crate) fn set_errno(err: c_int) {
    unsafe {
        errno = err;
    }
}

pub(crate) mod values {
    use super::*;

    pub(crate) const ENOENT: c_int = 2;
    pub(crate) const ENOMEM: c_int = 12;
}
