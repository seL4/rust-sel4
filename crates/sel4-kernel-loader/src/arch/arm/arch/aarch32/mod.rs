//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;

use crate::{arch::Arch, enter_kernel::KernelEntryExtraArgs, main, secondary_main};

pub(crate) mod drivers;

#[unsafe(no_mangle)]
extern "C" fn arch_main() -> ! {
    main(KernelEntryExtraArgs {})
}

#[unsafe(no_mangle)]
extern "C" fn arch_secondary_main() -> ! {
    secondary_main(KernelEntryExtraArgs {})
}

unsafe extern "C" {
    fn switch_translation_tables();
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

    fn prepare_to_enter_kernel(_core_id: usize) {
        unsafe {
            switch_translation_tables();
        }
    }
}

#[inline(never)]
pub(crate) unsafe fn reset_cntvoff() {
    unsafe {
        asm!("mcrr p15, 4, {val}, {val}, c14", val = in(reg) 0);
    }
}

const CPSR_MODE_MASK: usize = 0x1f;
const CPSR_MODE_HYPERVISOR: usize = 0x1a;

fn is_hyp_mode() -> bool {
    let mut val: usize;
    unsafe {
        asm!("mrs {val}, cpsr", val = out(reg) val);
    }
    (val & CPSR_MODE_MASK) == CPSR_MODE_HYPERVISOR
}
