//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use crate::arch::secondary_entry;

pub(crate) fn start_core(physical_core_id: usize, sp: usize) {
    let start = secondary_entry as *const () as usize;
    smccc::psci::cpu_on::<smccc::Smc>(
        physical_core_id.try_into().unwrap(),
        start.try_into().unwrap(),
        sp.try_into().unwrap(),
    )
    .unwrap();
}
