//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::boxed::Box;
use core::mem;

use unwinding::abi::*;

use super::{drop_panic, foreign_exception, RUST_EXCEPTION_CLASS};
use crate::Payload;

#[repr(C)]
struct ExceptionWithPayload {
    exception: UnwindException,
    payload: Payload,
}

pub(crate) fn panic_cleanup(exception: *mut u8) -> Payload {
    let exception = exception as *mut UnwindException;
    unsafe {
        if (*exception).exception_class != RUST_EXCEPTION_CLASS {
            _Unwind_DeleteException(exception);
            foreign_exception()
        } else {
            let exception = Box::from_raw(exception as *mut ExceptionWithPayload);
            exception.payload
        }
    }
}

pub(crate) fn start_panic(payload: Payload) -> i32 {
    extern "C" fn exception_cleanup(
        _unwind_code: UnwindReasonCode,
        exception: *mut UnwindException,
    ) {
        unsafe {
            let _: Box<ExceptionWithPayload> =
                Box::from_raw(exception as *mut ExceptionWithPayload);
        }
        drop_panic()
    }

    let mut exception = unsafe { mem::zeroed::<UnwindException>() };
    exception.exception_class = RUST_EXCEPTION_CLASS;
    exception.exception_cleanup = Some(exception_cleanup);
    let exception = Box::into_raw(Box::new(ExceptionWithPayload { exception, payload }))
        as *mut UnwindException;
    unsafe { _Unwind_RaiseException(&mut *exception).0 }
}
