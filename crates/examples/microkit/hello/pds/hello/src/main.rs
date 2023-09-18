#![no_std]
#![no_main]

use sel4_microkit::{debug_println, protection_domain, Handler};

#[protection_domain]
fn init() -> HandlerImpl {
    debug_println!("Hello, World!");
    HandlerImpl
}

struct HandlerImpl;

impl Handler for HandlerImpl {}
