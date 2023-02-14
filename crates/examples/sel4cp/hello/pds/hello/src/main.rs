#![no_std]
#![no_main]

use sel4cp::{debug_println, main, NullHandler};

#[main]
fn main() -> NullHandler {
    debug_println!("Hello, World!");
    NullHandler::new()
}
