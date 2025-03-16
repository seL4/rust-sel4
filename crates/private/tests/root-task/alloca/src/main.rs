//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::alloc::Layout;

use sel4_alloca::*;
use sel4_root_task::{debug_println, root_task};

const X: usize = 1234;

#[root_task]
fn main(_bootinfo: &sel4::BootInfoPtr) -> ! {
    let x = with_alloca_ptr(Layout::from_size_align(8, 8).unwrap(), |_| {
        debug_println!("abc");
        X
    });

    assert_eq!(x, X);

    debug_println!("TEST_PASS");

    sel4::init_thread::suspend_self()
}
