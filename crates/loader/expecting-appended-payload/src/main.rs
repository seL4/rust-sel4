#![no_std]
#![no_main]
#![feature(const_pointer_byte_offsets)]
#![feature(pointer_byte_offsets)]
#![feature(strict_provenance)]

use core::ops::Range;
use core::ptr;
use core::slice;

use loader_payload_types::*;

#[no_mangle]
extern "C" fn main() -> ! {
    loader_core::main(get_payload, &user_image_bounds())
}

mod translation_tables {
    include!(concat!(env!("OUT_DIR"), "/translation_tables.rs"));
}

fn get_payload() -> (PayloadForX, &'static [u8]) {
    let blob = unsafe { slice::from_raw_parts(loader_payload_start, loader_payload_size) };
    let (payload, source) = postcard::take_from_bytes(blob).unwrap();
    (payload, source)
}

#[no_mangle]
#[link_section = ".data"]
static mut loader_payload_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut loader_payload_size: usize = 0;

#[no_mangle]
#[link_section = ".data"]
static mut loader_image_start: *mut u8 = ptr::null_mut();

#[no_mangle]
#[link_section = ".data"]
static mut loader_image_end: *mut u8 = ptr::null_mut();

fn user_image_bounds() -> Range<usize> {
    unsafe { loader_image_start.expose_addr()..loader_image_end.expose_addr() }
}
