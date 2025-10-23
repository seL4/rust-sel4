//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use sel4_driver_interfaces::timer::SingleTimer;
use sel4_microkit::{Channel, Handler, memory_region_symbol, protection_domain};
use sel4_microkit_driver_adapters::timer::driver::HandlerImpl;
use sel4_sp804_driver::Driver;

const DEVICE: Channel = Channel::new(0);
const CLIENT: Channel = Channel::new(1);

const FREQ: u64 = 1_000_000;

#[protection_domain]
fn init() -> impl Handler {
    let driver = unsafe {
        Driver::new(
            memory_region_symbol!(sp804_mmio_vaddr: *mut ()).as_ptr(),
            FREQ,
        )
    };
    HandlerImpl::new(SingleTimer(driver), DEVICE, CLIENT).unwrap()
}
