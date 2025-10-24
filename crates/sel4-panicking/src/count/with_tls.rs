//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::Cell;
use core::fmt;

#[thread_local]
static PANIC_COUNT: Cell<usize> = Cell::new(0);

const MAX_PANIC_DEPTH: usize = if cfg!(feature = "alloc") { 3 } else { 1 };

pub(crate) enum MustAbort {
    MaxDepthExceeded,
}

impl fmt::Display for MustAbort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MaxDepthExceeded => {
                write!(f, "maximum panic depth of {MAX_PANIC_DEPTH} exceeded")
            }
        }
    }
}

pub(crate) fn count_panic() -> Option<MustAbort> {
    if PANIC_COUNT.get() == MAX_PANIC_DEPTH {
        return Some(MustAbort::MaxDepthExceeded);
    }
    PANIC_COUNT.update(|count| count + 1);
    None
}

pub(crate) fn count_panic_caught() {
    PANIC_COUNT.update(|count| count - 1);
}
