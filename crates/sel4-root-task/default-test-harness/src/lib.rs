//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_root_task::root_task;
use sel4_test_harness::run_test_main;

pub use sel4_test_harness::for_generated_code::*;

const HEAP_SIZE: usize = 64 * 1024 * 1024;

#[root_task(heap_size = HEAP_SIZE)]
fn main(_bootinfo: &sel4::BootInfo) -> ! {
    run_test_main();
    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}
