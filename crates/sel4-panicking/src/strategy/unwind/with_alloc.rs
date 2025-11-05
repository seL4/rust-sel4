//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::boxed::Box;

use unwinding::abi::*;
use unwinding::panicking::Exception;

use super::{ExceptionImpl, RustPanic, foreign_exception};

unsafe impl Exception for RustPanic {
    const CLASS: [u8; 8] = RustPanic::EXCEPTION_CLASS;

    fn wrap(this: Self) -> *mut UnwindException {
        Box::into_raw(Box::new(ExceptionImpl::new(this))).cast()
    }

    unsafe fn unwrap(ex: *mut UnwindException) -> Self {
        let Some(ex) = ExceptionImpl::check_cast(ex) else {
            foreign_exception()
        };
        let ex = unsafe { Box::from_raw(ex) };
        ex.payload
    }
}
