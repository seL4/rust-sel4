//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use sel4_reset::reset1;
use sel4_root_task::{debug_println, root_task};

const INIT: usize = 1337;

static mut NOT_PERSISTENT: usize = INIT;

#[unsafe(link_section = ".persistent")]
static mut PERSISTENT: usize = INIT;

#[unsafe(link_section = ".persistent")]
static mut RESET_COUNT: usize = 0;

#[root_task]
fn main(bootinfo: &sel4::BootInfoPtr) -> ! {
    unsafe {
        debug_println!("NOT_PERSISTENT: {NOT_PERSISTENT}");
        debug_println!("PERSISTENT: {PERSISTENT}");
        debug_println!("RESET_COUNT: {RESET_COUNT}");

        if RESET_COUNT == 3 {
            assert_eq!(NOT_PERSISTENT, INIT);
            assert_eq!(PERSISTENT, INIT + RESET_COUNT);
            sel4_test_root_task::indicate_success()
        }

        NOT_PERSISTENT += 1;
        PERSISTENT += 1;
        RESET_COUNT += 1;
    }
    reset1(bootinfo.ptr() as usize)
}
