//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};

static PANICKING: AtomicBool = AtomicBool::new(false);

pub(crate) enum MustAbort {
    AlreadyPanicking,
}

impl fmt::Display for MustAbort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AlreadyPanicking => write!(f, "program is already panicking"),
        }
    }
}

pub(crate) fn count_panic() -> Option<MustAbort> {
    if PANICKING.load(Ordering::SeqCst) {
        Some(MustAbort::AlreadyPanicking)
    } else {
        None
    }
}

pub(crate) fn count_panic_caught() {
    PANICKING.store(false, Ordering::SeqCst);
}
