//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: MIT
//

use sel4_config::sel4_cfg_if;

sel4_cfg_if! {
    if #[cfg(ARCH_X86_64)] {
        #[path = "x64/mod.rs"]
        mod imp;
    }
}

// HACK for rustfmt
#[cfg(any())]
mod x64;

pub(crate) use imp::*;
