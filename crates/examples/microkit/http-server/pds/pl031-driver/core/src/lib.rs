//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::time::Duration;

mod device;

use device::Device;

pub struct Driver {
    device: Device,
}

impl Driver {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(ptr: *mut ()) -> Self {
        let mut this = Self {
            device: Device::new(ptr),
        };
        this.init();
        this
    }

    fn init(&mut self) {}

    pub fn now(&mut self) -> Duration {
        Duration::from_secs(self.device.get_data().into())
    }
}
