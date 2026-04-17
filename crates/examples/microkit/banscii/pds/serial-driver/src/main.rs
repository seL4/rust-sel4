//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{Channel, Handler, memory_region_symbol, protection_domain, var};
use sel4_microkit_driver_adapters::serial::driver::HandlerImpl;

#[cfg(feature = "board-qemu_virt_aarch64")]
use sel4_pl011_driver::Driver;

#[protection_domain]
fn init() -> impl Handler {
    let driver =
        unsafe { Driver::new(memory_region_symbol!(serial_register_block: *mut ()).as_ptr()) };
    let device = Channel::new(*var!(serial_irq_id: usize = usize::MAX));
    let assistant = Channel::new(*var!(assistant_channel_id: usize = usize::MAX));
    HandlerImpl::new(driver, device, assistant)
}
