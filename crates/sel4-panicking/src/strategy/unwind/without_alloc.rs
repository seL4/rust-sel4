//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::Cell;
use core::mem::MaybeUninit;

use unwinding::abi::*;
use unwinding::panicking::Exception;

use super::{ExceptionImpl, RustPanic, foreign_exception};

#[cfg(not(target_thread_local))]
compile_error!("");

#[thread_local]
static CURRENT_EXCEPTION_PRESENT: Cell<bool> = Cell::new(false);

#[thread_local]
static CURRENT_EXCEPTION: Cell<MaybeUninit<ExceptionImpl>> = Cell::new(MaybeUninit::uninit());

unsafe impl Exception for RustPanic {
    const CLASS: [u8; 8] = RustPanic::EXCEPTION_CLASS;

    fn wrap(this: Self) -> *mut UnwindException {
        assert!(!CURRENT_EXCEPTION_PRESENT.get());
        CURRENT_EXCEPTION.set(MaybeUninit::new(ExceptionImpl::new(this)));
        CURRENT_EXCEPTION_PRESENT.set(true);
        CURRENT_EXCEPTION.as_ptr().cast()
    }

    unsafe fn unwrap(ex: *mut UnwindException) -> Self {
        let Some(ex) = ExceptionImpl::check_cast(ex) else {
            foreign_exception()
        };
        assert_eq!(CURRENT_EXCEPTION.as_ptr().cast(), ex); // sanity check
        assert!(CURRENT_EXCEPTION_PRESENT.get());
        let maybe_uninit = CURRENT_EXCEPTION.replace(MaybeUninit::uninit());
        CURRENT_EXCEPTION_PRESENT.set(false);
        let this = unsafe { maybe_uninit.assume_init() };
        this.payload
    }
}
