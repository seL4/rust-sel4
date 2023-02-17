#![no_std]
#![no_main]

use sel4cp::{main, NullHandler};

#[main(heap_size = 0x10000)]
fn main() -> NullHandler {
    NullHandler::new()
}
