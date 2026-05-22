//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_: &PanicInfo) -> ! {
    __sel4_no_panic__undefined()
}

unsafe extern "Rust" {
    safe fn __sel4_no_panic__undefined() -> !;
}
