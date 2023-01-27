#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use core::str;

use sel4cp::{main, NullHandler};

fn write(s: &[u8]) {
    sel4::debug_print!("{}", str::from_utf8(s).unwrap())
}

#[main]
fn main() -> NullHandler {
    sel4::debug_println!("assistant");
    let subject = "Hello";
    banscii_assistant_core::draft(subject, write);
    NullHandler::new()
}
