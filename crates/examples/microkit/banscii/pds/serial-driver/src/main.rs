//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{memory_region_symbol, protection_domain, Channel, Handler};
use sel4_microkit_driver_adapters::serial::driver::HandlerImpl;

#[cfg(feature = "board-qemu_virt_aarch64")]
use sel4_pl011_driver::Driver;

const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> impl Handler {
    let driver =
        unsafe { Driver::new(memory_region_symbol!(serial_register_block: *mut ()).as_ptr()) };
    HandlerImpl::new(driver, DEVICE, ASSISTANT)
}
