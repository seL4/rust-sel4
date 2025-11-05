//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_panicking_env::abort_without_info;

pub(crate) fn panic_cleanup(_exception: *mut u8) {
    unreachable!()
}

pub(crate) fn start_panic() -> i32 {
    abort_without_info()
}
