//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg;

// [TODO]
// sel4-config doesn't yet play nicely with:
//   - ARCH_ARM
//   - ARCH_RISCV
//   - ARCH_X86

#[sel4_cfg(any(ARCH_AARCH32, ARCH_AARCH64))]
#[path = "arm/mod.rs"]
mod imp;

#[sel4_cfg(any(ARCH_IA32, ARCH_X86_64))]
#[path = "x86/mod.rs"]
mod imp;

#[sel4_cfg(any(ARCH_RISCV32, ARCH_RISCV64))]
#[path = "riscv/mod.rs"]
mod imp;

pub(crate) use imp::*;
