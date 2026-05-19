//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::Write;

use sel4_config::sel4_cfg_bool;
use sel4_pl011_driver::Driver as Pl011Driver;

use crate::{
    arch::{drivers::psci, reset_cntvoff},
    plat::Plat,
};

const SERIAL_DEVICE_BASE_ADDR: usize = 0x0900_0000;

const fn get_serial_driver() -> Pl011Driver {
    unsafe { Pl011Driver::new_uninit(SERIAL_DEVICE_BASE_ADDR as *mut _) }
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init_per_core() {
        if sel4_cfg_bool!(ARM_HYPERVISOR_SUPPORT) {
            unsafe {
                reset_cntvoff();
            }
        }
    }

    fn put_char(c: u8) {
        nb::block!(get_serial_driver().write(c)).unwrap_or_else(|err| match err {});
    }

    fn start_core(physical_core_id: usize, sp: usize) {
        psci::start_core(physical_core_id, sp)
    }
}
