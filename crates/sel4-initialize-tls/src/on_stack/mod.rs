//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr;

use crate::{SetThreadPointerFn, TlsImage};

mod reserve;

use reserve::{reserve_on_stack, ReserveOnStackContArg};

pub type ContFn = fn(cont_arg: *mut ContArg) -> !;

pub enum ContArg {}

impl TlsImage {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn initialize_on_stack(
        &self,
        set_thread_pointer_fn: SetThreadPointerFn,
        cont_fn: ContFn,
        cont_arg: *mut ContArg,
    ) -> ! {
        let mut cont_arg = InternalContArg {
            tls_image: ptr::addr_of!(*self),
            set_thread_pointer_fn,
            cont_fn,
            cont_arg,
        };
        reserve_on_stack(
            self.reservation_layout().footprint(),
            reserve_on_stack_cont_fn,
            ptr::addr_of_mut!(cont_arg).cast(),
        )
    }

    unsafe fn continue_with(
        &self,
        tls_reservation_start: *mut u8,
        set_thread_pointer_fn: SetThreadPointerFn,
        cont_fn: ContFn,
        cont_arg: *mut ContArg,
    ) -> ! {
        self.initialize_reservation(tls_reservation_start);

        let thread_pointer = tls_reservation_start
            .wrapping_byte_add(self.reservation_layout().thread_pointer_offset());

        (set_thread_pointer_fn)(thread_pointer as usize);

        (cont_fn)(cont_arg)
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct InternalContArg {
    tls_image: *const TlsImage,
    set_thread_pointer_fn: SetThreadPointerFn,
    cont_fn: ContFn,
    cont_arg: *mut ContArg,
}

unsafe extern "C" fn reserve_on_stack_cont_fn(
    reservation_start: *mut u8,
    arg: *mut ReserveOnStackContArg,
) -> ! {
    let arg = arg.cast::<InternalContArg>().as_ref().unwrap();
    let tls_image = arg.tls_image.as_ref().unwrap();
    tls_image.continue_with(
        reservation_start,
        arg.set_thread_pointer_fn,
        arg.cont_fn,
        arg.cont_arg,
    )
}
