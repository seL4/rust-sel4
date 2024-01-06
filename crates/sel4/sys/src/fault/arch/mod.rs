//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[cfg(ARCH_AARCH64)] {
        #[path = "aarch64.rs"]
        mod imp;
    } else if #[cfg(ARCH_AARCH32)] {
        #[path = "aarch32.rs"]
        mod imp;
    } else if #[cfg(any(ARCH_RISCV64, ARCH_RISCV32))] {
        #[path = "riscv.rs"]
        mod imp;
    } else if #[cfg(ARCH_X86_64)] {
        #[path = "x86_64.rs"]
        mod imp;
    }
}
