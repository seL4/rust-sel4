//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

cfg_select! {
    panic = "unwind" => {
        mod unwind;
        use unwind as imp;
    }
    _ => {
        mod abort;
        use abort as imp;
    }
}

pub(crate) use imp::*;
