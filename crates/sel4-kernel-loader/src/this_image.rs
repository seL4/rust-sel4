use core::ops::Range;
use core::ptr;
use core::slice;

use sel4_kernel_loader_payload_types::*;

pub(crate) fn get_payload() -> (PayloadForX, &'static [u8]) {
    let blob = unsafe { slice::from_raw_parts(loader_payload_start, loader_payload_size) };
    let (payload, source) = postcard::take_from_bytes(blob).unwrap();
    (payload, source)
}

pub(crate) fn get_user_image_bounds() -> Range<usize> {
    unsafe { loader_image_start.expose_addr()..loader_image_end.expose_addr() }
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
