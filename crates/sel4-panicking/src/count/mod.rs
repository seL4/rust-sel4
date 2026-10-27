//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

cfg_select! {
    target_thread_local => {
        mod with_tls;
        use with_tls as whether_tls;
    }
    _ => {
        mod without_tls;
        use without_tls as whether_tls;
    }
}

pub(crate) use whether_tls::*;
