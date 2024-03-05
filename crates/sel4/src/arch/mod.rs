//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_if;

// TODO
// sel4-config doesn't yet play nicely with:
//   - ARCH_ARM
//   - ARCH_RISCV
//   - ARCH_X86

sel4_cfg_if! {
    if #[sel4_cfg(any(ARCH_AARCH64, ARCH_AARCH32))] {
        #[path = "arm/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(any(ARCH_RISCV64, ARCH_RISCV32))] {
        #[path = "riscv/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(ARCH_X86_64)] {
        #[path = "x86/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(any())]
mod arm;
#[cfg(any())]
mod riscv;
#[cfg(any())]
mod x86;

pub(crate) use imp::*;
