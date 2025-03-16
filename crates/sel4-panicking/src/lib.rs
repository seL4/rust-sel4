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

use core::intrinsics::catch_unwind as catch_unwind_intrinsic;
use core::mem::ManuallyDrop;
use core::panic::{PanicInfo, UnwindSafe};

use sel4_panicking_env::{abort, debug_println};

mod count;
mod hook;
mod strategy;

use count::{count_panic, count_panic_caught};
use hook::get_hook;
use strategy::{panic_cleanup, start_panic};

pub use hook::{set_hook, PanicHook};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(must_abort) = count_panic() {
        debug_println!("{}", info);
        abort!("{}", must_abort);
    }
    (get_hook())(info);
    if info.can_unwind() {
        let code = start_panic();
        abort!("failed to initiate panic, error {}", code)
    } else {
        abort!("can't unwind this panic")
    }
}

/// Like `std::panic::catch_unwind`.
#[allow(clippy::result_unit_err)]
pub fn catch_unwind<R, F: FnOnce() -> R + UnwindSafe>(f: F) -> Result<R, ()> {
    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
    }

    let mut data = Data {
        f: ManuallyDrop::new(f),
    };

    let data_ptr = (&raw mut data) as *mut u8;
    unsafe {
        return if catch_unwind_intrinsic(do_call::<F, R>, data_ptr, do_catch) == 0 {
            Ok(ManuallyDrop::into_inner(data.r))
        } else {
            Err(())
        };
    }

    #[inline]
    fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let f = ManuallyDrop::take(&mut data.f);
            data.r = ManuallyDrop::new(f());
        }
    }

    #[inline]
    fn do_catch(_data: *mut u8, exception: *mut u8) {
        panic_cleanup(exception);
        count_panic_caught();
    }
}

/// Like `std::panic::abort_unwind`.
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
