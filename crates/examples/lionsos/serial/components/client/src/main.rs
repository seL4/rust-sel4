//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sddf::serial::*;
use sel4_microkit::{debug_println, protection_domain, Handler, Infallible};

#[protection_domain]
fn init() -> HandlerImpl {
    debug_println!("Hello, World!");

    let x = sddf::config!(".serial_client_config", config: ClientConfig);

    HandlerImpl
}

struct HandlerImpl;

impl Handler for HandlerImpl {
    type Error = Infallible;
}
