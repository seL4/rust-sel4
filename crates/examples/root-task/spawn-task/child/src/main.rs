//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

mod runtime;

fn main() -> ! {
    sel4::debug_println!("In child task");

    sel4::cap::Notification::from_bits(1).signal();

    sel4::cap::Tcb::from_bits(2).tcb_suspend().unwrap();

    unreachable!()
}
