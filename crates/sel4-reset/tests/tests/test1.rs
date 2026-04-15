//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use sel4_minimal_linux_runtime::*;
use sel4_reset::reset;

const INIT: usize = 1337;

static mut NOT_PERSISTENT: usize = INIT;

#[unsafe(link_section = ".persistent")]
static mut PERSISTENT: usize = INIT;

#[unsafe(link_section = ".persistent")]
static mut RESET_COUNT: usize = 0;

#[main]
fn main() {
    unsafe {
        debug_println!("NOT_PERSISTENT: {NOT_PERSISTENT}");
        debug_println!("PERSISTENT: {PERSISTENT}");
        debug_println!("RESET_COUNT: {RESET_COUNT}");

        if RESET_COUNT == 3 {
            assert_eq!(NOT_PERSISTENT, INIT);
            assert_eq!(PERSISTENT, INIT + RESET_COUNT);
            exit_success()
        }

        NOT_PERSISTENT += 1;
        PERSISTENT += 1;
        RESET_COUNT += 1;
    }
    reset()
}
