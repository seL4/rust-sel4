use spin::Mutex;

use crate::pl011::Pl011Device;

const PL011_BASE: usize = 0x0900_0000;

static PL011: Mutex<Pl011Device> = Mutex::new(unsafe { Pl011Device::new(PL011_BASE) });

pub(crate) fn init() {
    PL011.lock().init();
}

pub(crate) fn put_char(c: u8) {
    PL011.lock().put_char(c);
}
