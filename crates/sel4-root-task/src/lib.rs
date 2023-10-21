//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![feature(cfg_target_thread_local)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use core::ffi::c_char;
use core::fmt;

#[cfg(target_thread_local)]
use core::ffi::c_void;

pub use sel4_panicking_env::{abort, debug_print, debug_println};
pub use sel4_root_task_macros::root_task;

#[doc(inline)]
pub use sel4_panicking as panicking;

mod termination;

use termination::Termination;

#[cfg(target_thread_local)]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    unsafe extern "C" fn cont_fn(cont_arg: *mut c_void) -> ! {
        let bootinfo = cont_arg.cast_const().cast::<sel4::sys::seL4_BootInfo>();
        inner_entry(bootinfo)
    }

    let cont_arg = bootinfo.cast::<c_void>().cast_mut();

    sel4_runtime_common::locate_tls_image()
        .unwrap()
        .initialize_on_stack_and_continue(cont_fn, cont_arg)
}

#[cfg(not(target_thread_local))]
#[no_mangle]
unsafe extern "C" fn sel4_runtime_rust_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    inner_entry(bootinfo)
}

unsafe extern "C" fn inner_entry(bootinfo: *const sel4::sys::seL4_BootInfo) -> ! {
    #[cfg(feature = "unwinding")]
    {
        sel4_runtime_common::set_eh_frame_finder().unwrap();
    }

    let ipc_buffer = sel4::BootInfo::from_ptr(bootinfo).ipc_buffer();
    sel4::set_ipc_buffer(ipc_buffer);
    __sel4_root_task_main(bootinfo);
    abort!("main thread returned")
}

extern "C" {
    fn __sel4_root_task_main(bootinfo: *const sel4::sys::seL4_BootInfo);
}

#[doc(hidden)]
#[macro_export]
macro_rules! declare_main {
    ($main:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn __sel4_root_task_main(
            bootinfo: *const $crate::_private::seL4_BootInfo,
        ) {
            $crate::_private::run_main($main, bootinfo);
        }
    };
}

#[doc(hidden)]
#[allow(clippy::missing_safety_doc)]
pub unsafe fn run_main<T>(
    f: impl Fn(&sel4::BootInfo) -> T,
    bootinfo: *const sel4::sys::seL4_BootInfo,
) where
    T: Termination,
    T::Error: fmt::Debug,
{
    match panicking::catch_unwind(|| {
        let bootinfo = sel4::BootInfo::from_ptr(bootinfo);
        f(&bootinfo).report()
    }) {
        Ok(err) => abort!("main thread terminated with error: {err:?}"),
        Err(_) => abort!("main thread panicked"),
    }
}

#[no_mangle]
fn sel4_runtime_debug_put_char(c: u8) {
    sel4::debug_put_char(c as c_char)
}

#[macro_export]
macro_rules! declare_root_task {
    {
       main = $main:expr $(,)?
   } => {
       $crate::_private::declare_root_task! {
           main = $main,
           stack_size = $crate::_private::DEFAULT_STACK_SIZE,
       }
   };
   {
       main = $main:expr,
       stack_size = $stack_size:expr $(,)?
   } => {
       $crate::_private::declare_main!($main);
       $crate::_private::declare_stack!($stack_size);
   };
   {
       main = $main:expr,
       $(stack_size = $stack_size:expr,)?
       heap_size = $heap_size:expr $(,)?
   } => {
       $crate::_private::declare_static_heap! {
           #[doc(hidden)]
           __GLOBAL_ALLOCATOR: $heap_size;
       }
       $crate::_private::declare_root_task! {
           main = $main,
           $(stack_size = $stack_size,)?
       }
   };
}

// For macros
#[doc(hidden)]
pub mod _private {
    pub use sel4::sys::seL4_BootInfo;
    pub use sel4_runtime_common::{declare_stack, declare_static_heap};

    pub use crate::{declare_main, declare_root_task, run_main};

    pub const DEFAULT_STACK_SIZE: usize = 0x10000;
}
