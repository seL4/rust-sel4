//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;
use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use sel4_config::sel4_cfg_usize;

use crate::plat::Plat;

#[no_mangle]
static mut hsm_exists: i32 = 0;

#[no_mangle]
static mut next_logical_core_id: i32 = 1;

#[no_mangle]
static mut start_core_by_logical_id: i32 = 0;

#[no_mangle]
static mut secondary_core_sp: usize = 0;

extern "C" {
    pub(crate) fn secondary_harts();
}

pub(crate) enum PlatImpl {}

impl Plat for PlatImpl {
    fn init() {
        assert!(get_hsm_exists());
        start_all_harts();
    }

    fn put_char(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    fn put_char_without_synchronization(c: u8) {
        sbi::legacy::console_putchar(c)
    }

    fn start_secondary_core(core_id: usize, sp: usize) {
        unsafe {
            AtomicUsize::from_ptr(ptr::addr_of_mut!(secondary_core_sp)).store(sp, Ordering::SeqCst);
            AtomicI32::from_ptr(ptr::addr_of_mut!(start_core_by_logical_id))
                .store(core_id.try_into().unwrap(), Ordering::SeqCst);
        }
    }
}

fn get_hsm_exists() -> bool {
    unsafe { hsm_exists != 0 }
}

fn start_all_harts() {
    for i in 0..sel4_cfg_usize!(MAX_NUM_NODES) {
        if i != sel4_cfg_usize!(FIRST_HART_ID) {
            let _ = sbi::hart_state_management::hart_start(i, secondary_harts as usize, i);
        }
    }
}
