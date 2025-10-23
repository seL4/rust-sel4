//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use sel4_microkit::{NullHandler, debug_println, protection_domain};
use sel4_reset::reset;

const INIT: usize = 1337;

static mut NOT_PERSISTENT: usize = INIT;

#[unsafe(link_section = ".persistent")]
static mut PERSISTENT: usize = INIT;

#[unsafe(link_section = ".persistent")]
static mut RESET_COUNT: usize = 0;

#[protection_domain]
fn init() -> NullHandler {
    unsafe {
        debug_println!("NOT_PERSISTENT: {NOT_PERSISTENT}");
        debug_println!("PERSISTENT: {PERSISTENT}");
        debug_println!("RESET_COUNT: {RESET_COUNT}");

        if RESET_COUNT == 3 {
            assert_eq!(NOT_PERSISTENT, INIT);
            assert_eq!(PERSISTENT, INIT + RESET_COUNT);
            debug_println!("TEST_PASS");
            return NullHandler::new();
        }

        NOT_PERSISTENT += 1;
        PERSISTENT += 1;
        RESET_COUNT += 1;
    }
    reset()
}
