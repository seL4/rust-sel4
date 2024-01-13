//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![feature(thread_local)]

use core::hint::black_box;

use sel4_root_task::{debug_println, root_task};

const X: i32 = 1337;

#[repr(C, align(4096))]
struct T(i32);

#[no_mangle]
#[thread_local]
static Y: T = T(X);

#[root_task]
fn main(_: &sel4::BootInfo) -> ! {
    let observed = Y.0;
    debug_println!("{}", observed);
    assert_eq!(observed, black_box(X));
    debug_println!("TEST_PASS");
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
