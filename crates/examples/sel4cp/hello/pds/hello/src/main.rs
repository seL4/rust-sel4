#![no_std]
#![no_main]

use sel4cp::{main, NullHandler};

#[main]
fn main() -> NullHandler {
    sel4::debug_println!("Hello, World!");
    NullHandler::new()
}
