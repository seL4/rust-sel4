#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(core_intrinsics)]
#![feature(thread_local)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::mem::ManuallyDrop;
use core::panic::PanicInfo;

use sel4_panicking_env::abort;

mod count;
mod hook;
mod payload;
mod strategy;

use count::{count_panic, count_panic_caught};
use hook::get_hook;
use strategy::{panic_cleanup, start_panic};

pub use hook::{set_hook, PanicHook};
pub use payload::{IntoPayload, Payload, TryFromPayload};

#[cfg(not(feature = "alloc"))]
pub use payload::{FromPayloadValue, IntoPayloadValue, PayloadValue, PAYLOAD_VALUE_SIZE};

// // //

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    do_panic(Some(info), None)
}

#[track_caller]
pub fn panic_any<M: IntoPayload>(msg: M) -> ! {
    // TODO pass location
    do_panic(None, Some(msg.into_payload()))
}

fn do_panic(info: Option<&PanicInfo<'_>>, payload: Option<Payload>) -> ! {
    count_panic();
    (get_hook())(info);
    let code = start_panic(payload.unwrap_or(().into_payload()));
    abort!("failed to initiate panic, error {}", code)
}

// // //

pub fn catch_unwind<R, F: FnOnce() -> R>(f: F) -> Result<R, Payload> {
    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
        p: ManuallyDrop<Payload>,
    }

    let mut data = Data {
        f: ManuallyDrop::new(f),
    };

    let data_ptr = &mut data as *mut _ as *mut u8;
    unsafe {
        return if core::intrinsics::r#try(do_call::<F, R>, data_ptr, do_catch::<F, R>) == 0 {
            Ok(ManuallyDrop::into_inner(data.r))
        } else {
            Err(ManuallyDrop::into_inner(data.p))
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
    fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, exception: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let payload = panic_cleanup(exception);
            count_panic_caught();
            data.p = ManuallyDrop::new(payload);
        }
    }
}
