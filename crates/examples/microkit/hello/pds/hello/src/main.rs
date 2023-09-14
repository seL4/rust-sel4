#![no_std]
#![no_main]

use sel4_microkit::{debug_println, protection_domain, NullHandler};

#[protection_domain]
fn init() -> NullHandler {
    debug_println!("Hello, World!");
    NullHandler::new()
}
