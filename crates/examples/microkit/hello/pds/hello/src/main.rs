//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Handler, Infallible, debug_println, protection_domain};

#[protection_domain]
fn init() -> HandlerImpl {
    debug_println!("Hello, World!");
    HandlerImpl
}

struct HandlerImpl;

impl Handler for HandlerImpl {
    type Error = Infallible;
}
