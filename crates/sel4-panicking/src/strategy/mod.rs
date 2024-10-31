//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

cfg_if::cfg_if! {
    if #[cfg(panic = "unwind")] {
        mod unwind;
        use unwind as imp;
    } else {
        mod abort;
        use abort as imp;
    }
}

pub(crate) use imp::*;
