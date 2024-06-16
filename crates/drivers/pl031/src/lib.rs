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
    pub unsafe fn new(ptr: *mut ()) -> Self {
        let mut this = Self {
            device: Device::new(ptr),
        };
        this.init();
        this
    }

    fn init(&mut self) {}

    pub fn now(&mut self) -> NaiveDateTime {
        DateTime::from_timestamp(self.device.get_data().into(), 0)
            .unwrap()
            .naive_utc()
    }
}

impl DateTimeAccess for Driver {
    type Error = Error;

    fn datetime(&mut self) -> Result<NaiveDateTime, Self::Error> {
        Ok(self.now())
    }

    fn set_datetime(&mut self, _datetime: &NaiveDateTime) -> Result<(), Self::Error> {
        Err(Error::UnsupportedOperation)
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    UnsupportedOperation,
}
