#![no_std]

use core::convert::Infallible;

use embedded_hal_nb::nb;
use embedded_hal_nb::serial;

mod device;

use device::Device;

pub struct Driver {
    device: Device,
}

unsafe impl Send for Driver {}
unsafe impl Sync for Driver {}

impl Driver {
    #[allow(clippy::missing_safety_doc)]
    pub const unsafe fn new_uninit(ptr: *mut ()) -> Self {
        Self {
            device: Device::new(ptr.cast()),
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(ptr: *mut ()) -> Self {
        let mut this = Self::new_uninit(ptr);
        this.init();
        this
    }

    pub fn init(&mut self) {
        self.device.init();
    }
}

impl serial::ErrorType for Driver {
    type Error = Infallible;
}

impl serial::Write for Driver {
    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.device.put_char(word);
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        Ok(())
    }
}
