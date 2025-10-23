//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{NullHandler, debug_println, protection_domain};

#[protection_domain]
fn init() -> NullHandler {
    debug_println!("TEST_PASS");
    NullHandler::new()
}
