//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::sync::atomic::{AtomicBool, Ordering};

use sel4_panicking_env::abort;

static PANICKING: AtomicBool = AtomicBool::new(false);

pub(crate) fn count_panic() {
    if PANICKING.load(Ordering::SeqCst) {
        abort!("program is already panicking");
    }
}

pub(crate) fn count_panic_caught() {
    PANICKING.store(false, Ordering::SeqCst);
}
