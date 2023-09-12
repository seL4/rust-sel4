use crate::arch::reset_cntvoff;

pub(crate) use crate::arch::drivers::psci::start_secondary_core;

pub(crate) fn init_platform_state_per_core(_core_id: usize) {
    unsafe {
        reset_cntvoff();
    }
}

pub(crate) mod debug {
    use spin::Mutex;

    use crate::drivers::pl011::Pl011Device;

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
}
