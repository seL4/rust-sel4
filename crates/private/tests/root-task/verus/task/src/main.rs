//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_root_task::{debug_println, root_task};

#[root_task(heap_size = 1024 * 1024)]
fn main(_: &sel4::BootInfoPtr) -> ! {
    assert_eq!(tests_root_task_verus_core::max(13, 37), 37);

    debug_println!("TEST_PASS");
    sel4::init_thread::suspend_self()
}
