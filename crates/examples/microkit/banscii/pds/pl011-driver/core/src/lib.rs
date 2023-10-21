//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

mod device;

use device::Device;

pub struct Driver {
    device: Device,
}

impl Driver {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(ptr: *mut ()) -> Self {
        let mut this = Self {
            device: Device::new(ptr.cast()),
        };
        this.init();
        this
    }

    fn init(&mut self) {
        self.device.init();
    }

    pub fn put_char(&self, c: u8) {
        self.device.put_char(c)
    }

    pub fn get_char(&self) -> Option<u8> {
        self.device.get_char()
    }

    pub fn handle_interrupt(&self) {
        self.device.clear_all_interrupts()
    }
}
