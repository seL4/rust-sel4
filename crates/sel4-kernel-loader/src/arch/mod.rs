//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(ARCH_ARM)] {
        #[path = "arm/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(ARCH_RISCV)] {
        #[path = "riscv/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(false)]
mod arm;
#[cfg(false)]
mod riscv;

pub(crate) use imp::*;

pub(crate) trait Arch {
    fn init() {}

    fn idle() -> !;

    fn prepare_to_enter_kernel(core_id: usize);
}
