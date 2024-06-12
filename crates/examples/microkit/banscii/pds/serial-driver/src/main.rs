//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_microkit::{memory_region_symbol, protection_domain, Channel};
use sel4_microkit_embedded_hal_adapters::serial::driver::Driver;
use sel4_pl011_driver::Driver as DriverImpl;

const DEVICE: Channel = Channel::new(0);
const ASSISTANT: Channel = Channel::new(1);

#[protection_domain]
fn init() -> Driver<DriverImpl> {
    let mut driver_impl =
        unsafe { DriverImpl::new(memory_region_symbol!(pl011_register_block: *mut ()).as_ptr()) };
    driver_impl.init();
    Driver::new(driver_impl, DEVICE, ASSISTANT)
}
