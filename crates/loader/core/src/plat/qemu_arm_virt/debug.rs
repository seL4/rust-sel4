use spin::Mutex;

use crate::pl011::Pl011Device;

const BASE_ADDR: usize = 0x0900_0000;

static DEVICE: Mutex<Pl011Device> = Mutex::new(get_device());

const fn get_device() -> Pl011Device {
    unsafe { Pl011Device::new(BASE_ADDR) }
}

pub(crate) fn init() {
    DEVICE.lock().init();
}

pub(crate) fn put_char(c: u8) {
    DEVICE.lock().put_char(c);
}

pub(crate) fn put_char_without_synchronization(c: u8) {
    get_device().put_char(c);
}
