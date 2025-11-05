//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ffi::c_int;
use core::mem::{self, MaybeUninit};
use core::panic::UnwindSafe;
use core::ptr;

use unwinding::abi::*;

use sel4_panicking_env::abort;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod with_alloc;
    } else {
        mod without_alloc;
    }
}

struct DropGuard;

impl Drop for DropGuard {
    fn drop(&mut self) {
        drop_panic()
    }
}

#[repr(transparent)]
struct RustPanic {
    drop_guard: DropGuard,
}

impl RustPanic {
    const EXCEPTION_CLASS: [u8; 8] = *b"MOZ\0RUST";

    const fn new() -> Self {
        Self {
            drop_guard: DropGuard,
        }
    }
}

#[repr(C)]
struct ExceptionImpl {
    exception: MaybeUninit<UnwindException>,
    // See rust/library/panic_unwind/src/gcc.rs for the canary values
    canary: *const u8,
    payload: RustPanic,
}

static CANARY: u8 = 0;

impl ExceptionImpl {
    fn new(payload: RustPanic) -> Self {
        Self {
            exception: MaybeUninit::uninit(),
            canary: &CANARY,
            payload,
        }
    }

    fn check_cast(ex: *mut UnwindException) -> Option<*mut Self> {
        let this = ex as *mut Self;
        let canary = unsafe { ptr::addr_of!((*this).canary).read() };
        if ptr::eq(canary, &CANARY) {
            Some(this)
        } else {
            None
        }
    }
}

pub(crate) fn begin_panic() -> c_int {
    unwinding::panicking::begin_panic(RustPanic::new()).0
}

#[allow(clippy::result_unit_err)]
pub fn catch_unwind<R, F: FnOnce() -> R + UnwindSafe>(f: F) -> Result<R, ()> {
    #[cold]
    fn process_panic(p: Option<RustPanic>) {
        match p {
            None => foreign_exception(),
            Some(RustPanic { drop_guard }) => {
                mem::forget(drop_guard);
            }
        }
    }
    unwinding::panicking::catch_unwind(f).map_err(process_panic)
}

fn drop_panic() -> ! {
    abort!("Rust panics must be rethrown");
}

fn foreign_exception() -> ! {
    abort!("Rust cannot catch foreign exceptions");
}
