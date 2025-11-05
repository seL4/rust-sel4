//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_int;
use core::panic::UnwindSafe;

use sel4_panicking_env::abort_without_info;

pub(crate) fn begin_panic() -> c_int {
    abort_without_info()
}

#[allow(clippy::result_unit_err)]
pub fn catch_unwind<R, F: FnOnce() -> R + UnwindSafe>(f: F) -> Result<R, ()> {
    Ok(f())
}
