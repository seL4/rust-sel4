//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_driver_interfaces::timer::SingleTimer;
use sel4_microkit::{memory_region_symbol, protection_domain, Channel, Handler};
use sel4_microkit_driver_adapters::timer::driver::Driver;
use sel4_sp804_driver::Driver as DriverImpl;

const DEVICE: Channel = Channel::new(0);
const CLIENT: Channel = Channel::new(1);

const FREQ: u64 = 1_000_000;

#[protection_domain]
fn init() -> impl Handler {
    let driver_impl = unsafe {
        DriverImpl::new(
            memory_region_symbol!(sp804_mmio_vaddr: *mut ()).as_ptr(),
            FREQ,
        )
    };
    Driver::new(SingleTimer(driver_impl), DEVICE, CLIENT).unwrap()
}
