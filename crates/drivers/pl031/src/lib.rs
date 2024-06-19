//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use rtcc::{DateTime, DateTimeAccess, NaiveDateTime};

mod device;

use device::Device;

pub struct Driver {
    device: Device,
}

impl Driver {
    #[allow(clippy::missing_safety_doc)]
    pub const unsafe fn new_uninit(ptr: *mut ()) -> Self {
        Self {
            device: Device::new(ptr),
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(ptr: *mut ()) -> Self {
        let mut this = Self::new_uninit(ptr);
        this.init();
        this
    }

    pub fn init(&mut self) {}
}

impl DateTimeAccess for Driver {
    type Error = Error;

    fn datetime(&mut self) -> Result<NaiveDateTime, Self::Error> {
        Ok(DateTime::from_timestamp(self.device.get_data().into(), 0)
            .unwrap()
            .naive_utc())
    }

    fn set_datetime(&mut self, _datetime: &NaiveDateTime) -> Result<(), Self::Error> {
        Err(Error::UnsupportedOperation)
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    UnsupportedOperation,
}
