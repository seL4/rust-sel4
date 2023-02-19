#![no_std]

use core::ptr;
use core::slice;

use sel4_backtrace_addr2line_context_helper::{new_context, Context, Error};

#[no_mangle]
#[link_section = ".data"]
static mut embedded_debug_info_start: *const u8 = ptr::null();

#[no_mangle]
#[link_section = ".data"]
static mut embedded_debug_info_size: usize = 0;

pub fn get_context() -> Result<Context, Error> {
    let embedded_debug_info =
        unsafe { slice::from_raw_parts(embedded_debug_info_start, embedded_debug_info_size) };
    let obj = object::File::parse(embedded_debug_info).unwrap();
    new_context(&obj)
}
