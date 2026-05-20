//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;

use aarch64_cpu::registers::{CurrentEL, MPIDR_EL1, Readable};

use crate::{arch::Arch, main, secondary_main};

pub(crate) mod drivers;
pub(crate) mod exception_handler;

unsafe extern "C" {
    fn switch_translation_tables_el2();
}

#[unsafe(no_mangle)]
extern "C" fn arch_main(dtb: usize) -> ! {
    let physical_core_id = get_physical_core_id();
    assert_eq!(physical_core_id, 0); // TODO Check in head.S like elfloader? On what platforms could this fail?
    main(physical_core_id, dtb)
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main() -> ! {
    let physical_core_id = get_physical_core_id();
    secondary_main(physical_core_id)
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    fn idle() -> ! {
        loop {
            unsafe {
                asm!("wfe");
            }
        }
    }

    fn prepare_to_enter_kernel(core_id: usize) {
        let current_el = get_current_el();
        assert!(current_el == Some(CurrentEL::EL::Value::EL2));

        unsafe {
            set_tpidr(core_id);
        }

        unsafe {
            switch_translation_tables_el2();
        }
    }
}

fn get_physical_core_id() -> usize {
    MPIDR_EL1.read(MPIDR_EL1::Aff0).try_into().unwrap()
}

fn get_current_el() -> Option<CurrentEL::EL::Value> {
    CurrentEL.read_as_enum(CurrentEL::EL)
}

#[inline(never)] // never inline to work around issues with optimizer
unsafe fn set_tpidr(tpidr: usize) {
    unsafe {
        asm!("msr tpidr_el1, {tpidr}", tpidr = in(reg) tpidr);
    }
}

#[inline(never)]
pub(crate) unsafe fn reset_cntvoff() {
    unsafe {
        asm!("msr cntvoff_el2, xzr");
    }
}

unsafe extern "C" {
    pub(crate) fn secondary_entry() -> !;
}
