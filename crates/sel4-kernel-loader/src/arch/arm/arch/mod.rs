//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

sel4_config::sel4_cfg_if! {
    if #[cfg(ARCH_AARCH64)] {
        #[path = "aarch64/mod.rs"]
        mod imp;
    } else if #[cfg(ARCH_AARCH32)] {
        #[path = "aarch32/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(any())]
mod aarch32;
#[cfg(any())]
mod aarch64;

pub(crate) use imp::*;
