//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(never_type)]

use sel4_microkit::{debug_println, protection_domain, Handler};

#[protection_domain]
fn init() -> HandlerImpl {
    debug_println!("Hello, World!");
    HandlerImpl
}

struct HandlerImpl;

impl Handler for HandlerImpl {
    type Error = !;
}
