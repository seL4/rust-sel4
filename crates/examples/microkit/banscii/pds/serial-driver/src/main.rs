//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{memory_region_symbol, protection_domain, Channel};
use sel4_microkit_embedded_hal_adapters::serial::driver::Driver;

#[cfg(feature = "board-qemu_virt_aarch64")]
use sel4_pl011_driver::Driver as DriverImpl;

const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> Driver<DriverImpl> {
    let driver_impl =
        unsafe { DriverImpl::new(memory_region_symbol!(serial_register_block: *mut ()).as_ptr()) };
    Driver::new(driver_impl, DEVICE, ASSISTANT)
}
