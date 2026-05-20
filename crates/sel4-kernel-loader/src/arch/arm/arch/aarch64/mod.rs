//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use aarch64_cpu::asm::wfe;
use aarch64_cpu::registers::{CNTVOFF_EL2, CurrentEL, MPIDR_EL1, Readable, TPIDR_EL1, Writeable};

use crate::{arch::Arch, main, secondary_main};

pub(crate) mod drivers;
pub(crate) mod exception_handler;

unsafe extern "C" {
    pub(crate) fn secondary_entry() -> !;
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

fn get_physical_core_id() -> usize {
    MPIDR_EL1.read(MPIDR_EL1::Aff0).try_into().unwrap()
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    fn idle() -> ! {
        loop {
            wfe()
        }
    }

    fn prepare_to_enter_kernel(core_id: usize) {
        let current_el = CurrentEL.read_as_enum(CurrentEL::EL);
        assert!(current_el == Some(CurrentEL::EL::Value::EL2));

        TPIDR_EL1.set(core_id.try_into().unwrap());

        unsafe {
            switch_translation_tables_el2();
        }
    }
}

#[inline(never)]
pub(crate) unsafe fn reset_cntvoff() {
    CNTVOFF_EL2.set(0)
}
