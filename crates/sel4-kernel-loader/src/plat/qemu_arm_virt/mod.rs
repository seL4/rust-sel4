//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::Write;
use spin::lock_api::Mutex;

use sel4_config::sel4_cfg_bool;
use sel4_pl011_driver::Driver as Pl011Driver;

use crate::{
    arch::{drivers::psci, reset_cntvoff},
    plat::Plat,
};

const SERIAL_DEVICE_BASE_ADDR: usize = 0x0900_0000;

static SERIAL_DRIVER: Mutex<Pl011Driver> = Mutex::new(get_serial_driver());

const fn get_serial_driver() -> Pl011Driver {
    unsafe { Pl011Driver::new_uninit(SERIAL_DEVICE_BASE_ADDR as *mut _) }
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init() {
        SERIAL_DRIVER.lock().init();
    }

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
