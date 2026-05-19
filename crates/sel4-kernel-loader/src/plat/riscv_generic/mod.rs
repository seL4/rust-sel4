//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::plat::Plat;

unsafe extern "C" {
    pub(crate) fn secondary_harts(sp: usize) -> !;
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init() {}

    fn put_char(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    fn start_core(physical_core_id: usize, sp: usize) {
        unsafe {
            sbi::hart_state_management::hart_start(
                physical_core_id,
                sbi::PhysicalAddress::new(secondary_harts as *const () as usize),
                sp,
            )
        }
        .unwrap()
    }

    fn stop_core() -> ! {
        match sbi::hart_state_management::hart_stop().unwrap() {}
    }
}
