//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(ARCH_AARCH64)] {
        #[path = "aarch64/mod.rs"]
        mod imp;
    } else if #[sel4_cfg(ARCH_AARCH32)] {
        #[path = "aarch32/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(false)]
mod aarch32;
#[cfg(false)]
mod aarch64;

pub(crate) use imp::*;
