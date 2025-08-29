//
// Copyright 2023, Colias Group, LLC
// Copyright 2025, UNSW
//
// SPDX-License-Identifier: BSD-2-Clause
//

use spin::Mutex;

use sel4_config::sel4_cfg_bool;
use sel4_meson_uart_driver::Driver as MesonDriver;
use crate::{
    arch::{drivers::psci, reset_cntvoff},
    plat::Plat,
};

const SERIAL_DEVICE_BASE_ADDR: usize =  0xc81004c0;

static SERIAL_DEVICE: Mutex<MesonDriver> = Mutex::new(get_serial_device());

const fn get_serial_device() -> MesonDriver {
    unsafe { MesonDriver::new(SERIAL_DEVICE_BASE_ADDR) }
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init() {
        SERIAL_DEVICE.lock().init();
    }

    fn init_per_core() {
        if sel4_cfg_bool!(ARM_HYPERVISOR_SUPPORT) {
            unsafe {
                reset_cntvoff();
            }
        }
    }

    fn put_char(c: u8) {
        SERIAL_DEVICE.lock().put_char(c);
    }

    fn put_char_without_synchronization(c: u8) {
        get_serial_device().put_char(c);
    }

    // TODO: fix to use : https://github.com/au-ts/rust-sel4/blob/main/crates/sel4-kernel-loader/src/plat/bcm2711/mod.rs#L58
    fn start_secondary_core(core_id: usize, sp: usize) {
        psci::start_secondary_core(core_id, sp)
    }
}
