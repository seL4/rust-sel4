//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[sel4_cfg(ARCH_X86_64)] {
        #[path = "x64/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(false)]
mod x64;

pub(crate) use imp::*;
