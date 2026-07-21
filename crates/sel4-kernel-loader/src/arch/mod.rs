//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config::sel4_cfg_usize;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        mod aarch64;
        pub(crate) use aarch64::*;
    } else if #[cfg(target_arch = "arm")] {
        mod aarch32;
        pub(crate) use aarch32::*;
    } else if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        mod riscv;
        pub(crate) use riscv::*;
    }
}

pub(crate) trait Arch {
    fn physical_to_logical_core_id(physical_core_id: usize) -> Option<usize> {
        if physical_core_id < sel4_cfg_usize!(MAX_NUM_NODES) {
            Some(physical_core_id)
        } else {
            None
        }
    }

    fn logical_to_physical_core_id(logical_core_id: usize) -> usize {
        logical_core_id
    }

    fn idle() -> !;

    fn prepare_to_enter_kernel(core_id: usize);
}
