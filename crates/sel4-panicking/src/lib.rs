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
#![cfg_attr(not(panic_info_message_stable), feature(panic_info_message))]
#![allow(internal_features)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::fmt;
use core::mem::ManuallyDrop;
use core::panic::Location;
use core::panic::{PanicInfo, UnwindSafe};

#[cfg(panic_info_message_stable)]
pub use core::panic::PanicMessage;

use cfg_if::cfg_if;

use sel4_panicking_env::abort;

mod count;
mod hook;
mod payload;
mod strategy;

use count::{count_panic, count_panic_caught};
use hook::get_hook;
use payload::NoPayload;
use strategy::{panic_cleanup, start_panic};

pub use hook::{set_hook, PanicHook};
pub use payload::{Payload, SmallPayload, UpcastIntoPayload, SMALL_PAYLOAD_MAX_SIZE};

#[cfg(not(panic_info_message_stable))]
pub type PanicMessage<'a> = &'a fmt::Arguments<'a>;

/// Information passed to a [`PanicHook`].
///
/// Analogous to `std::panic::PanicHookInfo`.
pub struct ExternalPanicInfo<'a> {
    payload: Payload,
    message: Option<PanicMessage<'a>>,
    location: Option<&'a Location<'a>>,
    can_unwind: bool,
}

impl<'a> ExternalPanicInfo<'a> {
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    pub fn message(&self) -> Option<&PanicMessage> {
        self.message.as_ref()
    }

    pub fn location(&self) -> Option<&Location> {
        self.location
    }

    pub fn can_unwind(&self) -> bool {
        self.can_unwind
    }
}

impl fmt::Display for ExternalPanicInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("panicked at ")?;
        if let Some(location) = self.location() {
            location.fmt(f)?;
        } else {
            f.write_str("unknown location")?;
        }
        if let Some(message) = self.message() {
            f.write_str(":\n")?;
            message.fmt(f)?;
        }
        Ok(())
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    do_panic(ExternalPanicInfo {
        payload: NoPayload.upcast_into_payload(),
        #[cfg(panic_info_message_stable)]
        message: Some(info.message()),
        #[cfg(not(panic_info_message_stable))]
        message: info.message(),
        location: info.location(),
        can_unwind: info.can_unwind(),
    })
}

/// Like `std::panic::panic_any`.
#[track_caller]
pub fn panic_any<M: UpcastIntoPayload>(msg: M) -> ! {
    do_panic(ExternalPanicInfo {
        payload: msg.upcast_into_payload(),
        message: None,
        location: Some(Location::caller()),
        can_unwind: true,
    })
}

fn do_panic(info: ExternalPanicInfo) -> ! {
    count_panic();
    (get_hook())(&info);
    if info.can_unwind() {
        let code = start_panic(info.payload);
        abort!("failed to initiate panic, error {}", code)
    } else {
        abort!("can't unwind this panic")
    }
}

cfg_if! {
    if #[cfg(catch_unwind_intrinsic_still_named_try)] {
        use core::intrinsics::r#try as catch_unwind_intrinsic;
    } else {
        use core::intrinsics::catch_unwind as catch_unwind_intrinsic;
    }
}

/// Like `std::panic::catch_unwind`.
pub fn catch_unwind<R, F: FnOnce() -> R + UnwindSafe>(f: F) -> Result<R, Payload> {
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
        return if catch_unwind_intrinsic(do_call::<F, R>, data_ptr, do_catch::<F, R>) == 0 {
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
