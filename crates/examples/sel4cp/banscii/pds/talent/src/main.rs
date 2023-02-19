#![no_std]
#![no_main]

use core::ptr;
use core::slice;

use volatile::{access::ReadOnly, Volatile};

use sel4cp::{main, NullHandler};

const REGION_SIZE: usize = 0x4_000;

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut region_in_start: *mut u8 = ptr::null_mut();

#[used]
#[no_mangle]
#[link_section = ".data"]
static mut region_out_start: *mut u8 = ptr::null_mut();

fn get_region_in() -> Volatile<&'static mut [u8], ReadOnly> {
    Volatile::new_read_only(unsafe { slice::from_raw_parts_mut(region_in_start, REGION_SIZE) })
}

fn get_region_out() -> Volatile<&'static mut [u8]> {
    Volatile::new(unsafe { slice::from_raw_parts_mut(region_out_start, REGION_SIZE) })
}

#[main(heap_size = 0x10000)]
fn main() -> NullHandler {
    unsafe {
        assert!(!region_in_start.is_null());
        assert!(!region_out_start.is_null());
    }
    NullHandler::new()
}
