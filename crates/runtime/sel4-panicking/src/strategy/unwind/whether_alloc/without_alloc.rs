use core::cell::RefCell;
use core::mem::{self, MaybeUninit};

use unwinding::abi::*;

use crate::{drop_panic, foreign_exception, Payload};

use crate::strategy::unwind::RUST_EXCEPTION_CLASS;

struct CurrentException {
    exception_present: bool,
    exception: MaybeUninit<UnwindException>,
}

#[cfg(not(target_thread_local))]
compile_error!("");

#[thread_local]
static CURRENT_PAYLOAD: RefCell<Option<Payload>> = RefCell::new(None);

#[thread_local]
static mut CURRENT_EXCEPTION: CurrentException = CurrentException {
    exception_present: false,
    exception: MaybeUninit::uninit(),
};

pub(crate) fn panic_cleanup(exception: *mut u8) -> Payload {
    let exception = exception as *mut UnwindException;
    unsafe {
        if (*exception).exception_class != RUST_EXCEPTION_CLASS {
            _Unwind_DeleteException(exception);
            foreign_exception()
        } else {
            CURRENT_EXCEPTION.exception_present = false;
            CURRENT_PAYLOAD.replace(None).unwrap()
        }
    }
}

pub(crate) fn start_panic(payload: Payload) -> i32 {
    extern "C" fn exception_cleanup(
        _unwind_code: UnwindReasonCode,
        _exception: *mut UnwindException,
    ) {
        drop_panic()
    }

    let mut exception = unsafe { mem::zeroed::<UnwindException>() };
    exception.exception_class = RUST_EXCEPTION_CLASS;
    exception.exception_cleanup = Some(exception_cleanup);

    assert!(CURRENT_PAYLOAD.replace(Some(payload)).is_none());

    unsafe {
        assert!(!CURRENT_EXCEPTION.exception_present);
        CURRENT_EXCEPTION = CurrentException {
            exception_present: true,
            exception: MaybeUninit::new(exception),
        };
        _Unwind_RaiseException(CURRENT_EXCEPTION.exception.assume_init_mut()).0
    }
}
