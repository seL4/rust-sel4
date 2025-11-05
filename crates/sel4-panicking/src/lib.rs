//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(panic_can_unwind)]
#![feature(thread_local)]
#![allow(internal_features)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::panic::{PanicInfo, UnwindSafe};

use sel4_panicking_env::{abort, debug_println};

mod count;
mod hook;
mod strategy;

use count::{count_panic, count_panic_caught};
use hook::get_hook;

pub use hook::{PanicHook, set_hook};

#[cfg_attr(feature = "panic-handler", panic_handler)]
#[cfg_attr(not(feature = "panic-handler"), allow(dead_code))]
fn panic(info: &PanicInfo) -> ! {
    if let Some(must_abort) = count_panic() {
        debug_println!("{}", info);
        abort!("{}", must_abort);
    }
    (get_hook())(info);
    if info.can_unwind() {
        let code = strategy::begin_panic();
        abort!("failed to initiate panic, error {}", code)
    } else {
        abort!("can't unwind this panic")
    }
}

/// Like `std::panic::catch_unwind`.
#[allow(clippy::result_unit_err)]
pub fn catch_unwind<R, F: FnOnce() -> R + UnwindSafe>(f: F) -> Result<R, ()> {
    strategy::catch_unwind(f).inspect_err(|_| count_panic_caught())
}

/// Like the unstable `core::panic::abort_unwind`
pub fn abort_unwind<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    extern "C" fn wrap<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }

    wrap(f)
}
