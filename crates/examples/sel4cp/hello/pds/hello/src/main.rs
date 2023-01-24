#![no_std]
#![no_main]
#![feature(never_type)]

use sel4cp::{main, NullHandler};

#[main]
fn main() -> NullHandler {
    sel4::debug_println!("Hello, World!");
    NullHandler::new()
}
