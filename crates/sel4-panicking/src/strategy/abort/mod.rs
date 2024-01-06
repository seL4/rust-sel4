//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_panicking_env::abort_without_info;

#[cfg(panic = "unwind")]
use sel4_panicking_env::abort;

use crate::Payload;

pub(crate) fn panic_cleanup(_exception: *mut u8) -> Payload {
    unreachable!()
}

pub(crate) fn start_panic(_payload: Payload) -> i32 {
    abort_without_info()
}

#[cfg(panic = "unwind")]
#[lang = "eh_personality"]
fn eh_personality() -> ! {
    abort!("unexpected call to eh_personality")
}
