#![no_std]
#![no_main]

use sel4cp::{main, NullHandler};

#[main]
fn main() -> NullHandler {
    NullHandler::new()
}
