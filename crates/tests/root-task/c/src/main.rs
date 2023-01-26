#![no_std]
#![no_main]

use core::ffi::{c_char, CStr};

use sel4_full_root_task_runtime::{debug_println, main};

extern "C" {
    fn test(s: *const c_char) -> i32;
}

#[main]
fn main(_: &sel4::BootInfo) -> ! {
    let s = CStr::from_bytes_with_nul(b"1234\0").unwrap();
    let n = unsafe { test(s.as_ptr()) };
    debug_println!("n = {}", n);
    assert_eq!(n, 1234 + 234);
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
