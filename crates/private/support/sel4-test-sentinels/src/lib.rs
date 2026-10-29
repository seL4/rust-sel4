//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use sel4_crash_polyfill::crash;
use sel4_panicking_env::{debug_println, register_abort_trap};

pub fn indicate_success() -> ! {
    debug_println!("INDICATE_SUCCESS\x06");
    debug_println!("sentinel fallthrough");
    crash()
}

pub fn indicate_failure() -> ! {
    debug_println!("INDICATE_FAILURE\x15");
    debug_println!("sentinel fallthrough");
    crash()
}

register_abort_trap! {
    indicate_failure
}
