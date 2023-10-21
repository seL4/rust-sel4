//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use spin::Mutex;

use crate::{
    arch::{drivers::spin_table, reset_cntvoff},
    drivers::bcm2835_aux_uart::Bcm2835AuxUartDevice,
    plat::Plat,
};

const SPIN_TABLE: &[usize] = &[0xd8, 0xe0, 0xe8, 0xf0];

const SERIAL_DEVICE_BASE_ADDR: usize = 0xfe21_5000;

static SERIAL_DEVICE: Mutex<Bcm2835AuxUartDevice> = Mutex::new(get_serial_device());

const fn get_serial_device() -> Bcm2835AuxUartDevice {
    unsafe { Bcm2835AuxUartDevice::new(SERIAL_DEVICE_BASE_ADDR) }
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init() {
        SERIAL_DEVICE.lock().init();
    }

    fn init_per_core() {
        unsafe {
            reset_cntvoff();
        }
    }

    fn put_char(c: u8) {
        SERIAL_DEVICE.lock().put_char(c);
    }

    fn put_char_without_synchronization(c: u8) {
        get_serial_device().put_char(c);
    }

    fn start_secondary_core(core_id: usize, sp: usize) {
        spin_table::start_secondary_core(SPIN_TABLE, core_id, sp)
    }
}
