//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(core_intrinsics)]
#![allow(internal_features)]

use sel4_panicking_env::{debug_println, register_abort_trap};

pub fn indicate_success() -> ! {
    debug_println!("INDICATE_SUCCESS\x06");
    debug_println!("sentinel fallthrough");
    core::intrinsics::abort()
}

pub fn indicate_failure() -> ! {
    debug_println!("INDICATE_FAILURE\x15");
    debug_println!("sentinel fallthrough");
    core::intrinsics::abort()
}

register_abort_trap! {
    indicate_failure
}
