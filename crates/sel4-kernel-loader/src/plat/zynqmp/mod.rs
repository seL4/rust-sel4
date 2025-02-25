use embedded_hal_nb::nb;
use embedded_hal_nb::serial::Write;
use spin::lock_api::Mutex;

use sel4_zynqmp_xuartps_driver::Driver as ZynqmpXuartpsDriver;
use sel4_config::{sel4_cfg, sel4_cfg_bool};

use crate::{
    arch::{drivers::psci, reset_cntvoff},
    plat::Plat,
};

const SERIAL_DEVICE_BASE_ADDR: usize = 0x00FF000000;
static SERIAL_DRIVER: Mutex<ZynqmpXuartpsDriver> = Mutex::new(get_serial_driver());

const fn get_serial_driver() -> ZynqmpXuartpsDriver {
    unsafe { ZynqmpXuartpsDriver::new_uninit(SERIAL_DEVICE_BASE_ADDR as *mut _) }
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init() {}

    fn init_per_core() {
        if sel4_cfg_bool!(ARM_HYPERVISOR_SUPPORT) {
            unsafe {
                reset_cntvoff();
            }
        }
    }

    fn put_char(c: u8) {
        nb::block!(SERIAL_DRIVER.lock().write(c)).unwrap_or_else(|err| match err {});
    }

    fn put_char_without_synchronization(c: u8) {
        nb::block!(get_serial_driver().write(c)).unwrap_or_else(|err| match err {});
    }


    fn start_secondary_core(core_id: usize, sp: usize) {
        psci::start_secondary_core(core_id, sp)
    }
}
