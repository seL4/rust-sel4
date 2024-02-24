//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::arch::asm;
use core::ptr;

use cfg_if::cfg_if;

use crate::TlsImage;

mod reserve;

use reserve::{reserve_on_stack, ReserveOnStackContArg};

pub type ContFn = unsafe extern "C" fn(cont_arg: *mut ContArg) -> !;

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
        self.initialize_tls_reservation(tls_reservation_start);

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

pub type SetThreadPointerFn = unsafe extern "C" fn(thread_pointer: usize);

pub const DEFAULT_SET_THREAD_POINTER_FN: SetThreadPointerFn = default_set_thread_pointer;

unsafe extern "C" fn default_set_thread_pointer(thread_pointer: usize) {
    let val = thread_pointer;

    cfg_if! {
        if #[cfg(target_arch = "aarch64")] {
            asm!("msr tpidr_el0, {val}", val = in(reg) val);
        } else if #[cfg(target_arch = "arm")] {
            // TODO which register to use must match with LLVM -mtp, I think
            asm!("mcr p15, 0, {val}, c13, c0, 2", val = in(reg) val);
        } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
            asm!("mv tp, {val}", val = in(reg) val);
        } else if #[cfg(target_arch = "x86_64")] {
            asm!("wrfsbase {val}", val = in(reg) val);
        } else {
            compile_error!("unsupported architecture");
        }
    }
}
