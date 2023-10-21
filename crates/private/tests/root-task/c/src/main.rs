//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::ffi::{c_char, CStr};

use sel4_newlib as _;
use sel4_root_task::{debug_println, root_task};

extern "C" {
    fn test(s: *const c_char) -> i32;
}

#[root_task]
fn main(_: &sel4::BootInfo) -> ! {
    let s = CStr::from_bytes_with_nul(b"1234\0").unwrap();
    let n = unsafe { test(s.as_ptr()) };
    debug_println!("n = {}", n);
    assert_eq!(n, 1234 + 234);
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
