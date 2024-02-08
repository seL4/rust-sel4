//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_root_task::root_task;

#[root_task]
fn main(_bootinfo: &sel4::BootInfoPtr) -> ! {
    sel4::debug_println!("Hello, World!");

    sel4::init_thread::suspend_self()
}
