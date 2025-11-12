//
// Copyright 2023, Colias Group, LLC
// Copyright 2023, Rust project contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//

use core::hint::black_box;

#[inline(never)]
pub(crate) fn __rust_begin_short_backtrace<T, F: FnOnce() -> T>(f: F) -> T {
    let result = f();

    // prevent this frame from being tail-call optimised away
    black_box(result)
}
