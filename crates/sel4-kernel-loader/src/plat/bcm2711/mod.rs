use crate::arch::drivers::spin_table;
use crate::arch::reset_cntvoff;

pub(crate) fn init_platform_state_per_core(_core_id: usize) {
    unsafe {
        reset_cntvoff();
    }
}

const SPIN_TABLE: &[usize] = &[0xd8, 0xe0, 0xe8, 0xf0];

pub(crate) fn start_secondary_core(core_id: usize, sp: usize) {
    spin_table::start_secondary_core(SPIN_TABLE, core_id, sp)
}

pub(crate) mod debug {
    use spin::Mutex;

    use crate::drivers::bcm2835_aux_uart::Bcm2835AuxUartDevice;

    const BASE_ADDR: usize = 0xfe21_5000;

    static DEVICE: Mutex<Bcm2835AuxUartDevice> = Mutex::new(get_device());

    const fn get_device() -> Bcm2835AuxUartDevice {
        unsafe { Bcm2835AuxUartDevice::new(BASE_ADDR) }
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
