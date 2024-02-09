//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::global_asm;

use sel4_config::sel4_cfg_bool;

type PsciFunc = unsafe extern "C" fn(id: usize, param1: usize, param2: usize, param3: usize) -> i32;

extern "C" {
    fn smc_psci_func(id: usize, param1: usize, param2: usize, param3: usize) -> i32;
    fn hvc_psci_func(id: usize, param1: usize, param2: usize, param3: usize) -> i32;
}

// TODO
#[allow(clippy::if_same_then_else)]
static CHOSEN_PSCI_FUNC: PsciFunc = if sel4_cfg_bool!(ARM_HYPERVISOR_SUPPORT) {
    smc_psci_func as PsciFunc
} else {
    smc_psci_func as PsciFunc
    // hvc_psci_func as PsciFunc
};

const PSCI_FID_CPU_ON: usize = 0x84000003;

unsafe fn psci_cpu_on(target_cpu: usize, entry_point: usize, context_id: usize) {
    let ret = CHOSEN_PSCI_FUNC(PSCI_FID_CPU_ON, target_cpu, entry_point, context_id);
    assert_eq!(ret, 0);
}

pub(crate) fn start_secondary_core(core_id: usize, sp: usize) {
    let start = psci_secondary_entry as *const PsciSecondaryEntryFn as usize;
    unsafe {
        psci_cpu_on(core_id, start, sp);
    }
}

type PsciSecondaryEntryFn = extern "C" fn() -> !;

extern "C" {
    fn psci_secondary_entry() -> !;
}

global_asm! {
    r#"
        .extern secondary_entry

        .section .text

        .global psci_secondary_entry
        psci_secondary_entry:
            mov sp, r0
            b secondary_entry
    "#
}
