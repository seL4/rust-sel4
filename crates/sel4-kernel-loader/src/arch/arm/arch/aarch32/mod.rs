//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use aarch32_cpu::asm::wfe;
use aarch32_cpu::register::{CntVoff, Cpsr, Mpidr, cpsr::ProcessorMode};

use crate::{arch::Arch, main, secondary_main};

pub(crate) mod drivers;

#[unsafe(no_mangle)]
extern "C" fn arch_main(dtb: usize) -> ! {
    let physical_core_id = get_physical_core_id();
    assert_eq!(physical_core_id, 0);
    main(physical_core_id, dtb)
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main() -> ! {
    let physical_core_id = get_physical_core_id();
    secondary_main(physical_core_id)
}

fn get_physical_core_id() -> usize {
    (Mpidr::read().0 & 0xff).try_into().unwrap()
}

unsafe extern "C" {
    fn switch_translation_tables();
}

pub(crate) enum ArchImpl {}

impl Arch for ArchImpl {
    fn idle() -> ! {
        loop {
            wfe()
        }
    }

    fn prepare_to_enter_kernel(_core_id: usize) {
        unsafe {
            switch_translation_tables();
        }
    }
}

#[inline(never)]
pub(crate) unsafe fn reset_cntvoff() {
    CntVoff::write(CntVoff(0))
}

const CPSR_MODE_MASK: usize = 0x1f;
const CPSR_MODE_HYPERVISOR: usize = 0x1a;

fn is_hyp_mode() -> bool {
    Cpsr::read().mode() == Ok(ProcessorMode::Hyp)
}

unsafe extern "C" {
    pub(crate) fn secondary_entry() -> !;
}
