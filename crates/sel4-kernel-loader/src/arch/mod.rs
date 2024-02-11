//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config::sel4_cfg_if;
use sel4_kernel_loader_payload_types::PayloadInfo;

sel4_cfg_if! {
    if #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))] {
        #[path = "arm/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))] {
        #[path = "riscv/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(any())]
mod arm;
#[cfg(any())]
mod riscv;

pub(crate) use imp::*;

pub(crate) trait Arch {
    type PerCore;

    fn init() {}

    fn idle() -> !;

    fn enter_kernel(
        core_id: usize,
        payload_info: &PayloadInfo<usize>,
        per_core: Self::PerCore,
    ) -> !;
}
