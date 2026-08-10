//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use embedded_hal_nb::nb;
use embedded_hal_nb::serial::Write;

use sel4_bcm2835_aux_uart_driver::Driver as Bcm2835AuxUartDriver;
use sel4_config::{sel4_cfg, sel4_cfg_bool};

use crate::{arch::reset_cntvoff, plat::Plat};

const SERIAL_DEVICE_BASE_ADDR: usize = 0xfe21_5000;

const fn get_serial_driver() -> Bcm2835AuxUartDriver {
    unsafe { Bcm2835AuxUartDriver::new_uninit(SERIAL_DEVICE_BASE_ADDR as *mut _) }
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

    #[sel4_cfg(ARCH_AARCH64)]
    fn start_core(physical_core_id: usize, sp: usize) {
        const SPIN_TABLE: &[usize] = &[0xd8, 0xe0, 0xe8, 0xf0];

        crate::arch::drivers::spin_table::start_core(SPIN_TABLE, physical_core_id, sp)
    }

    #[sel4_cfg(ARCH_AARCH32)]
    fn start_core(physical_core_id: usize, sp: usize) {
        const SPIN_TABLE: &[usize] = &[0xff80_008C, 0xff80_009C, 0xff80_00AC, 0xff80_00BC];

        crate::arch::drivers::spin_table::start_core(SPIN_TABLE, physical_core_id, sp)
    }
}
